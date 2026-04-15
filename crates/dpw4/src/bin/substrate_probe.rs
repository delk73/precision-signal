#![forbid(unsafe_code)]

#[path = "common/mod.rs"]
mod common;

use common::{ArtifactStaging, CliError, CliResult, CliStatus, ResultBlock};
use std::io;

const DEFAULT_TARGET: &str = "substrate://probe/default";

fn parse_probe_args() -> Result<(Option<String>, String), CliError> {
    let mut force_id = None;
    let mut target = DEFAULT_TARGET.to_string();
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--force-id" => {
                let value = args.next().ok_or_else(|| {
                    CliError::User("--force-id requires a value".to_string())
                })?;
                force_id = Some(value);
            }
            "--target" => {
                target = args.next().ok_or_else(|| {
                    CliError::User("--target requires a value".to_string())
                })?;
            }
            "--help" | "-h" => {
                return Err(CliError::User(
                    "usage: substrate_probe [--force-id RUN_ID] [--target TARGET]".to_string(),
                ));
            }
            other => {
                return Err(CliError::User(format!("unexpected argument: {other}")));
            }
        }
    }

    Ok((force_id, target))
}

fn run() -> CliResult {
    let (force_id, target) = parse_probe_args()?;
    let staging = ArtifactStaging::new("artifacts");
    let result_block = ResultBlock {
        result: "PASS".to_string(),
        command: "substrate-probe".to_string(),
        target,
        mode: "audit".to_string(),
        equivalence: "exact".to_string(),
        first_divergence: "none".to_string(),
        artifact: "artifacts/PLACEHOLDER".to_string(),
    };

    let published = match force_id {
        Some(run_id) => staging.stage_and_publish_with_run_id(&run_id, &result_block, b"{}", b"{}"),
        None => staging.stage_and_publish(&result_block, b"{}", b"{}"),
    };

    match published {
        Ok(published) => {
            published.result_block.write_to_stdout()?;
            Ok(CliStatus::Success)
        }
        Err(CliError::Io(err)) if err.kind() == io::ErrorKind::AlreadyExists => {
            Err(CliError::User(err.to_string()))
        }
        Err(err) => Err(err),
    }
}

fn main() {
    common::exit_with_result(run());
}
