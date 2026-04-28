#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct I256 {
    pub(crate) hi: i128,
    pub(crate) lo: u128,
}

impl I256 {
    #[inline]
    pub const fn from_i128(x: i128) -> Self {
        Self {
            hi: x >> 127,
            lo: x as u128,
        }
    }

    #[inline]
    pub const fn sub(self, other: Self) -> Self {
        let (lo, borrow) = self.lo.overflowing_sub(other.lo);
        let hi = self.hi.wrapping_sub(other.hi).wrapping_sub(borrow as i128);
        Self { hi, lo }
    }

    #[inline]
    pub const fn sar(self, bits: u32) -> Self {
        if bits >= 256 {
            let sign = self.hi >> 127;
            Self {
                hi: sign,
                lo: sign as u128,
            }
        } else if bits >= 128 {
            let shift = bits - 128;
            let sign = self.hi >> 127;
            let lo = (self.hi >> shift) as u128;
            Self { hi: sign, lo }
        } else if bits > 0 {
            let hi_shifted = self.hi >> bits;
            let lo_shifted = self.lo >> bits;
            let carry = (self.hi as u128) << (128 - bits);
            Self {
                hi: hi_shifted,
                lo: lo_shifted | carry,
            }
        } else {
            self
        }
    }

    #[inline]
    pub const fn mul_u32(self, n: u32) -> Self {
        // Fixed-width modulo-2^256 arithmetic
        let n = n as u128;

        let lo_0 = (self.lo as u64) as u128;
        let hi_0 = self.lo >> 64;

        let p0 = lo_0.wrapping_mul(n);
        let p1 = hi_0.wrapping_mul(n);

        let carry_0 = p0 >> 64;
        let sum_1 = p1.wrapping_add(carry_0);

        const MASK64: u128 = 0xFFFF_FFFF_FFFF_FFFF;
        let lo_low64 = p0 & MASK64;
        let lo_high64 = sum_1 & MASK64;
        let lo = lo_low64 | (lo_high64 << 64);
        let carry_1 = sum_1 >> 64;

        let hi = self
            .hi
            .wrapping_mul(n as i128)
            .wrapping_add(carry_1 as i128);

        Self { hi, lo }
    }

    #[inline]
    pub const fn clamp_to_i128(self) -> i128 {
        let required_hi = (self.lo as i128) >> 127;
        if self.hi == required_hi {
            self.lo as i128
        } else if self.hi < 0 {
            i128::MIN
        } else {
            i128::MAX
        }
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    // --- Byte-Level Oracle Implementation (Little Endian) ---

    // Note: I256 comprises `lo: u128` (bytes 0..15) and `hi: i128` (bytes 16..31).
    // The LE representation maps bit 0 to byte 0, bit 0, and bit 255 to byte 31, bit 7.
    fn to_bytes_le(val: I256) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        bytes[0..16].copy_from_slice(&val.lo.to_le_bytes());
        bytes[16..32].copy_from_slice(&val.hi.to_le_bytes());
        bytes
    }

    fn from_bytes_le(bytes: [u8; 32]) -> I256 {
        // Manual copy avoids try_into().unwrap() — total, panic-free under all CBMC paths.
        let mut lo_bytes = [0u8; 16];
        let mut hi_bytes = [0u8; 16];
        let mut i = 0;
        while i < 16 {
            lo_bytes[i] = bytes[i];
            i += 1;
        }
        let mut i = 0;
        while i < 16 {
            hi_bytes[i] = bytes[16 + i];
            i += 1;
        }
        let lo = u128::from_le_bytes(lo_bytes);
        let hi = i128::from_le_bytes(hi_bytes);
        I256 { hi, lo }
    }

    fn spec_sub_le(a: [u8; 32], b: [u8; 32]) -> [u8; 32] {
        let mut result = [0u8; 32];
        let mut borrow = 0u16;
        for i in 0..32 {
            let diff = (a[i] as u16).wrapping_sub(b[i] as u16).wrapping_sub(borrow);
            result[i] = diff as u8;
            borrow = (diff >> 8) & 1;
        }
        result
    }

    fn get_bit(bytes: &[u8; 32], bit_idx: usize) -> u8 {
        let byte_idx = bit_idx / 8;
        let bit_in_byte = bit_idx % 8;
        (bytes[byte_idx] >> bit_in_byte) & 1
    }

    fn set_bit(bytes: &mut [u8; 32], bit_idx: usize, val: u8) {
        let byte_idx = bit_idx / 8;
        let bit_in_byte = bit_idx % 8;
        if val == 1 {
            bytes[byte_idx] |= 1 << bit_in_byte;
        } else {
            bytes[byte_idx] &= !(1 << bit_in_byte);
        }
    }

    fn spec_sar_le(a: [u8; 32], shift: u32) -> [u8; 32] {
        let mut result = [0u8; 32];
        let sign_bit = get_bit(&a, 255);
        let shift_usize = shift as usize;

        if shift_usize >= 256 {
            let ext = if sign_bit == 1 { 0xFF } else { 0x00 };
            result.fill(ext);
            return result;
        }

        for i in 0..256 {
            let src_idx = i + shift_usize;
            let bit_val = if src_idx < 256 {
                get_bit(&a, src_idx)
            } else {
                sign_bit
            };
            set_bit(&mut result, i, bit_val);
        }
        result
    }

    fn spec_add_le(x: [u8; 32], y: [u8; 32]) -> [u8; 32] {
        let mut result = [0u8; 32];
        let mut carry = 0u16;
        for i in 0..32 {
            let sum = x[i] as u16 + y[i] as u16 + carry;
            result[i] = sum as u8;
            carry = sum >> 8;
        }
        // carry-out beyond 256 bits discarded (mod 2^256)
        result
    }

    fn spec_shl_bytes_le(x: [u8; 32], k: usize) -> [u8; 32] {
        // Shift left by k bytes (multiply by 256^k) in LE representation.
        // Low indices 0..k-1 are zeroed (least-significant bytes vacated by the shift).
        // Bytes at index k..32 receive x[i-k] (the original bytes shifted toward higher indices).
        // mod 2^256: bytes shifted beyond index 31 are discarded.
        let mut result = [0u8; 32];
        for i in 0..32 {
            if i >= k {
                result[i] = x[i - k];
            }
        }
        result
    }

    fn spec_mul_u8_le(a: [u8; 32], b: u8) -> [u8; 32] {
        let mut result = [0u8; 32];
        let mut carry = 0u16;
        for i in 0..32 {
            let prod = a[i] as u16 * b as u16 + carry;
            result[i] = prod as u8;
            carry = prod >> 8;
        }
        // carry-out beyond 256 bits discarded (mod 2^256)
        result
    }

    fn spec_mul_u32_le(a: [u8; 32], n: u32) -> [u8; 32] {
        // spec_mul_u32_le is independent of limb arithmetic.
        // Decompose n into bytes and use linearity:
        //   a * n = a*n0 + a*n1*256 + a*n2*256^2 + a*n3*256^3  (mod 2^256)
        let n_bytes = n.to_le_bytes();
        let r0 = spec_mul_u8_le(a, n_bytes[0]);
        let r1 = spec_shl_bytes_le(spec_mul_u8_le(a, n_bytes[1]), 1);
        let r2 = spec_shl_bytes_le(spec_mul_u8_le(a, n_bytes[2]), 2);
        let r3 = spec_shl_bytes_le(spec_mul_u8_le(a, n_bytes[3]), 3);
        spec_add_le(spec_add_le(r0, r1), spec_add_le(r2, r3))
    }

    fn spec_clamp_to_i128_le(a: [u8; 32]) -> i128 {
        // In-range test: upper 128 bits (bytes 16..32) must be the sign-extension
        // of bit 127 — the MSB of byte 15 (the most-significant byte of lo).
        let lo_sign = (a[15] & 0x80) != 0;
        let ext = if lo_sign { 0xFF } else { 0x00 };

        let mut in_range = true;
        for i in 16..32 {
            if a[i] != ext {
                in_range = false;
                break;
            }
        }

        if in_range {
            // Manual copy avoids try_into().unwrap() in proof code.
            let mut lo_bytes = [0u8; 16];
            let mut i = 0;
            while i < 16 {
                lo_bytes[i] = a[i];
                i += 1;
            }
            i128::from_le_bytes(lo_bytes)
        } else {
            // Saturation direction is the sign of hi (bytes 16..32), i.e.
            // bit 255 — the MSB of byte 31. This matches impl's `self.hi < 0`.
            let hi_negative = (a[31] & 0x80) != 0;
            if hi_negative {
                i128::MIN
            } else {
                i128::MAX
            }
        }
    }

    // --- Clamp spec contract micro-harnesses ---

    /// Verify: when hi bytes are sign-extension of bit127, spec returns exact lo value.
    #[kani::proof]
    fn proof_spec_clamp_in_range_contract() {
        let mut a = [0u8; 32];
        // Randomize lo (bytes 0..16) and pick a sign for bit127.
        for i in 0..16 {
            a[i] = kani::any();
        }
        let lo_sign = (a[15] & 0x80) != 0;
        let ext = if lo_sign { 0xFF } else { 0x00 };
        // Construct hi as exact sign-extension of bit127.
        for i in 16..32 {
            a[i] = ext;
        }

        let result = spec_clamp_to_i128_le(a);

        let mut lo_bytes = [0u8; 16];
        let mut i = 0;
        while i < 16 {
            lo_bytes[i] = a[i];
            i += 1;
        }
        let expected = i128::from_le_bytes(lo_bytes);
        kani::assert(result == expected, "in-range clamp spec returns exact lo");
    }

    /// Verify: when hi is NOT sign-extension of bit127, saturation uses bit255 (not bit127).
    #[kani::proof]
    fn proof_spec_clamp_out_of_range_contract() {
        let mut a = [0u8; 32];
        for i in 0..32 {
            a[i] = kani::any();
        }
        // Force out-of-range: flip the ext expected at byte 16.
        let lo_sign = (a[15] & 0x80) != 0;
        let ext = if lo_sign { 0xFF } else { 0x00 };
        // Ensure a[16] disagrees with ext, making hi ≠ required sign-extension.
        a[16] = !ext;

        let result = spec_clamp_to_i128_le(a);

        // Saturation direction must be determined by bit255 (a[31] MSB).
        let hi_negative = (a[31] & 0x80) != 0;
        if hi_negative {
            kani::assert(
                result == i128::MIN,
                "out-of-range negative uses bit255 → MIN",
            );
        } else {
            kani::assert(
                result == i128::MAX,
                "out-of-range positive uses bit255 → MAX",
            );
        }
    }

    // --- SAR spec sanity micro-harness ---
    // Checks three known fixed patterns at fixed shifts to catch endianness
    // inversions in get_bit/set_bit without requiring full symbolic proof.
    #[kani::proof]
    fn proof_spec_sar_sanity() {
        // Pattern A: only bit 255 set (MSB of a[31] = 0x80, all else 0x00).
        // sign_bit = 1. sar(1):
        //   result bit i = input bit (i+1) for i+1<256, else sign_bit (=1).
        //   Only i=254 gets input bit 255 = 1; i=255 gets sign_bit = 1.
        //   All other bits: input bit (i+1) = 0.
        //   => byte31 = bits[248..255] = 0b11000000 = 0xC0
        //   => bytes 0..30 = 0x00 (no input bits set there after shift)
        let mut only_msb = [0u8; 32];
        only_msb[31] = 0x80;
        let shifted_msb = spec_sar_le(only_msb, 1);
        kani::assert(
            shifted_msb[31] == 0xC0,
            "sar sanity A: bit255 sar(1) -> byte31=0xC0",
        );
        kani::assert(
            shifted_msb[30] == 0x00,
            "sar sanity A: bit255 sar(1) -> byte30=0x00",
        );
        kani::assert(
            shifted_msb[0] == 0x00,
            "sar sanity A: bit255 sar(1) -> byte0=0x00",
        );

        // Pattern B: only bit 0 set (LSB of a[0] = 0x01, all else 0x00).
        // Value is positive (sign_bit=0). sar(1):
        //   result bit i = input bit (i+1). Input bit 0 disappears. All zero.
        let mut only_lsb = [0u8; 32];
        only_lsb[0] = 0x01;
        let shifted_lsb = spec_sar_le(only_lsb, 1);
        kani::assert(
            shifted_lsb[0] == 0x00,
            "sar sanity B: bit0 sar(1) -> byte0=0x00",
        );
        kani::assert(
            shifted_lsb[31] == 0x00,
            "sar sanity B: bit0 sar(1) -> byte31=0x00",
        );

        // Pattern C: alternating 0xAA bytes, sign bit cleared (positive).
        // alt[31]=0x2A (0b0010_1010), alt[0..30]=0xAA (0b1010_1010). sign_bit=0.
        // sar(8) shifts right 8 bits (1 byte). Result:
        //   result byte 31 = sign-ext byte = 0x00 (positive)
        //   result byte 30 = original byte 31 = 0x2A
        //   result byte 0  = original byte 1  = 0xAA
        let mut alt = [0xAAu8; 32];
        alt[31] = 0x2A;
        let shifted_alt = spec_sar_le(alt, 8);
        kani::assert(
            shifted_alt[31] == 0x00,
            "sar sanity C: positive sar(8) -> byte31=0x00",
        );
        kani::assert(
            shifted_alt[30] == 0x2A,
            "sar sanity C: positive sar(8) -> byte30=0x2A",
        );
        kani::assert(
            shifted_alt[0] == 0xAA,
            "sar sanity C: positive sar(8) -> byte0=0xAA",
        );
    }

    // --- Formal Proofs against the Spec ---

    #[kani::proof]
    fn proof_i256_sub_matches_spec() {
        let a = I256 {
            hi: kani::any(),
            lo: kani::any(),
        };
        let b = I256 {
            hi: kani::any(),
            lo: kani::any(),
        };
        let impl_res = a.sub(b);

        let a_bytes = to_bytes_le(a);
        let b_bytes = to_bytes_le(b);
        let spec_res_bytes = spec_sub_le(a_bytes, b_bytes);
        let spec_res = from_bytes_le(spec_res_bytes);

        kani::assert(impl_res == spec_res, "I256::sub matches byte-level spec");
    }

    #[kani::proof]
    #[kani::unwind(257)]
    fn proof_i256_sar_in_range_matches_spec() {
        let a = I256 {
            hi: kani::any(),
            lo: kani::any(),
        };
        let shift: u32 = kani::any();
        // Specifically testing within boundary
        kani::assume(shift < 256);

        let impl_res = a.sar(shift);
        let a_bytes = to_bytes_le(a);
        let spec_res_bytes = spec_sar_le(a_bytes, shift);
        let spec_res = from_bytes_le(spec_res_bytes);

        kani::assert(
            impl_res == spec_res,
            "I256::sar matches byte-level spec (<256)",
        );
    }

    #[kani::proof]
    fn proof_i256_sar_out_of_range_matches_spec() {
        let a = I256 {
            hi: kani::any(),
            lo: kani::any(),
        };
        let shift: u32 = kani::any();
        kani::assume(shift >= 256);

        let impl_res = a.sar(shift);
        let a_bytes = to_bytes_le(a);
        let spec_res_bytes = spec_sar_le(a_bytes, shift);
        let spec_res = from_bytes_le(spec_res_bytes);

        kani::assert(
            impl_res == spec_res,
            "I256::sar matches byte-level spec (>=256)",
        );
    }

    #[kani::proof]
    fn proof_i256_mul_u32_matches_spec() {
        let a = I256 {
            hi: kani::any(),
            lo: kani::any(),
        };
        let n: u32 = kani::any();
        let impl_res = a.mul_u32(n);

        let a_bytes = to_bytes_le(a);
        let spec_res_bytes = spec_mul_u32_le(a_bytes, n);
        let spec_res = from_bytes_le(spec_res_bytes);

        kani::assert(
            impl_res == spec_res,
            "I256::mul_u32 matches byte-level spec",
        );
    }

    #[kani::proof]
    fn proof_i256_clamp_matches_spec() {
        let a = I256 {
            hi: kani::any(),
            lo: kani::any(),
        };
        let impl_res = a.clamp_to_i128();

        let a_bytes = to_bytes_le(a);
        let spec_res = spec_clamp_to_i128_le(a_bytes);

        kani::assert(
            impl_res == spec_res,
            "I256::clamp_to_i128 matches byte-level spec",
        );
    }
}
