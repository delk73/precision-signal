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

#[cfg(any(test, feature = "verification-runtime", feature = "cli"))]
use sha2::{Digest, Sha256};
#[cfg(any(test, feature = "verification-runtime", feature = "cli"))]
use std::io::{self, Read, Seek, SeekFrom};

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

/// Compute a SHA-256 digest for a stream after skipping an initial prefix.
///
/// On success, the reader is consumed from `offset` to EOF and will be left at
/// the end of the stream.
#[cfg(any(test, feature = "verification-runtime", feature = "cli"))]
pub fn compute_stream_hash<R: Read + Seek>(reader: &mut R, offset: u64) -> io::Result<[u8; 32]> {
    let stream_len = reader.seek(SeekFrom::End(0))?;
    if offset > stream_len {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            std::format!("hash offset {offset} exceeds stream length {stream_len}"),
        ));
    }

    reader.seek(SeekFrom::Start(offset))?;

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().into())
}

/// Compute the canonical SHA-256 hash of an artifact payload.
///
/// The first [`crate::header::OriginHeader::SIZE`] bytes are always skipped so
/// run-specific identity metadata does not affect the mathematical payload hash.
#[cfg(any(test, feature = "verification-runtime", feature = "cli"))]
pub fn compute_payload_hash<R: Read + Seek>(reader: &mut R) -> io::Result<[u8; 32]> {
    compute_stream_hash(reader, crate::header::OriginHeader::SIZE as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OriginHeader;
    use sha2::{Digest, Sha256};
    use std::io::Cursor;
    use std::vec;

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

    #[test]
    fn test_stream_hash_with_custom_offset() {
        let payload = b"payload-only";
        let mut artifact = vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE];
        artifact.extend_from_slice(payload);

        let digest = compute_stream_hash(&mut Cursor::new(artifact), 5)
            .expect("stream hash should compute with custom offset");
        let expected: [u8; 32] = Sha256::digest(payload).into();

        assert_eq!(
            digest, expected,
            "custom offset must hash only the tail payload"
        );
    }

    #[test]
    fn test_stream_hash_rejects_offset_past_eof() {
        let mut reader = Cursor::new(b"abc".to_vec());

        let error =
            compute_stream_hash(&mut reader, 4).expect_err("offsets past EOF must be rejected");

        assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn test_payload_hash_determinism() {
        let payload = b"same mathematical payload across unique headers";

        let header_a = OriginHeader::new(1, [0x11; 20], [0x22; 16], 1_714_560_000);
        let header_b = OriginHeader::new(1, [0x33; 20], [0x44; 16], 1_714_560_123);

        let mut artifact_a = header_a.to_bytes().to_vec();
        artifact_a.extend_from_slice(payload);

        let mut artifact_b = header_b.to_bytes().to_vec();
        artifact_b.extend_from_slice(payload);

        let digest_a = compute_payload_hash(&mut Cursor::new(artifact_a))
            .expect("payload hash should compute for artifact A");
        let digest_b = compute_payload_hash(&mut Cursor::new(artifact_b))
            .expect("payload hash should compute for artifact B");

        assert_eq!(
            digest_a, digest_b,
            "origin header must be masked from payload hash"
        );
    }
}
