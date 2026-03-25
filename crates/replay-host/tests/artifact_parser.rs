use replay_core::artifact::{
    EVENTFRAME0_SIZE, FRAME_SIZE, HEADER0_SIZE, MAGIC, V1_MIN_HEADER_SIZE, V1_OFF_BOARD_ID,
    V1_OFF_BUILD_HASH, V1_OFF_CAPTURE_BOUNDARY, V1_OFF_CLOCK_PROFILE, V1_OFF_CONFIG_HASH,
    V1_OFF_FLAGS, V1_OFF_FRAME_COUNT, V1_OFF_FRAME_SIZE, V1_OFF_HEADER_LEN, V1_OFF_RESERVED,
    V1_OFF_SCHEMA_HASH, V1_OFF_SCHEMA_LEN, V1_OFF_VERSION, VERSION0,
};
use replay_host::{
    parse_artifact, parse_artifact_allow_trailing, parse_frames0, parse_header0, ParseError,
    ParsedArtifact,
};
use sha2::{Digest, Sha256};

fn frame_bytes(frame_idx: u32) -> [u8; FRAME_SIZE] {
    let irq_id = ((frame_idx as u8) & 0x0f).wrapping_add(1);
    let flags = (frame_idx as u8) ^ 0x22;
    let rsv = (frame_idx as u16).wrapping_mul(3);
    let timer_delta = 1_000u32.wrapping_add(frame_idx.wrapping_mul(17));
    let input_sample = (frame_idx as i32).wrapping_mul(13) - 9;

    let mut out = [0u8; FRAME_SIZE];
    out[0..4].copy_from_slice(&frame_idx.to_le_bytes());
    out[4] = irq_id;
    out[5] = flags;
    out[6..8].copy_from_slice(&rsv.to_le_bytes());
    out[8..12].copy_from_slice(&timer_delta.to_le_bytes());
    out[12..16].copy_from_slice(&input_sample.to_le_bytes());
    out
}

fn build_valid_artifact_v0(frame_count: u32) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEADER0_SIZE + (frame_count as usize) * EVENTFRAME0_SIZE);
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&VERSION0.to_le_bytes());
    out.extend_from_slice(&frame_count.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    for idx in 0..frame_count {
        out.extend_from_slice(&frame_bytes(idx));
    }
    out
}

fn fill_pattern(buf: &mut [u8], seed: u8) {
    for (idx, byte) in buf.iter_mut().enumerate() {
        *byte = seed.wrapping_add((idx as u8).wrapping_mul(17));
    }
}

fn build_valid_artifact_v1(frame_count: u32, schema: &[u8], header_ext_len: usize) -> Vec<u8> {
    let header_len = V1_MIN_HEADER_SIZE + header_ext_len;
    let mut header = vec![0u8; header_len];
    header[0..4].copy_from_slice(&MAGIC);
    header[V1_OFF_VERSION..V1_OFF_VERSION + 2].copy_from_slice(&1u16.to_le_bytes());
    header[V1_OFF_HEADER_LEN..V1_OFF_HEADER_LEN + 2]
        .copy_from_slice(&(header_len as u16).to_le_bytes());
    header[V1_OFF_FRAME_COUNT..V1_OFF_FRAME_COUNT + 4].copy_from_slice(&frame_count.to_le_bytes());
    header[V1_OFF_FRAME_SIZE..V1_OFF_FRAME_SIZE + 2]
        .copy_from_slice(&(FRAME_SIZE as u16).to_le_bytes());
    header[V1_OFF_FLAGS..V1_OFF_FLAGS + 2].copy_from_slice(&0u16.to_le_bytes());
    header[V1_OFF_SCHEMA_LEN..V1_OFF_SCHEMA_LEN + 4]
        .copy_from_slice(&(schema.len() as u32).to_le_bytes());

    let schema_hash = Sha256::digest(schema);
    header[V1_OFF_SCHEMA_HASH..V1_OFF_SCHEMA_HASH + 32].copy_from_slice(&schema_hash);

    fill_pattern(&mut header[V1_OFF_BUILD_HASH..V1_OFF_BUILD_HASH + 32], 0x11);
    fill_pattern(
        &mut header[V1_OFF_CONFIG_HASH..V1_OFF_CONFIG_HASH + 32],
        0x22,
    );
    fill_pattern(
        &mut header[V1_OFF_CLOCK_PROFILE..V1_OFF_CLOCK_PROFILE + 16],
        0x33,
    );
    fill_pattern(&mut header[V1_OFF_BOARD_ID..V1_OFF_BOARD_ID + 16], 0x44);
    header[V1_OFF_CAPTURE_BOUNDARY..V1_OFF_CAPTURE_BOUNDARY + 2]
        .copy_from_slice(&7u16.to_le_bytes());
    header[V1_OFF_RESERVED..V1_OFF_RESERVED + 2].copy_from_slice(&0u16.to_le_bytes());

    if header_ext_len > 0 {
        fill_pattern(&mut header[V1_MIN_HEADER_SIZE..header_len], 0x55);
    }

    let mut out = header;
    out.extend_from_slice(schema);
    for idx in 0..frame_count {
        out.extend_from_slice(&frame_bytes(idx));
    }
    out
}

#[test]
fn parse_valid_header_and_frames_v0() {
    let bytes = build_valid_artifact_v0(3);
    let header = parse_header0(&bytes).expect("valid header should parse");
    let frames = parse_frames0(&bytes, &header).expect("valid frames should parse");

    assert_eq!(header.magic, MAGIC);
    assert_eq!(header.version, VERSION0);
    assert_eq!(header.frame_count, 3);
    assert_eq!(frames.len(), 3);
    assert_eq!(frames[0].frame_idx, 0);
    assert_eq!(frames[2].frame_idx, 2);
}

#[test]
fn parse_rejects_bad_magic_v0() {
    let mut bytes = build_valid_artifact_v0(1);
    bytes[0] = b'X';

    let err = parse_header0(&bytes).expect_err("bad magic must fail");
    assert!(matches!(err, ParseError::BadMagic { .. }));
}

#[test]
fn parse_rejects_bad_length_v0() {
    let mut bytes = build_valid_artifact_v0(3);
    bytes.pop();

    let header = parse_header0(&bytes).expect("header bytes still present");
    let err = parse_frames0(&bytes, &header).expect_err("length mismatch must fail");
    assert!(matches!(err, ParseError::LengthMismatch { .. }));
}

#[test]
fn parse_rejects_v0_nonzero_reserved_in_generic_path() {
    let mut bytes = build_valid_artifact_v0(1);
    bytes[12..16].copy_from_slice(&7u32.to_le_bytes());

    let err = parse_artifact(&bytes).expect_err("reserved must be rejected");
    assert!(matches!(err, ParseError::ReservedNonZeroV0 { actual: 7 }));
}

#[test]
fn parse_accepts_v1_zero_frames_and_empty_schema() {
    let bytes = build_valid_artifact_v1(0, b"", 0);

    let parsed = parse_artifact(&bytes).expect("valid v1 artifact should parse");
    match parsed {
        ParsedArtifact::V1(parsed) => {
            assert_eq!(parsed.header.version, 1);
            assert_eq!(parsed.header.header_len as usize, V1_MIN_HEADER_SIZE);
            assert_eq!(parsed.header.frame_count, 0);
            assert_eq!(parsed.header.schema_len, 0);
            assert!(parsed.schema_block.is_empty());
            assert!(parsed.frames.is_empty());
            assert_eq!(parsed.frames_offset, V1_MIN_HEADER_SIZE);
            assert_eq!(parsed.canonical_len, V1_MIN_HEADER_SIZE);
        }
        ParsedArtifact::V0(_) => panic!("expected v1"),
    }
}

#[test]
fn parse_accepts_v1_extended_header() {
    let schema = b"schema-v1";
    let bytes = build_valid_artifact_v1(3, schema, 16);

    let parsed = parse_artifact(&bytes).expect("extended header should parse");
    match parsed {
        ParsedArtifact::V1(parsed) => {
            assert_eq!(parsed.header.header_len as usize, V1_MIN_HEADER_SIZE + 16);
            assert_eq!(parsed.schema_block, schema);
            assert_eq!(parsed.frames_offset, V1_MIN_HEADER_SIZE + 16 + schema.len());
            assert_eq!(parsed.frames.len(), 3 * FRAME_SIZE);
        }
        ParsedArtifact::V0(_) => panic!("expected v1"),
    }
}

#[test]
fn parse_rejects_v1_dispatch_boundary_with_unsupported_u16_version() {
    let mut bytes = build_valid_artifact_v1(1, b"", 0);
    bytes[V1_OFF_VERSION..V1_OFF_VERSION + 2].copy_from_slice(&2u16.to_le_bytes());

    let err = parse_artifact(&bytes).expect_err("unsupported version must fail");
    assert!(matches!(err, ParseError::BadVersion { actual: 2 }));
}

#[test]
fn parse_rejects_v1_header_len_too_small() {
    let mut bytes = build_valid_artifact_v1(1, b"", 0);
    bytes[V1_OFF_HEADER_LEN..V1_OFF_HEADER_LEN + 2]
        .copy_from_slice(&((V1_MIN_HEADER_SIZE - 1) as u16).to_le_bytes());

    let err = parse_artifact(&bytes).expect_err("small header_len must fail");
    assert!(matches!(err, ParseError::HeaderLenTooSmall { .. }));
}

#[test]
fn parse_rejects_v1_header_len_above_file_len() {
    let mut bytes = build_valid_artifact_v1(1, b"", 0);
    bytes[V1_OFF_HEADER_LEN..V1_OFF_HEADER_LEN + 2].copy_from_slice(&0x0200u16.to_le_bytes());

    let err = parse_artifact(&bytes).expect_err("oversized header_len must fail");
    assert!(matches!(err, ParseError::HeaderLenExceedsFileLen { .. }));
}

#[test]
fn parse_rejects_v1_invalid_frame_size() {
    let mut bytes = build_valid_artifact_v1(1, b"", 0);
    bytes[V1_OFF_FRAME_SIZE..V1_OFF_FRAME_SIZE + 2].copy_from_slice(&8u16.to_le_bytes());

    let err = parse_artifact(&bytes).expect_err("invalid frame_size must fail");
    assert!(matches!(err, ParseError::InvalidFrameSize { .. }));
}

#[test]
fn parse_rejects_v1_nonzero_flags_and_reserved() {
    let mut bytes = build_valid_artifact_v1(1, b"", 0);
    bytes[V1_OFF_FLAGS..V1_OFF_FLAGS + 2].copy_from_slice(&1u16.to_le_bytes());
    let err = parse_artifact(&bytes).expect_err("nonzero flags must fail");
    assert!(matches!(err, ParseError::UnsupportedFlagsV1 { actual: 1 }));

    let mut bytes = build_valid_artifact_v1(1, b"", 0);
    bytes[V1_OFF_RESERVED..V1_OFF_RESERVED + 2].copy_from_slice(&1u16.to_le_bytes());
    let err = parse_artifact(&bytes).expect_err("nonzero reserved must fail");
    assert!(matches!(err, ParseError::ReservedNonZeroV1 { actual: 1 }));
}

#[test]
fn parse_rejects_v1_schema_hash_mismatch() {
    let mut bytes = build_valid_artifact_v1(1, b"abcd", 0);
    bytes[V1_OFF_SCHEMA_HASH] ^= 0x01;

    let err = parse_artifact(&bytes).expect_err("schema hash mismatch must fail");
    assert!(matches!(err, ParseError::SchemaHashMismatch));
}

#[test]
fn parse_rejects_v1_schema_out_of_bounds() {
    let mut bytes = build_valid_artifact_v1(1, b"", 0);
    bytes[V1_OFF_SCHEMA_LEN..V1_OFF_SCHEMA_LEN + 4].copy_from_slice(&0xffff_fff0u32.to_le_bytes());

    let err = parse_artifact(&bytes).expect_err("schema overflow must fail");
    assert!(matches!(
        err,
        ParseError::LengthOverflow { .. } | ParseError::SchemaOutOfBounds { .. }
    ));
}

#[test]
fn parse_rejects_v1_length_mismatch_cases() {
    let mut bytes = build_valid_artifact_v1(2, b"abc", 0);
    bytes.truncate(bytes.len() - 5);

    let err = parse_artifact(&bytes).expect_err("partial frame region must fail");
    assert!(matches!(err, ParseError::LengthMismatch { .. }));

    let bytes = build_valid_artifact_v1(1, b"", 0);
    let err = parse_artifact(&[bytes.as_slice(), b"\xAA\xBB"].concat())
        .expect_err("trailing bytes must fail in strict mode");
    assert!(matches!(err, ParseError::LengthMismatch { .. }));

    let with_trailing = [bytes.as_slice(), b"\xAA\xBB"].concat();
    let parsed = parse_artifact_allow_trailing(&with_trailing)
        .expect("trailing bytes may be ignored in canonical hash mode");
    match parsed {
        ParsedArtifact::V1(parsed) => assert_eq!(parsed.canonical_len, bytes.len()),
        ParsedArtifact::V0(_) => panic!("expected v1"),
    }
}

#[test]
fn parse_accepts_metadata_only_mutations() {
    for offset in [
        V1_OFF_BUILD_HASH,
        V1_OFF_CONFIG_HASH + 1,
        V1_OFF_BOARD_ID + 2,
        V1_OFF_CLOCK_PROFILE + 3,
        V1_OFF_CAPTURE_BOUNDARY,
    ] {
        let mut bytes = build_valid_artifact_v1(2, b"schema", 0);
        bytes[offset] ^= 0x80;
        parse_artifact(&bytes).expect("metadata mutation must remain valid");
    }
}
