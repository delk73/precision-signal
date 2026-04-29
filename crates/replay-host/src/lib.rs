#![forbid(unsafe_code)]

pub mod artifact;
pub mod replay;

use replay_core::artifact::{
    encode_event_frame0_le, encode_header1_le, EventFrame0, Header1, FRAME_COUNT, FRAME_SIZE,
    HEADER1_SIZE, MAGIC, VERSION1,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

pub use artifact::{
    debug_dump_first_frames, parse_artifact, parse_artifact_allow_trailing, parse_frames0,
    parse_header0, parse_replay_frames_legacy0, ParseError, ParsedArtifact, ParsedArtifact0,
    ParsedArtifact1,
};
pub use replay::{
    diff_artifacts0, first_divergence0, hash_state0, replay_hashes0, step0, SutState0,
};

const IMPORT_IRQ_ID: u8 = 0x02;
const IMPORT_TIMER_DELTA: u32 = 1000;
const IMPORT_CAPTURE_BOUNDARY_ISR: u16 = 0;
const EMPTY_SCHEMA: &[u8] = b"";
const BUILD_HASH_INPUT: &[u8] = b"replay-host:import-interval-csv:v1";
const CONFIG_HASH_INPUT: &[u8] = b"source=index,interval_us;pad=zero;timer_delta=1000;irq=0x02";
const BOARD_ID: [u8; 16] = *b"interval-csv-run";
const CLOCK_PROFILE: [u8; 16] = *b"offline-fixed-v1";
const INTERVAL_CSV_HEADER: &str = "index,interval_us";
const INTERVAL_CSV_ROW_COUNT: usize = 138;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntervalCapture {
    pub intervals: Vec<u32>,
}

pub fn load_interval_csv(path: &Path) -> Result<IntervalCapture, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|err| format!("read error for {}: {err}", path.display()))?;
    parse_interval_csv(&text).map_err(|err| format!("{}: {err}", path.display()))
}

pub fn parse_interval_csv(text: &str) -> Result<IntervalCapture, String> {
    let mut lines = text.lines();
    let Some(header) = lines.next() else {
        return Err("empty csv".to_string());
    };
    if header != INTERVAL_CSV_HEADER {
        return Err(format!("invalid header, expected {INTERVAL_CSV_HEADER}"));
    }

    let mut intervals = Vec::new();
    for (expected_index, line) in lines.enumerate() {
        if line.is_empty() {
            return Err("invalid empty row".to_string());
        }
        let mut parts = line.split(',');
        let Some(index_str) = parts.next() else {
            return Err("invalid row".to_string());
        };
        let Some(interval_str) = parts.next() else {
            return Err("invalid row".to_string());
        };
        if parts.next().is_some() {
            return Err("invalid row".to_string());
        }

        let index = index_str
            .parse::<usize>()
            .map_err(|err| format!("invalid index at row {} ({err})", expected_index + 1))?;
        if index != expected_index {
            return Err("non-contiguous index".to_string());
        }

        let interval = interval_str
            .parse::<u32>()
            .map_err(|err| format!("invalid interval at row {} ({err})", expected_index + 1))?;
        if interval == 0 {
            return Err(format!(
                "interval must be > 0 at row {}",
                expected_index + 1
            ));
        }
        intervals.push(interval);
    }

    if intervals.is_empty() {
        return Err("no interval rows".to_string());
    }
    if intervals.len() != INTERVAL_CSV_ROW_COUNT {
        return Err(format!(
            "expected {INTERVAL_CSV_ROW_COUNT} interval rows, found {}",
            intervals.len()
        ));
    }

    Ok(IntervalCapture { intervals })
}

pub fn import_interval_capture_bytes(capture: &IntervalCapture) -> Vec<u8> {
    let header = Header1 {
        magic: MAGIC,
        version: VERSION1,
        header_len: HEADER1_SIZE as u16,
        frame_count: FRAME_COUNT as u32,
        frame_size: FRAME_SIZE as u16,
        flags: 0,
        schema_len: EMPTY_SCHEMA.len() as u32,
        schema_hash: Sha256::digest(EMPTY_SCHEMA).into(),
        build_hash: Sha256::digest(BUILD_HASH_INPUT).into(),
        config_hash: Sha256::digest(CONFIG_HASH_INPUT).into(),
        board_id: BOARD_ID,
        clock_profile: CLOCK_PROFILE,
        capture_boundary: IMPORT_CAPTURE_BOUNDARY_ISR,
        reserved: 0,
    };

    let mut out =
        Vec::with_capacity(HEADER1_SIZE + EMPTY_SCHEMA.len() + (FRAME_COUNT * FRAME_SIZE));
    out.extend_from_slice(&encode_header1_le(&header));
    out.extend_from_slice(EMPTY_SCHEMA);

    for frame_idx in 0..FRAME_COUNT {
        let input_sample = if frame_idx < capture.intervals.len() {
            capture.intervals[frame_idx] as i32
        } else {
            0
        };
        let frame = EventFrame0 {
            frame_idx: frame_idx as u32,
            irq_id: IMPORT_IRQ_ID,
            flags: 0,
            rsv: 0,
            timer_delta: IMPORT_TIMER_DELTA,
            input_sample,
        };
        out.extend_from_slice(&encode_event_frame0_le(&frame));
    }

    out
}
