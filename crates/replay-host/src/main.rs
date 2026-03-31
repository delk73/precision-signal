#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use replay_core::artifact::{
    encode_event_frame0_le, encode_header1_le, EventFrame0, Header1, FRAME_COUNT, FRAME_SIZE,
    HEADER1_SIZE, MAGIC, VERSION1,
};
use replay_host::diff_artifacts0;
use sha2::{Digest, Sha256};

const IMPORT_IRQ_ID: u8 = 0x02;
const IMPORT_TIMER_DELTA: u32 = 1000;
const IMPORT_CAPTURE_BOUNDARY_ISR: u16 = 0;
const EMPTY_SCHEMA: &[u8] = b"";
const BUILD_HASH_INPUT: &[u8] = b"replay-host:import-interval-csv:v1";
const CONFIG_HASH_INPUT: &[u8] = b"source=index,interval_us;pad=zero;timer_delta=1000;irq=0x02";
const BOARD_ID: [u8; 16] = *b"interval-csv-run";
const CLOCK_PROFILE: [u8; 16] = *b"offline-fixed-v1";

fn usage(program: &str) {
    eprintln!(
        "usage: {program} diff <artifact-a.rpl> <artifact-b.rpl>\n       {program} import-interval-csv <intervals.csv> <artifact.rpl>"
    );
}

fn read_file(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|err| format!("read error for {}: {err}", path.display()))
}

fn load_interval_csv(path: &Path) -> Result<Vec<u32>, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("read error for {}: {err}", path.display()))?;

    let mut lines = text.lines();
    let Some(header) = lines.next() else {
        return Err(format!("{}: empty csv", path.display()));
    };
    if header != "index,interval_us" {
        return Err(format!("{}: invalid header", path.display()));
    }

    let mut intervals = Vec::new();
    for (expected_index, line) in lines.enumerate() {
        if line.is_empty() {
            return Err(format!("{}: invalid empty row", path.display()));
        }
        let mut parts = line.split(',');
        let Some(index_str) = parts.next() else {
            return Err(format!("{}: invalid row", path.display()));
        };
        let Some(interval_str) = parts.next() else {
            return Err(format!("{}: invalid row", path.display()));
        };
        if parts.next().is_some() {
            return Err(format!("{}: invalid row", path.display()));
        }

        let index = index_str.parse::<usize>().map_err(|err| {
            format!(
                "{}: invalid index at row {} ({err})",
                path.display(),
                expected_index + 1
            )
        })?;
        if index != expected_index {
            return Err(format!("{}: non-contiguous index", path.display()));
        }

        let interval = interval_str.parse::<u32>().map_err(|err| {
            format!(
                "{}: invalid interval at row {} ({err})",
                path.display(),
                expected_index + 1
            )
        })?;
        intervals.push(interval);
    }

    if intervals.is_empty() {
        return Err(format!("{}: no interval rows", path.display()));
    }

    Ok(intervals)
}

fn import_interval_csv(csv_path: &Path, out_path: &Path) -> Result<(), String> {
    let intervals = load_interval_csv(csv_path)?;

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
        let input_sample = if frame_idx < intervals.len() {
            intervals[frame_idx] as i32
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

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("mkdir error for {}: {err}", parent.display()))?;
    }
    fs::write(out_path, out)
        .map_err(|err| format!("write error for {}: {err}", out_path.display()))?;

    println!("imported: {}", csv_path.display());
    println!("wrote: {}", out_path.display());
    println!("source_rows: {}", intervals.len());
    println!("frame_count: {}", FRAME_COUNT);
    Ok(())
}

fn run() -> Result<(), String> {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "replay-host".to_string());
    let Some(command) = args.next() else {
        usage(&program);
        return Err("missing command".to_string());
    };

    match command.as_str() {
        "diff" => {
            let Some(a_path) = args.next() else {
                usage(&program);
                return Err("missing first artifact path".to_string());
            };
            let Some(b_path) = args.next() else {
                usage(&program);
                return Err("missing second artifact path".to_string());
            };

            if args.next().is_some() {
                usage(&program);
                return Err("unexpected extra arguments".to_string());
            }

            let a_bytes = read_file(Path::new(&a_path))?;
            let b_bytes = read_file(Path::new(&b_path))?;

            match diff_artifacts0(&a_bytes, &b_bytes) {
                Ok(None) => {
                    println!("no divergence");
                    Ok(())
                }
                Ok(Some(idx)) => {
                    println!("first divergence at frame {idx}");
                    Ok(())
                }
                Err(err) => Err(format!("parse error: {err:?}")),
            }
        }
        "import-interval-csv" => {
            let Some(csv_path) = args.next() else {
                usage(&program);
                return Err("missing interval csv path".to_string());
            };
            let Some(out_path) = args.next() else {
                usage(&program);
                return Err("missing output artifact path".to_string());
            };

            if args.next().is_some() {
                usage(&program);
                return Err("unexpected extra arguments".to_string());
            }

            import_interval_csv(Path::new(&csv_path), Path::new(&out_path))
        }
        _ => {
            usage(&program);
            Err(format!("unknown command: {command}"))
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}
