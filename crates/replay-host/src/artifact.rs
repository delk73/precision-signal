use replay_core::artifact::{
    EventFrame0, Header0, Header1, BOARD_ID_SIZE, EVENTFRAME0_SIZE, FRAME_SIZE, HEADER0_SIZE,
    MAGIC, SCHEMA_HASH_SIZE, V1_MIN_HEADER_SIZE, V1_OFF_BOARD_ID, V1_OFF_BUILD_HASH,
    V1_OFF_CAPTURE_BOUNDARY, V1_OFF_CLOCK_PROFILE, V1_OFF_CONFIG_HASH, V1_OFF_FLAGS,
    V1_OFF_FRAME_COUNT, V1_OFF_FRAME_SIZE, V1_OFF_HEADER_LEN, V1_OFF_RESERVED, V1_OFF_SCHEMA_HASH,
    V1_OFF_SCHEMA_LEN, V1_OFF_VERSION, VERSION0, VERSION1,
};
use sha2::{Digest, Sha256};

// Current parser-policy parity with the Python tooling requires these bytes to
// remain zero, even though the v1 artifact contract treats them as reserved or
// opaque metadata rather than schema-bearing fields.
const REQUIRED_RESERVED_V0: u32 = 0;
const REQUIRED_RESERVED_V1: u16 = 0;
const REQUIRED_FLAGS_V1: u16 = 0;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ParseError {
    InputTooShort {
        expected_at_least: usize,
        actual: usize,
    },
    BadMagic {
        actual: [u8; 4],
    },
    BadVersion {
        actual: u32,
    },
    ReservedNonZeroV0 {
        actual: u32,
    },
    UnsupportedFlagsV1 {
        actual: u16,
    },
    ReservedNonZeroV1 {
        actual: u16,
    },
    HeaderLenTooSmall {
        actual: u16,
        minimum: usize,
    },
    HeaderLenExceedsFileLen {
        header_len: u16,
        file_len: usize,
    },
    InvalidFrameSize {
        actual: u16,
        expected: usize,
    },
    FrameCountTooLarge {
        frame_count: u32,
    },
    LengthOverflow {
        field: &'static str,
    },
    SchemaOutOfBounds {
        schema_end: usize,
        file_len: usize,
    },
    LengthMismatch {
        expected: usize,
        actual: usize,
    },
    SchemaHashMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParsedArtifact0<'a> {
    pub header: Header0,
    pub frames: &'a [u8],
    pub canonical_len: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParsedArtifact1<'a> {
    pub header: Header1,
    pub header_bytes: &'a [u8],
    pub schema_block: &'a [u8],
    pub frames: &'a [u8],
    pub frames_offset: usize,
    pub canonical_len: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParsedArtifact<'a> {
    V0(ParsedArtifact0<'a>),
    V1(ParsedArtifact1<'a>),
}

impl<'a> ParsedArtifact<'a> {
    pub fn frame_bytes(self) -> &'a [u8] {
        match self {
            Self::V0(parsed) => parsed.frames,
            Self::V1(parsed) => parsed.frames,
        }
    }
}

pub fn parse_artifact(bytes: &[u8]) -> Result<ParsedArtifact<'_>, ParseError> {
    parse_artifact_with_mode(bytes, false)
}

pub fn parse_artifact_allow_trailing(bytes: &[u8]) -> Result<ParsedArtifact<'_>, ParseError> {
    parse_artifact_with_mode(bytes, true)
}

pub fn parse_header0(bytes: &[u8]) -> Result<Header0, ParseError> {
    if bytes.len() < HEADER0_SIZE {
        return Err(ParseError::InputTooShort {
            expected_at_least: HEADER0_SIZE,
            actual: bytes.len(),
        });
    }

    let magic = read_array4(bytes, 0);
    if magic != MAGIC {
        return Err(ParseError::BadMagic { actual: magic });
    }

    let version = read_u32(bytes, 4);
    if version != VERSION0 {
        return Err(ParseError::BadVersion { actual: version });
    }

    Ok(Header0 {
        magic,
        version,
        frame_count: read_u32(bytes, 8),
        reserved: read_u32(bytes, 12),
    })
}

pub fn parse_frames0(bytes: &[u8], header: &Header0) -> Result<Vec<EventFrame0>, ParseError> {
    let frame_count =
        usize::try_from(header.frame_count).map_err(|_| ParseError::FrameCountTooLarge {
            frame_count: header.frame_count,
        })?;
    let payload_len = checked_mul(frame_count, EVENTFRAME0_SIZE, "frame region length")?;
    let expected_len = checked_add(HEADER0_SIZE, payload_len, "artifact length")?;

    if bytes.len() != expected_len {
        return Err(ParseError::LengthMismatch {
            expected: expected_len,
            actual: bytes.len(),
        });
    }

    decode_event_frames0(&bytes[HEADER0_SIZE..], header.frame_count)
}

/// Decodes the artifact frame region using the legacy `EventFrame0`
/// interpretation.
///
/// This is explicit replay semantics, not generic container parsing:
/// - v0 artifacts decode under the original fixed layout
/// - v1 artifacts must first pass structural/container validation, then their
///   16-byte frames are interpreted as legacy `EventFrame0` payloads
///
/// This does not imply schema-aware replay support for arbitrary future v1
/// frame semantics.
pub fn parse_replay_frames_legacy0(bytes: &[u8]) -> Result<Vec<EventFrame0>, ParseError> {
    let parsed = parse_artifact(bytes)?;
    let frame_count = match parsed {
        ParsedArtifact::V0(parsed) => parsed.header.frame_count,
        ParsedArtifact::V1(parsed) => parsed.header.frame_count,
    };
    decode_event_frames0(parsed.frame_bytes(), frame_count)
}

pub fn debug_dump_first_frames(bytes: &[u8], n: usize) -> Result<(), ParseError> {
    let header = parse_header0(bytes)?;
    let frames = parse_frames0(bytes, &header)?;

    println!(
        "header magic={:?} version={} frame_count={} reserved={}",
        header.magic, header.version, header.frame_count, header.reserved
    );

    for frame in frames.iter().take(n) {
        println!(
            "frame idx={} irq_id={} flags={} rsv={} timer_delta={} input_sample={}",
            frame.frame_idx,
            frame.irq_id,
            frame.flags,
            frame.rsv,
            frame.timer_delta,
            frame.input_sample
        );
    }

    Ok(())
}

fn parse_artifact_with_mode(
    bytes: &[u8],
    allow_trailing: bool,
) -> Result<ParsedArtifact<'_>, ParseError> {
    if bytes.len() < HEADER0_SIZE {
        return Err(ParseError::InputTooShort {
            expected_at_least: HEADER0_SIZE,
            actual: bytes.len(),
        });
    }

    let magic = read_array4(bytes, 0);
    if magic != MAGIC {
        return Err(ParseError::BadMagic { actual: magic });
    }

    let version32 = read_u32(bytes, 4);
    if version32 == VERSION0 {
        return parse_v0(bytes, allow_trailing);
    }

    parse_v1(bytes, allow_trailing)
}

fn parse_v0(bytes: &[u8], allow_trailing: bool) -> Result<ParsedArtifact<'_>, ParseError> {
    if bytes.len() < HEADER0_SIZE {
        return Err(ParseError::InputTooShort {
            expected_at_least: HEADER0_SIZE,
            actual: bytes.len(),
        });
    }

    let header = Header0 {
        magic: read_array4(bytes, 0),
        version: read_u32(bytes, 4),
        frame_count: read_u32(bytes, 8),
        reserved: read_u32(bytes, 12),
    };

    if header.magic != MAGIC {
        return Err(ParseError::BadMagic {
            actual: header.magic,
        });
    }
    if header.version != VERSION0 {
        return Err(ParseError::BadVersion {
            actual: header.version,
        });
    }
    if header.reserved != REQUIRED_RESERVED_V0 {
        return Err(ParseError::ReservedNonZeroV0 {
            actual: header.reserved,
        });
    }

    let frame_count =
        usize::try_from(header.frame_count).map_err(|_| ParseError::FrameCountTooLarge {
            frame_count: header.frame_count,
        })?;
    let frames_len = checked_mul(frame_count, FRAME_SIZE, "frame region length")?;
    let expected_len = checked_add(HEADER0_SIZE, frames_len, "artifact length")?;

    if bytes.len() < expected_len {
        return Err(ParseError::LengthMismatch {
            expected: expected_len,
            actual: bytes.len(),
        });
    }
    if !allow_trailing && bytes.len() != expected_len {
        return Err(ParseError::LengthMismatch {
            expected: expected_len,
            actual: bytes.len(),
        });
    }

    Ok(ParsedArtifact::V0(ParsedArtifact0 {
        header,
        frames: &bytes[HEADER0_SIZE..expected_len],
        canonical_len: expected_len,
    }))
}

fn parse_v1(bytes: &[u8], allow_trailing: bool) -> Result<ParsedArtifact<'_>, ParseError> {
    if bytes.len() < V1_MIN_HEADER_SIZE {
        return Err(ParseError::InputTooShort {
            expected_at_least: V1_MIN_HEADER_SIZE,
            actual: bytes.len(),
        });
    }

    let version = read_u16(bytes, V1_OFF_VERSION);
    if version != VERSION1 {
        return Err(ParseError::BadVersion {
            actual: u32::from(version),
        });
    }

    let header_len = read_u16(bytes, V1_OFF_HEADER_LEN);
    if usize::from(header_len) < V1_MIN_HEADER_SIZE {
        return Err(ParseError::HeaderLenTooSmall {
            actual: header_len,
            minimum: V1_MIN_HEADER_SIZE,
        });
    }
    if usize::from(header_len) > bytes.len() {
        return Err(ParseError::HeaderLenExceedsFileLen {
            header_len,
            file_len: bytes.len(),
        });
    }

    let frame_size = read_u16(bytes, V1_OFF_FRAME_SIZE);
    if usize::from(frame_size) != FRAME_SIZE {
        return Err(ParseError::InvalidFrameSize {
            actual: frame_size,
            expected: FRAME_SIZE,
        });
    }

    let flags = read_u16(bytes, V1_OFF_FLAGS);
    if flags != REQUIRED_FLAGS_V1 {
        return Err(ParseError::UnsupportedFlagsV1 { actual: flags });
    }

    let reserved = read_u16(bytes, V1_OFF_RESERVED);
    if reserved != REQUIRED_RESERVED_V1 {
        return Err(ParseError::ReservedNonZeroV1 { actual: reserved });
    }

    let schema_len = read_u32(bytes, V1_OFF_SCHEMA_LEN);
    let frame_count = read_u32(bytes, V1_OFF_FRAME_COUNT);
    let schema_offset = usize::from(header_len);
    let schema_len_usize = usize::try_from(schema_len).map_err(|_| ParseError::LengthOverflow {
        field: "schema end",
    })?;
    let frame_count_usize =
        usize::try_from(frame_count).map_err(|_| ParseError::FrameCountTooLarge { frame_count })?;
    let schema_end = checked_add(schema_offset, schema_len_usize, "schema end")?;
    if schema_end > bytes.len() {
        return Err(ParseError::SchemaOutOfBounds {
            schema_end,
            file_len: bytes.len(),
        });
    }

    let frames_len = checked_mul(frame_count_usize, FRAME_SIZE, "frame region length")?;
    let expected_len = checked_add(schema_end, frames_len, "artifact length")?;
    if bytes.len() < expected_len {
        return Err(ParseError::LengthMismatch {
            expected: expected_len,
            actual: bytes.len(),
        });
    }
    if !allow_trailing && bytes.len() != expected_len {
        return Err(ParseError::LengthMismatch {
            expected: expected_len,
            actual: bytes.len(),
        });
    }

    let schema_block = &bytes[schema_offset..schema_end];
    let computed_schema_hash = Sha256::digest(schema_block);
    if computed_schema_hash.as_slice()
        != &bytes[V1_OFF_SCHEMA_HASH..V1_OFF_SCHEMA_HASH + SCHEMA_HASH_SIZE]
    {
        return Err(ParseError::SchemaHashMismatch);
    }

    let header = Header1 {
        magic: read_array4(bytes, 0),
        version,
        header_len,
        frame_count,
        frame_size,
        flags,
        schema_len,
        schema_hash: read_array32(bytes, V1_OFF_SCHEMA_HASH),
        build_hash: read_array32(bytes, V1_OFF_BUILD_HASH),
        config_hash: read_array32(bytes, V1_OFF_CONFIG_HASH),
        board_id: read_array16(bytes, V1_OFF_BOARD_ID),
        clock_profile: read_array16(bytes, V1_OFF_CLOCK_PROFILE),
        capture_boundary: read_u16(bytes, V1_OFF_CAPTURE_BOUNDARY),
        reserved,
    };

    Ok(ParsedArtifact::V1(ParsedArtifact1 {
        header,
        header_bytes: &bytes[..schema_offset],
        schema_block,
        frames: &bytes[schema_end..expected_len],
        frames_offset: schema_end,
        canonical_len: expected_len,
    }))
}

fn decode_event_frames0(bytes: &[u8], frame_count: u32) -> Result<Vec<EventFrame0>, ParseError> {
    let frame_count =
        usize::try_from(frame_count).map_err(|_| ParseError::FrameCountTooLarge { frame_count })?;
    let expected_len = checked_mul(frame_count, EVENTFRAME0_SIZE, "frame region length")?;
    if bytes.len() != expected_len {
        return Err(ParseError::LengthMismatch {
            expected: expected_len,
            actual: bytes.len(),
        });
    }

    let mut parsed = Vec::with_capacity(frame_count);
    let mut chunks = bytes.chunks_exact(EVENTFRAME0_SIZE);
    for chunk in &mut chunks {
        parsed.push(decode_event_frame0(chunk));
    }
    debug_assert!(chunks.remainder().is_empty());
    Ok(parsed)
}

fn decode_event_frame0(chunk: &[u8]) -> EventFrame0 {
    EventFrame0 {
        frame_idx: read_u32(chunk, 0),
        irq_id: chunk[4],
        flags: chunk[5],
        rsv: read_u16(chunk, 6),
        timer_delta: read_u32(chunk, 8),
        input_sample: i32::from_le_bytes([chunk[12], chunk[13], chunk[14], chunk[15]]),
    }
}

fn checked_add(lhs: usize, rhs: usize, field: &'static str) -> Result<usize, ParseError> {
    lhs.checked_add(rhs)
        .ok_or(ParseError::LengthOverflow { field })
}

fn checked_mul(lhs: usize, rhs: usize, field: &'static str) -> Result<usize, ParseError> {
    lhs.checked_mul(rhs)
        .ok_or(ParseError::LengthOverflow { field })
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn read_array4(bytes: &[u8], offset: usize) -> [u8; 4] {
    [
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]
}

fn read_array16(bytes: &[u8], offset: usize) -> [u8; 16] {
    let mut out = [0u8; BOARD_ID_SIZE];
    out.copy_from_slice(&bytes[offset..offset + BOARD_ID_SIZE]);
    out
}

fn read_array32(bytes: &[u8], offset: usize) -> [u8; 32] {
    let mut out = [0u8; SCHEMA_HASH_SIZE];
    out.copy_from_slice(&bytes[offset..offset + SCHEMA_HASH_SIZE]);
    out
}
