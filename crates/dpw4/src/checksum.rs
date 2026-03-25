//! Fletcher-32 Checksum Module
//!
//! Position-sensitive integrity validation for `SignalFrameHeader`.
//!
//! # Algorithm
//!
//! Fletcher-32 uses two 16-bit accumulators with modulus 65535 ($2^{16} - 1$):
//! - $S_1$: Simple sum of 16-bit words
//! - $S_2$: "Triangle" sum (running total of $S_1$ values)
//!
//! This modulus distinguishes 0x0000 from 0xFFFF, unlike modulus $2^{16}$.
//!
//! # Endianness
//!
//! 16-bit words are processed as **Little-Endian** (`u16::from_le_bytes`)
//! for cross-platform determinism (x86/ARM compatibility).

/// Fletcher-32 validation error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumError {
    /// Input length must be word-aligned (even bytes).
    OddLength,
}

/// Checked Fletcher-32 checksum with modulus 65535.
///
/// Returns `Err(ChecksumError::OddLength)` if `data` is not word-aligned.
pub fn fletcher32_checked(data: &[u8]) -> Result<u32, ChecksumError> {
    if !data.len().is_multiple_of(2) {
        return Err(ChecksumError::OddLength);
    }
    Ok(fletcher32(data))
}

/// Fletcher-32 checksum with modulus 65535.
///
/// # Arguments
/// * `data` - Byte slice to checksum (must have even length for word alignment)
///
/// # Returns
/// 32-bit checksum: `(S2 << 16) | S1`
///
/// # Panics
/// Panics if `data.len()` is odd (not word-aligned).
pub const fn fletcher32(data: &[u8]) -> u32 {
    assert!(
        data.len().is_multiple_of(2),
        "Data must be word-aligned (even length)"
    );

    const MOD: u32 = 65535; // 2^16 - 1

    let mut s1: u32 = 0;
    let mut s2: u32 = 0;

    let mut i = 0;
    while i < data.len() {
        // Little-Endian word extraction
        let word = (data[i] as u32) | ((data[i + 1] as u32) << 8);

        s1 = (s1 + word) % MOD;
        s2 = (s2 + s1) % MOD;

        i += 2;
    }

    (s2 << 16) | s1
}

/// Size of the metadata portion of the header (excludes checksum field).
pub const HEADER_METADATA_SIZE: usize = crate::HEADER_CHECKSUM_OFFSET;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        // Edge case: empty slice
        let result = fletcher32(&[]);
        assert_eq!(result, 0, "Empty input should return 0");
    }

    #[test]
    fn test_all_zeros() {
        // All-zero header metadata (60 bytes)
        let zeros = [0u8; 60];
        let result = fletcher32(&zeros);
        // Document: With initial accumulators at 0, all-zero input returns 0
        assert_eq!(result, 0, "All-zero input should return 0");
    }

    #[test]
    fn test_known_vector() {
        // "abcd" in bytes: [0x61, 0x62, 0x63, 0x64]
        // Word 0: 0x6261 (LE), Word 1: 0x6463 (LE)
        let data = b"abcd";
        let result = fletcher32(data);

        // Manual calculation:
        // s1 = 0x6261 % 65535 = 25185
        // s2 = 25185 % 65535 = 25185
        // s1 = (25185 + 0x6463) % 65535 = (25185 + 25699) % 65535 = 50884
        // s2 = (25185 + 50884) % 65535 = 76069 % 65535 = 10534
        // Result: (10534 << 16) | 50884 = 0x2926C6C4
        assert_eq!(result, 0x2926C6C4, "Known vector mismatch");
    }

    #[test]
    fn test_checked_odd_length_returns_err() {
        let result = fletcher32_checked(&[0x01, 0x02, 0x03]);
        assert_eq!(result, Err(ChecksumError::OddLength));
    }

    #[test]
    fn test_checked_even_length_matches_known_vector() {
        let data = b"abcd";
        let checked = fletcher32_checked(data).expect("even length should be accepted");
        assert_eq!(checked, 0x2926C6C4, "Known vector mismatch");
    }

    #[test]
    fn test_bitflip_changes_checksum() {
        let mut data = [0x12u8, 0x34, 0x56, 0x78];
        let original = fletcher32(&data);

        // Flip one bit
        data[0] ^= 1;
        let flipped = fletcher32(&data);

        assert_ne!(original, flipped, "Single bit flip must change checksum");
    }

    #[test]
    fn test_word_swap_changes_checksum() {
        // Two different orderings of the same words
        let data_a = [0x12u8, 0x34, 0x56, 0x78];
        let data_b = [0x56u8, 0x78, 0x12, 0x34];

        let checksum_a = fletcher32(&data_a);
        let checksum_b = fletcher32(&data_b);

        assert_ne!(
            checksum_a, checksum_b,
            "Word swap must change checksum (triangle property)"
        );
    }
}
