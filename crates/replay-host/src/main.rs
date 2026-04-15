#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use replay_core::artifact::FRAME_COUNT;
use replay_host::{diff_artifacts0, import_interval_capture_bytes, load_interval_csv};

fn usage(program: &str) {
    eprintln!(
        "usage: {program} diff <artifact-a.rpl> <artifact-b.rpl>\n       {program} validate-interval-csv <intervals.csv>\n       {program} import-interval-csv <intervals.csv> <artifact.rpl>"
    );
}

fn read_file(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|err| format!("read error for {}: {err}", path.display()))
}

fn validate_interval_csv_quiet(csv_path: &Path) -> Result<replay_host::IntervalCapture, String> {
    load_interval_csv(csv_path)
}

fn validate_interval_csv(csv_path: &Path) -> Result<(), String> {
    let capture = validate_interval_csv_quiet(csv_path)?;
    println!("validated: {}", csv_path.display());
    println!("header: index,interval_us");
    println!("rows: {}", capture.intervals.len());
    println!("first_index: 0");
    println!("last_index: {}", capture.intervals.len() - 1);
    Ok(())
}

fn import_interval_csv(csv_path: &Path, out_path: &Path) -> Result<(), String> {
    let capture = validate_interval_csv_quiet(csv_path)?;
    let out = import_interval_capture_bytes(&capture);

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("mkdir error for {}: {err}", parent.display()))?;
    }
    fs::write(out_path, out)
        .map_err(|err| format!("write error for {}: {err}", out_path.display()))?;

    println!("imported: {}", csv_path.display());
    println!("wrote: {}", out_path.display());
    println!("source_rows: {}", capture.intervals.len());
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
        "validate-interval-csv" => {
            let Some(csv_path) = args.next() else {
                usage(&program);
                return Err("missing interval csv path".to_string());
            };

            if args.next().is_some() {
                usage(&program);
                return Err("unexpected extra arguments".to_string());
            }

            validate_interval_csv(Path::new(&csv_path))
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
