//! Formal Verification Harnesses (Opt-In)
//!
//! This module is only active when Kani is running.
//! It uses Kani to prove mathematical invariants of the core signal kernels.

#[cfg(feature = "verification-runtime")]
use crate::{
    checksum::fletcher32_checked, DP32_MAGIC, HEADER_CHECKSUM_OFFSET, HEADER_CHECKSUM_SIZE,
    HEADER_RESERVED_OFFSET, HEADER_RESERVED_SIZE, HEADER_SIZE,
};

#[cfg(feature = "verification-runtime")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationError {
    MagicMismatch,
    ReservedBytesNotEmpty,
    ChecksumMismatch,
    ChecksumRegionInvalid,
    TruncatedFrame {
        frame_index: usize,
        bytes_read: usize,
    },
    #[cfg(feature = "verification-runtime")]
    Io(io_helpers::ErrorKind),
    FrameSizeInvalid,
}

#[cfg(any(test, feature = "verification-runtime"))]
pub mod io_helpers {
    pub use std::error::Error;
    pub use std::io::{ErrorKind, Read};

    pub fn kind_str(k: ErrorKind) -> &'static str {
        match k {
            ErrorKind::NotFound => "not_found",
            ErrorKind::PermissionDenied => "permission_denied",
            ErrorKind::ConnectionRefused => "connection_refused",
            ErrorKind::ConnectionReset => "connection_reset",
            ErrorKind::ConnectionAborted => "connection_aborted",
            ErrorKind::AddrInUse => "addr_in_use",
            ErrorKind::AddrNotAvailable => "addr_not_available",
            ErrorKind::BrokenPipe => "broken_pipe",
            ErrorKind::AlreadyExists => "already_exists",
            ErrorKind::WouldBlock => "would_block",
            ErrorKind::InvalidInput => "invalid_input",
            ErrorKind::InvalidData => "invalid_data",
            ErrorKind::TimedOut => "timed_out",
            ErrorKind::WriteZero => "write_zero",
            ErrorKind::UnexpectedEof => "unexpected_eof",
            ErrorKind::Interrupted => "interrupted",
            ErrorKind::OutOfMemory => "out_of_memory",
            ErrorKind::NotConnected => "not_connected",
            ErrorKind::Unsupported => "unsupported",
            _ => "other",
        }
    }
}

#[cfg(feature = "verification-runtime")]
impl core::fmt::Display for VerificationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            VerificationError::MagicMismatch => write!(f, "magic_mismatch"),
            VerificationError::ReservedBytesNotEmpty => write!(f, "reserved_bytes_not_empty"),
            VerificationError::ChecksumMismatch => write!(f, "checksum_mismatch"),
            VerificationError::ChecksumRegionInvalid => write!(f, "checksum_region_invalid"),
            VerificationError::TruncatedFrame {
                frame_index,
                bytes_read,
            } => {
                write!(
                    f,
                    "truncated_frame frame={} bytes_read={}",
                    frame_index, bytes_read
                )
            }
            VerificationError::Io(kind) => {
                write!(f, "io_error kind={}", io_helpers::kind_str(*kind))
            }
            VerificationError::FrameSizeInvalid => {
                write!(f, "frame_size_invalid_exact_64_required")
            }
        }
    }
}

#[cfg(feature = "verification-runtime")]
impl io_helpers::Error for VerificationError {}

#[cfg(feature = "verification-runtime")]
pub struct HeaderVerifier;

#[cfg(feature = "verification-runtime")]
impl HeaderVerifier {
    /// Strictly verify a 64-byte buffer as a DP32 header frame.
    ///
    /// # Protocol Definition
    /// * Exactly 64 bytes.
    /// * Bytes `[0..4)` equal `b"DP32"`.
    /// * Bytes `[56..60)` are all zero.
    /// * Bytes `[60..64)` contain a LE `u32` equal to `Fletcher32(frame[0..60])`.
    pub fn verify_frame_exact(buffer: &[u8]) -> Result<(), VerificationError> {
        // 1.3 Exact Bounds Safety
        if buffer.len() != HEADER_SIZE {
            return Err(VerificationError::FrameSizeInvalid);
        }

        // 1. Magic Verification
        if buffer[0..4] != *DP32_MAGIC {
            return Err(VerificationError::MagicMismatch);
        }

        // 2. Reserved Bytes Verification
        // Must be zero on wire.
        let reserved =
            &buffer[HEADER_RESERVED_OFFSET..HEADER_RESERVED_OFFSET + HEADER_RESERVED_SIZE];
        if reserved.iter().any(|&b| b != 0) {
            return Err(VerificationError::ReservedBytesNotEmpty);
        }

        // 3. Checksum Verification
        // Checksum is at [60..64]. It covers [0..60].
        let stored_checksum = u32::from_le_bytes(
            buffer[HEADER_CHECKSUM_OFFSET..HEADER_CHECKSUM_OFFSET + HEADER_CHECKSUM_SIZE]
                .try_into()
                .map_err(|_| VerificationError::ChecksumRegionInvalid)?,
        );

        // 1.5 Checksum Semantics: Use fletcher32_checked
        // Map Err to ChecksumRegionInvalid (never Mismatch)
        let computed = fletcher32_checked(&buffer[0..HEADER_CHECKSUM_OFFSET])
            .map_err(|_| VerificationError::ChecksumRegionInvalid)?;

        if stored_checksum != computed {
            return Err(VerificationError::ChecksumMismatch);
        }

        Ok(())
    }

    /// Verify a stream of headers from a Reader.
    ///
    /// # Streaming Requirements
    /// * Constant memory (64-byte buffer).
    /// * Distinguish clean EOF from truncated frame.
    /// * Return exact frame count.
    pub fn verify_header_stream<R: io_helpers::Read>(
        mut reader: R,
    ) -> Result<usize, VerificationError> {
        let mut buffer = [0u8; HEADER_SIZE];
        let mut frame_count = 0;

        loop {
            // Zero-initialization before read (Security/Hygiene)
            buffer.fill(0);

            // Attempt to read exactly 64 bytes
            let mut offset = 0;
            while offset < HEADER_SIZE {
                match reader.read(&mut buffer[offset..]) {
                    Ok(0) => {
                        // EOF
                        if offset == 0 {
                            return Ok(frame_count);
                        } else {
                            return Err(VerificationError::TruncatedFrame {
                                frame_index: frame_count,
                                bytes_read: offset,
                            });
                        }
                    }
                    Ok(n) => {
                        offset += n;
                    }
                    Err(e) if e.kind() == io_helpers::ErrorKind::Interrupted => {
                        continue;
                    }
                    Err(e) => return Err(VerificationError::Io(e.kind())),
                }
            }

            // We have a full 64-byte frame
            Self::verify_frame_exact(&buffer)?;
            frame_count += 1;
        }
    }

    /// Verify a byte slice as a stream of headers.
    pub fn verify_header_stream_bytes(bytes: &[u8]) -> Result<usize, VerificationError> {
        let mut offset = 0;
        let mut frame_count = 0;
        let total_len = bytes.len();

        while offset < total_len {
            let remaining = total_len - offset;
            if remaining < HEADER_SIZE {
                return Err(VerificationError::TruncatedFrame {
                    frame_index: frame_count,
                    bytes_read: remaining,
                });
            }

            let frame = &bytes[offset..offset + HEADER_SIZE];
            Self::verify_frame_exact(frame)?;

            offset += HEADER_SIZE;
            frame_count += 1;
        }

        Ok(frame_count)
    }
}

#[cfg(kani)]
use crate::*;

#[cfg(kani)]
#[kani::proof]
fn proof_compute_x2_safe() {
    let s_q31: i64 = kani::any();
    // Prove that compute_x2_q124 never panics for any i64 input.
    // This covers arithmetic overflow and the Normative 1-bit truncation.
    let _ = compute_x2_q124(s_q31);
}

#[cfg(kani)]
#[kani::proof]
fn proof_saturate_safe() {
    let val: i64 = kani::any();
    let res = saturate_i32(val);

    // Invariants (Corrected for i32 range)
    // Using kani::assert to minimize panic machinery.

    if val > 2147483647 {
        kani::assert(res == 2147483647, "Saturate high");
    } else if val < -2147483648 {
        kani::assert(res == -2147483648, "Saturate low");
    } else {
        kani::assert(res == val as i32, "Inside range");
    }
}

#[cfg(kani)]
#[kani::proof]
fn proof_apply_gain_i128_min_safe() {
    let exp: i32 = kani::any();
    let _ = apply_gain(i128::MIN, 1u64 << 63, exp);
}

#[cfg(kani)]
#[kani::proof]
fn proof_apply_gain_i128_min_exp_extremes_safe() {
    let _ = apply_gain(i128::MIN, 1u64 << 63, i32::MIN);
    let _ = apply_gain(i128::MIN, 1u64 << 63, i32::MAX);
}

#[cfg(kani)]
#[kani::proof]
fn proof_apply_gain_overflow_path_is_explicitly_saturating() {
    let pos = saturating_mul_i128((1i128 << 64) - 1, u64::MAX as i128);
    let neg = saturating_mul_i128(-((1i128 << 64) - 1), u64::MAX as i128);
    kani::assert(pos == i128::MAX, "positive overflow saturates high");
    kani::assert(neg == i128::MIN, "negative overflow saturates low");
}

#[cfg(kani)]
#[kani::proof]
fn proof_apply_gain_total_over_full_domain() {
    let raw_q124: i128 = kani::any();
    let m_q63: u64 = kani::any();
    let exp: i32 = kani::any();
    let _ = apply_gain(raw_q124, m_q63, exp);
}

// =============================================================================
// Δ-01: Phase Projection Truncation
// Closes OQ-1. Proves to_num::<u32>() on the phase-projection path:
//   (a) no intermediate panic,
//   (b) product ∈ [0, 2^32),
//   (c) result equals integer-part bit extraction (documented fixed 1.30.0 semantics).
// =============================================================================

/// Δ-01 / Proof 1: No overflow — product ∈ [0, 2^32) for all p ∈ [0, TWO_PI).
#[cfg(kani)]
#[kani::proof]
fn proof_phase_u32_no_overflow() {
    let bits: i128 = kani::any();
    let p = Scalar::from_bits(bits);
    // Precondition: p ∈ [0, TWO_PI)  — the post-wrap invariant at lib.rs:444-446.
    kani::assume(p >= Scalar::ZERO && p < math::TWO_PI);

    let product = p / math::TWO_PI * SCALE_2_32; // lib.rs:448 expression

    // Bound assertions (integer comparisons on Scalar).
    kani::assert(product >= Scalar::ZERO, "product non-negative");
    kani::assert(product < SCALE_2_32, "product < 2^32");

    let _phase_u32 = product.to_num::<u32>(); // no panic
}

/// Δ-01 / Proof 2: to_num::<u32>() equals integer-part bit extraction.
/// Validity of bit extraction: product is non-negative (asserted) and < 2^32,
/// so upper 64 bits of i128 are the non-negative integer part; cast to u64 then
/// u32 is exact (no information lost; upper 32 bits of integer part are zero).
#[cfg(kani)]
#[kani::proof]
fn proof_phase_u32_fixed_to_u32_conversion() {
    let bits: i128 = kani::any();
    let p = Scalar::from_bits(bits);
    kani::assume(p >= Scalar::ZERO && p < math::TWO_PI);

    let product = p / math::TWO_PI * SCALE_2_32;

    // Assume bounds (proven by proof_phase_u32_no_overflow).
    kani::assume(product >= Scalar::ZERO && product < SCALE_2_32);

    let via_to_num = product.to_num::<u32>();

    // Integer-only extraction: I64F64 integer part = upper 64 bits of 128-bit repr.
    // Product non-negative → i128 bits >> 64 gives the non-negative integer as u64.
    let int_as_u64 = (product.to_bits() >> 64) as u64;
    kani::assert(int_as_u64 <= u32::MAX as u64, "integer part fits u32");
    let via_bits = int_as_u64 as u32;

    kani::assert(
        via_to_num == via_bits,
        "to_num::<u32> equals integer-part bit extraction",
    );
}

// =============================================================================
// Δ-04: Sine Quantization + Multiply Safety
// Closes OQ-5 and OQ-6.
// Precondition for all three harnesses: |s| ≤ 1.0 — the documented contract
// of sin_cos_fast; CORDIC convergence already proven by proof_sin_cos_no_panic
// in geom-signal.
// These harnesses prove egress arithmetic given that contract.
// =============================================================================

// Compile-time integer-constructed Scalar constant for sine egress scaling.
#[cfg(kani)]
const SINE_SCALE: Scalar = SINE_EGRESS_SCALE;

/// Δ-04 / Proof 1: s * SINE_SCALE does not panic in I64F64 for |s| ≤ 1.
/// Closes OQ-6.
#[cfg(kani)]
#[kani::proof]
fn proof_sine_scale_no_overflow() {
    let bits: i128 = kani::any();
    let s = Scalar::from_bits(bits);
    // Precondition: sin_cos_fast contract.
    kani::assume(s >= -Scalar::ONE && s <= Scalar::ONE);
    let _ = s * SINE_SCALE; // no panic = no I64F64 overflow
}

/// Δ-04 / Proof 2: to_num::<i32>() is in range and equals signed integer-part
/// bit extraction.
/// Tight bound: scaling by `SINE_EGRESS_SCALE_Q31` under |s|≤1
/// keeps result in [−SINE_EGRESS_SCALE_Q31, SINE_EGRESS_SCALE_Q31].
/// Closes OQ-5.
#[cfg(kani)]
#[kani::proof]
fn proof_sine_to_i32_in_range() {
    let bits: i128 = kani::any();
    let s = Scalar::from_bits(bits);
    kani::assume(s >= -Scalar::ONE && s <= Scalar::ONE);

    let scaled = s * SINE_SCALE;
    let via_to_num: i32 = scaled.to_num::<i32>();

    kani::assert(
        via_to_num >= -SINE_EGRESS_SCALE_Q31,
        "within calibrated lower bound",
    );
    kani::assert(
        via_to_num <= SINE_EGRESS_SCALE_Q31,
        "within calibrated upper bound",
    );

    // Integer-only bit extraction (signed):
    // I64F64 integer part = upper 64 bits of 128-bit two's-complement repr, as i64.
    // Matches fixed's documented integer-part extraction for I64F64.
    let hi: i64 = (scaled.to_bits() >> 64) as i64;
    kani::assert(
        hi >= i32::MIN as i64 && hi <= i32::MAX as i64,
        "integer part fits i32",
    );
    kani::assert(
        via_to_num == hi as i32,
        "to_num::<i32> equals signed integer-part bit extraction",
    );
}

/// Δ-04 / Proof 3: Full egress chain output stays within the calibrated
/// sine container bound after `HEADROOM_BITS`.
#[cfg(kani)]
#[kani::proof]
fn proof_sine_egress_bounded() {
    let bits: i128 = kani::any();
    let s = Scalar::from_bits(bits);
    kani::assume(s >= -Scalar::ONE && s <= Scalar::ONE);

    // Exact egress expression from lib.rs:473-474.
    let pre_headroom = (s * SINE_SCALE).to_num::<i32>() as i128;
    let result = saturate_i128_to_i32(pre_headroom >> HEADROOM_BITS);

    let egress_cap = SINE_EGRESS_SCALE_Q31 >> HEADROOM_BITS;
    kani::assert(result >= -egress_cap, "lower egress bound");
    kani::assert(result <= egress_cap, "upper egress bound");
}

#[cfg(kani)]
#[kani::proof]
fn proof_triangle_delta_clamp_identity_when_in_range() {
    let hi: i128 = kani::any();
    let lo: u128 = kani::any();
    let delta_wide = crate::i256::I256 { hi, lo };

    let required_hi = (delta_wide.lo as i128) >> 127;
    kani::assume(delta_wide.hi == required_hi);

    kani::assert(
        delta_wide.clamp_to_i128() == (delta_wide.lo as i128),
        "in-range clamp returns exact lo as i128",
    );
}

#[cfg(kani)]
#[kani::proof]
fn proof_triangle_delta_clamp_saturates_when_out_of_range() {
    let hi: i128 = kani::any();
    let lo: u128 = kani::any();
    let delta_wide = crate::i256::I256 { hi, lo };

    let required_hi = (delta_wide.lo as i128) >> 127;
    kani::assume(delta_wide.hi != required_hi);

    let clamp = delta_wide.clamp_to_i128();
    if delta_wide.hi < 0 {
        kani::assert(clamp == i128::MIN, "negative overflow clamps to MIN");
    } else {
        kani::assert(clamp == i128::MAX, "positive overflow clamps to MAX");
    }
}

#[cfg(kani)]
#[kani::proof]
fn proof_triangle_z_update_is_saturating() {
    let z0: i128 = kani::any();
    let delta: i128 = kani::any();

    let sat = z0.saturating_add(delta);
    let checked = z0.checked_add(delta);

    if let Some(v) = checked {
        kani::assert(sat == v, "saturating matches checked when representable");
    } else if delta > 0 {
        kani::assert(sat == i128::MAX, "positive overflow rails to MAX");
    } else if delta < 0 {
        kani::assert(sat == i128::MIN, "negative overflow rails to MIN");
    } else {
        kani::assert(false, "overflow with delta == 0 is impossible");
    }
}

// =============================================================================
// Δ-06: Control-Surface Invariant — post-guard arithmetic identity.
// Proves: when freeze_condition fires (dphi set to 0 by guard),
//   delta_i128 == 0 and z_next == z_prev.
// Scope: post-guard arithmetic only. Does NOT prove control-flow correctness
//   (i.e., that the guard fires when dphi > DISCONTINUITY_THRESHOLD).
//   That is covered by code inspection and test_triangle_freeze_tick_keeps_z_constant.
// =============================================================================

/// Δ-06 / Tier-1: Control-Surface post-guard arithmetic identity.
///
/// Assumes the guard has already set dphi = 0.
/// Proves downstream pipeline produces delta_i128 == 0 and z_next == z_prev
/// for all symbolic raw_a, raw_b, z_prev.
#[cfg(kani)]
#[kani::proof]
fn proof_triangle_freeze_invariant() {
    let raw_a: i128 = kani::any();
    let raw_b: i128 = kani::any();
    let z_prev: i128 = kani::any();
    // Post-guard state: the freeze branch has set dphi = 0.
    let dphi_after_guard: u32 = 0;

    let wide_a = i256::I256::from_i128(raw_a);
    let wide_b = i256::I256::from_i128(raw_b);
    let raw_square_diff = wide_a.sub(wide_b);
    let shifted = raw_square_diff.sar(DPW_TRUNCATION_BITS as u32);
    let delta_wide = shifted.mul_u32(dphi_after_guard);
    let delta_i128 = delta_wide.clamp_to_i128();
    let z_next = z_prev.saturating_add(delta_i128);

    kani::assert(delta_i128 == 0, "delta_i128 is zero when frozen");
    kani::assert(z_next == z_prev, "z unchanged when frozen");
}

/// Δ-06 / Tier-1: Control-surface egress identity.
///
/// Assumes the guard has already set dphi = 0 and proves the released
/// triangle egress sample is unchanged for this tick under the default
/// retained release gain routing because z_next == z_prev.
#[cfg(kani)]
#[kani::proof]
fn proof_triangle_freeze_egress_invariant() {
    let raw_a: i128 = kani::any();
    let raw_b: i128 = kani::any();
    let z_prev: i128 = kani::any();
    let dphi_after_guard: u32 = 0;
    let m_q63: u64 = 1u64 << 63;
    let exp: i32 = 0;

    let wide_a = i256::I256::from_i128(raw_a);
    let wide_b = i256::I256::from_i128(raw_b);
    let raw_square_diff = wide_a.sub(wide_b);
    let shifted = raw_square_diff.sar(DPW_TRUNCATION_BITS as u32);
    let delta_wide = shifted.mul_u32(dphi_after_guard);
    let delta_i128 = delta_wide.clamp_to_i128();
    let z_next = z_prev.saturating_add(delta_i128);

    let out_prev = apply_gain(z_prev, m_q63, exp);
    let out_next = apply_gain(z_next, m_q63, exp);

    kani::assert(delta_i128 == 0, "delta_i128 is zero when frozen");
    kani::assert(z_next == z_prev, "z unchanged when frozen");
    kani::assert(
        out_next == out_prev,
        "triangle egress is unchanged when frozen",
    );
}
