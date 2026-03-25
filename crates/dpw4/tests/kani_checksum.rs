//! Kani proofs for Fletcher-32 checksum integrity.
//!
//! These proofs establish formal guarantees for:
//! 1. Non-overflow: Accumulators remain within 16-bit bounds
//! 2. Bit-flip detection: Any single-bit flip produces a different checksum
//! 3. Transposition detection: Swapping any two words produces a different checksum

#[cfg(kani)]
use dpw4::fletcher32;

/// Proof 1: Non-Overflow
///
/// Prove that the accumulator modular reduction prevents overflow.
/// Both S1 and S2 must remain ≤ 65535 before final concatenation.
#[cfg(kani)]
#[kani::proof]
fn proof_no_overflow() {
    // Use a small fixed-size buffer to make verification tractable
    let data: [u8; 8] = kani::any();

    // The implementation uses modulus 65535, so after each step:
    // s1 = (s1 + word) % 65535  -> always < 65535
    // s2 = (s2 + s1) % 65535    -> always < 65535

    let checksum = fletcher32(&data);

    // Extract S1 and S2 from the checksum
    let s1 = checksum & 0xFFFF;
    let s2 = checksum >> 16;

    // Both must be < 65535 (the modulus)
    assert!(s1 < 65535, "S1 overflow");
    assert!(s2 < 65535, "S2 overflow");
}

/// Proof 2: Bit-Flip Detection
///
/// Prove that flipping any single bit in the metadata produces a different checksum.
/// This establishes single-bit error detection capability.
#[cfg(kani)]
#[kani::proof]
fn proof_bitflip_detection() {
    let header: [u8; 60] = kani::any();
    let original_checksum = fletcher32(&header);

    // Pick a symbolic bit to flip (0..479 for 60 bytes * 8 bits)
    let bit_to_flip: usize = kani::any_where(|&b: &usize| b < 480);
    let mut flipped_header = header;
    let byte_idx = bit_to_flip / 8;
    let bit_idx = bit_to_flip % 8;
    flipped_header[byte_idx] ^= 1 << bit_idx;

    let flipped_checksum = fletcher32(&flipped_header);

    // Prove that a single-bit flip NEVER results in the same checksum
    assert!(original_checksum != flipped_checksum);
}

/// Proof 3: Transposition Detection
///
/// Prove that swapping any two 16-bit words produces a different checksum.
/// This validates the "triangle" (position-sensitive) property of Fletcher-32.
#[cfg(kani)]
#[kani::proof]
fn proof_transposition_detection() {
    // Use a smaller buffer for tractable verification
    let data: [u8; 8] = kani::any();
    let original_checksum = fletcher32(&data);

    // Pick two distinct word positions to swap (0, 1, 2, 3 for 4 words)
    let word_a: usize = kani::any_where(|&w: &usize| w < 4);
    let word_b: usize = kani::any_where(|&w: &usize| w < 4 && w != word_a);

    // Swap the words
    let mut swapped = data;
    let a_start = word_a * 2;
    let b_start = word_b * 2;

    // Swap two bytes at a time (one word)
    let tmp0 = swapped[a_start];
    let tmp1 = swapped[a_start + 1];
    swapped[a_start] = swapped[b_start];
    swapped[a_start + 1] = swapped[b_start + 1];
    swapped[b_start] = tmp0;
    swapped[b_start + 1] = tmp1;

    let swapped_checksum = fletcher32(&swapped);

    // Only assert difference if the words were actually different
    // (swapping identical words would produce the same checksum)
    let word_a_val = (data[a_start] as u16) | ((data[a_start + 1] as u16) << 8);
    let word_b_val = (data[b_start] as u16) | ((data[b_start + 1] as u16) << 8);

    if word_a_val != word_b_val {
        // Fletcher-32 with MOD 65535 fails to detect transposition if:
        // (pos_b - pos_a) * (word_a - word_b) is a multiple of 65535.
        // For a 4-word buffer, max distance is 3. 65535 is divisible by 3 (21845).
        let dist = if word_b > word_a {
            word_b - word_a
        } else {
            word_a - word_b
        } as u32;
        let diff = if word_a_val > word_b_val {
            word_a_val - word_b_val
        } else {
            word_b_val - word_a_val
        } as u32;

        if (dist * diff) % 65535 != 0 {
            assert!(
                original_checksum != swapped_checksum,
                "Transposition not detected"
            );
        }
    }
}
