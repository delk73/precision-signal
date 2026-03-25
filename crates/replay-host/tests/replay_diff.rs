use replay_core::artifact::{
    EVENTFRAME0_SIZE, FRAME_SIZE, HEADER0_SIZE, MAGIC, V1_MIN_HEADER_SIZE, V1_OFF_FLAGS,
    V1_OFF_FRAME_COUNT, V1_OFF_FRAME_SIZE, V1_OFF_HEADER_LEN, V1_OFF_RESERVED, V1_OFF_SCHEMA_HASH,
    V1_OFF_SCHEMA_LEN, V1_OFF_VERSION, VERSION0,
};
use replay_host::{diff_artifacts0, parse_frames0, parse_header0, ParseError};
use sha2::{Digest, Sha256};

fn frame_bytes(frame_idx: u32, timer_delta: u32) -> [u8; FRAME_SIZE] {
    let irq_id = (frame_idx as u8).wrapping_add(1);
    let flags = (frame_idx as u8) & 0x0f;
    let rsv = 0u16;
    let input_sample = (frame_idx as i32) * 7 - 4;

    let mut out = [0u8; FRAME_SIZE];
    out[0..4].copy_from_slice(&frame_idx.to_le_bytes());
    out[4] = irq_id;
    out[5] = flags;
    out[6..8].copy_from_slice(&rsv.to_le_bytes());
    out[8..12].copy_from_slice(&timer_delta.to_le_bytes());
    out[12..16].copy_from_slice(&input_sample.to_le_bytes());
    out
}

fn build_artifact_v0(timer_deltas: &[u32]) -> Vec<u8> {
    let frame_count = timer_deltas.len() as u32;
    let mut out = Vec::with_capacity(HEADER0_SIZE + (frame_count as usize) * EVENTFRAME0_SIZE);
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&VERSION0.to_le_bytes());
    out.extend_from_slice(&frame_count.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());

    for (i, timer_delta) in timer_deltas.iter().copied().enumerate() {
        out.extend_from_slice(&frame_bytes(i as u32, timer_delta));
    }

    out
}

fn build_artifact_v1(timer_deltas: &[u32], schema: &[u8]) -> Vec<u8> {
    let frame_count = timer_deltas.len() as u32;
    let mut header = vec![0u8; V1_MIN_HEADER_SIZE];
    header[0..4].copy_from_slice(&MAGIC);
    header[V1_OFF_VERSION..V1_OFF_VERSION + 2].copy_from_slice(&1u16.to_le_bytes());
    header[V1_OFF_HEADER_LEN..V1_OFF_HEADER_LEN + 2]
        .copy_from_slice(&(V1_MIN_HEADER_SIZE as u16).to_le_bytes());
    header[V1_OFF_FRAME_COUNT..V1_OFF_FRAME_COUNT + 4].copy_from_slice(&frame_count.to_le_bytes());
    header[V1_OFF_FRAME_SIZE..V1_OFF_FRAME_SIZE + 2]
        .copy_from_slice(&(FRAME_SIZE as u16).to_le_bytes());
    header[V1_OFF_FLAGS..V1_OFF_FLAGS + 2].copy_from_slice(&0u16.to_le_bytes());
    header[V1_OFF_SCHEMA_LEN..V1_OFF_SCHEMA_LEN + 4]
        .copy_from_slice(&(schema.len() as u32).to_le_bytes());
    header[V1_OFF_SCHEMA_HASH..V1_OFF_SCHEMA_HASH + 32].copy_from_slice(&Sha256::digest(schema));
    header[V1_OFF_RESERVED..V1_OFF_RESERVED + 2].copy_from_slice(&0u16.to_le_bytes());

    let mut out = header;
    out.extend_from_slice(schema);
    for (i, timer_delta) in timer_deltas.iter().copied().enumerate() {
        out.extend_from_slice(&frame_bytes(i as u32, timer_delta));
    }
    out
}

fn bump_timer_delta_at(bytes: &mut [u8], frame_offset: usize, amount: u32) {
    let timer_off = frame_offset + 8;
    let old = u32::from_le_bytes(
        bytes[timer_off..timer_off + 4]
            .try_into()
            .expect("fixed width"),
    );
    let new = old.wrapping_add(amount);
    bytes[timer_off..timer_off + 4].copy_from_slice(&new.to_le_bytes());
}

#[test]
fn identical_v0_artifacts_have_no_divergence() {
    let a = build_artifact_v0(&[10, 20, 30, 40, 50]);
    let b = a.clone();

    let diff = diff_artifacts0(&a, &b).expect("valid artifacts should parse");
    assert_eq!(diff, None);
}

#[test]
fn single_frame_perturbation_reports_its_index_for_v0() {
    let mut b = build_artifact_v0(&[10, 20, 30, 40, 50]);
    let a = b.clone();
    let k = 3usize;
    let frame_start = HEADER0_SIZE + k * EVENTFRAME0_SIZE;
    bump_timer_delta_at(&mut b, frame_start, 1);

    let diff = diff_artifacts0(&a, &b).expect("valid artifacts should parse");
    assert_eq!(diff, Some(k));
}

#[test]
fn malformed_length_is_rejected_by_v0_parser() {
    let mut a = build_artifact_v0(&[10, 20, 30]);
    a.pop();

    let header = parse_header0(&a).expect("header still intact");
    let err = parse_frames0(&a, &header).expect_err("length mismatch must fail");

    assert!(matches!(err, ParseError::LengthMismatch { .. }));
}

#[test]
fn identical_v1_artifacts_have_no_divergence() {
    let a = build_artifact_v1(&[10, 20, 30, 40, 50], b"schema");
    let b = a.clone();

    let diff = diff_artifacts0(&a, &b).expect("valid v1 artifacts should parse");
    assert_eq!(diff, None);
}

#[test]
fn single_frame_perturbation_reports_its_index_for_v1() {
    let mut b = build_artifact_v1(&[10, 20, 30, 40, 50], b"schema");
    let a = b.clone();
    let k = 2usize;
    let frame_start = V1_MIN_HEADER_SIZE + b"schema".len() + k * EVENTFRAME0_SIZE;
    bump_timer_delta_at(&mut b, frame_start, 1);

    let diff = diff_artifacts0(&a, &b).expect("valid v1 artifacts should parse");
    assert_eq!(diff, Some(k));
}
