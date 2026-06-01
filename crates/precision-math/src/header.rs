//! Artifact Identity Header (v.next)
//!
//! Defines the 128-byte prefix for .rpl artifacts to reconcile
//! execution identity with mathematical determinism.

/// Run-specific artifact identity metadata.
///
/// The authoritative serialized wire format is 128 bytes and is defined by
/// [`OriginHeader::to_bytes`] and [`OriginHeader::from_bytes`]. The Rust struct
/// layout itself is not the serialization contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OriginHeader {
    pub magic: [u8; 4],       // 0x52 0x50 0x4C 0x30 ('RPL0')
    pub schema_version: u32,  // Math schema versioning
    pub git_commit: [u8; 20], // SHA-1 Git hash
    pub run_id: [u8; 16],     // UUID or Timestamp-entropy
    pub timestamp: i64,       // Unix epoch
    pub reserved: [u8; 76],   // Padding to 128 bytes
}

impl OriginHeader {
    pub const SIZE: usize = 128;
    pub const MAGIC: [u8; 4] = *b"RPL0";

    pub fn new(schema: u32, git: [u8; 20], run: [u8; 16], time: i64) -> Self {
        Self {
            magic: Self::MAGIC,
            schema_version: schema,
            git_commit: git,
            run_id: run,
            timestamp: time,
            reserved: [0u8; 76],
        }
    }

    /// Safe serialization to a fixed-size byte array.
    /// Complies with #![forbid(unsafe_code)] by using copy_from_slice.
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        buf[0..4].copy_from_slice(&self.magic);
        buf[4..8].copy_from_slice(&self.schema_version.to_le_bytes());
        buf[8..28].copy_from_slice(&self.git_commit);
        buf[28..44].copy_from_slice(&self.run_id);
        buf[44..52].copy_from_slice(&self.timestamp.to_le_bytes());
        buf[52..Self::SIZE].copy_from_slice(&self.reserved);
        buf
    }

    /// Safe deserialization from a byte slice.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }

        let mut magic = [0u8; 4];
        magic.copy_from_slice(&bytes[0..4]);
        if magic != Self::MAGIC {
            return None;
        }

        let mut git_commit = [0u8; 20];
        git_commit.copy_from_slice(&bytes[8..28]);

        let mut run_id = [0u8; 16];
        run_id.copy_from_slice(&bytes[28..44]);

        let mut reserved = [0u8; 76];
        reserved.copy_from_slice(&bytes[52..Self::SIZE]);

        Some(Self {
            magic,
            schema_version: u32::from_le_bytes(bytes[4..8].try_into().ok()?),
            git_commit,
            run_id,
            timestamp: i64::from_le_bytes(bytes[44..52].try_into().ok()?),
            reserved,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_wire_size() {
        let header = OriginHeader::new(1, [0xAA; 20], [0xBB; 16], 1_714_560_000);

        assert_eq!(header.to_bytes().len(), OriginHeader::SIZE);
    }

    #[test]
    fn test_serialization_round_trip() {
        let git = [0xAA; 20];
        let run = [0xBB; 16];
        let timestamp = 1714560000; // Example Unix epoch

        let header = OriginHeader::new(1, git, run, timestamp);
        let bytes = header.to_bytes();

        // Verify magic bytes and size
        assert_eq!(&bytes[0..4], b"RPL0");
        assert_eq!(bytes.len(), 128);

        // Verify deserialization parity
        let restored = OriginHeader::from_bytes(&bytes).expect("Failed to parse header bytes");
        assert_eq!(header.schema_version, restored.schema_version);
        assert_eq!(header.git_commit, restored.git_commit);
        assert_eq!(header.run_id, restored.run_id);
        assert_eq!(header.timestamp, restored.timestamp);
        assert_eq!(header.reserved, restored.reserved);
    }

    #[test]
    fn test_reserved_bytes_round_trip() {
        let mut bytes = [0u8; OriginHeader::SIZE];
        bytes[0..4].copy_from_slice(b"RPL0");
        bytes[4..8].copy_from_slice(&7u32.to_le_bytes());
        bytes[8..28].copy_from_slice(&[0xAA; 20]);
        bytes[28..44].copy_from_slice(&[0xBB; 16]);
        bytes[44..52].copy_from_slice(&1_714_560_000i64.to_le_bytes());
        bytes[52..OriginHeader::SIZE].fill(0xCC);

        let restored = OriginHeader::from_bytes(&bytes).expect("Failed to parse header bytes");

        assert_eq!(restored.reserved, [0xCC; 76]);
        assert_eq!(restored.to_bytes(), bytes);
    }

    #[test]
    fn test_invalid_magic_rejection() {
        let mut bytes = [0u8; 128];
        bytes[0..4].copy_from_slice(b"BAD!");
        assert!(OriginHeader::from_bytes(&bytes).is_none());
    }
}
