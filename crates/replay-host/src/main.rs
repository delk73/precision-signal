#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use replay_host::diff_artifacts0;

fn usage(program: &str) {
    eprintln!("usage: {program} diff <artifact-a.rpl> <artifact-b.rpl>");
}

fn read_file(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|err| format!("read error for {}: {err}", path.display()))
}

fn run() -> Result<(), String> {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "replay-host".to_string());
    let Some(command) = args.next() else {
        usage(&program);
        return Err("missing command".to_string());
    };

    if command != "diff" {
        usage(&program);
        return Err(format!("unknown command: {command}"));
    }

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

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}
