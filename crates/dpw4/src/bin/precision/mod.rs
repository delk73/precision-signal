use crate::common;
use clap::{Parser, Subcommand, ValueEnum};
use common::{CliError, CliResult, CliStatus};
use std::io;
use std::path::PathBuf;

mod artifacts;
mod generate;
mod inspect;
mod validate;
mod verify;

/// Precision-DPW Reference Tool
///
/// Generates, inspects, and validates
/// DP32 Reference Standard signals.
#[derive(Parser)]
#[command(name = "precision")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "DP32 Reference Signal Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Emit build provenance metadata as JSON
    #[arg(long, global = true)]
    audit_state: bool,

    #[cfg(feature = "audit")]
    /// Print audit telemetry at end of run
    #[arg(long)]
    audit: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a reference signal (Default)
    Generate(GenerateArgs),
    /// Generate forensic artifacts into a directory
    Artifacts(ArtifactsArgs),
    /// Run canonical deterministic validation gate
    Validate(ValidateArgs),
    /// Inspect a DP32 file header without reading the payload
    Inspect {
        /// Input file path (defaults to stdin if not provided)
        #[arg(short, long, value_name = "FILE")]
        file: Option<PathBuf>,
    },
    /// Verify a DP32 file for integrity and alignment
    Verify {
        /// Input file path
        #[arg(short, long, required = true, value_name = "FILE")]
        file: PathBuf,
    },
}

#[derive(Clone, ValueEnum)]
enum ShapeArg {
    Saw,
    Square,
    Triangle,
    #[value(name = "triangle-dpw1")]
    TriangleDpw1,
    Pulse,
}

#[derive(Parser)]
struct GenerateArgs {
    /// Waveform shape
    #[arg(short, long, value_enum, default_value_t = ShapeArg::Saw)]
    shape: ShapeArg,

    /// Frequency in Hz
    #[arg(short, long, default_value_t = 440.0)]
    freq: f64,

    /// Sample rate in Hz
    #[arg(short, long, default_value_t = 48000)]
    rate: u32,

    /// Duration in seconds (Optional). If omitted, streams indefinitely.
    #[arg(long)]
    seconds: Option<u64>,

    /// Output gain in dB
    #[arg(long, default_value_t = -3.0)]
    gain: f64,

    /// Wrap in RIFF WAVE container (Requires --seconds)
    #[arg(long)]
    container_wav: bool,

    /// Output path (`-` for stdout)
    #[arg(short = 'o', long, default_value = "-", value_name = "PATH")]
    out: PathBuf,
}

#[derive(Parser)]
struct ArtifactsArgs {
    /// Output directory for generated forensic artifacts
    #[arg(short = 'o', long, default_value = "docs/verification", value_name = "PATH")]
    out: PathBuf,
}

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum ValidateMode {
    Quick,
    Full,
}

#[derive(Parser)]
struct ValidateArgs {
    /// Output directory for validation run artifacts
    #[arg(short = 'o', long, value_name = "PATH")]
    out: Option<PathBuf>,

    /// Validation mode
    #[arg(long, value_enum, default_value_t = ValidateMode::Quick)]
    mode: ValidateMode,

    /// Emit a single JSON report to stdout; logs go to stderr
    #[arg(long)]
    json: bool,

    /// Keep run directories even when validation passes
    #[arg(long)]
    keep: bool,
}

#[derive(Debug)]
pub(crate) enum VerifyError {
    Io(io::Error),
    Parse(String),
    Integrity(String),
}

impl core::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            VerifyError::Io(e) => write!(f, "{}", e),
            VerifyError::Parse(msg) => write!(f, "{}", msg),
            VerifyError::Integrity(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for VerifyError {}

impl From<io::Error> for VerifyError {
    fn from(value: io::Error) -> Self {
        VerifyError::Io(value)
    }
}

fn run_artifacts(args: ArtifactsArgs) -> io::Result<()> {
    artifacts::generate_forensic_artifacts(&args.out, None)
}

pub(crate) fn run() -> CliResult {
    let cli = common::parse_args::<Cli>()?;

    if cli.audit_state {
        println!("{}", common::audit_state_json("precision"));
        return Ok(CliStatus::Info);
    }

    #[cfg(feature = "audit")]
    if cli.audit {
        dpw4::reset_audit_counters();
    }

    let command = match cli.command {
        Some(command) => command,
        None => Commands::Generate(common::parse_from_args(std::env::args())?),
    };

    let result = match command {
        Commands::Generate(args) => generate::run_generate(args).map(|()| CliStatus::Success),
        Commands::Artifacts(args) => run_artifacts(args).map(|()| CliStatus::Success).map_err(CliError::from),
        Commands::Validate(args) => Ok(validate::run_validate(args)),
        Commands::Inspect { file } => inspect::run_inspect(file).map(|()| CliStatus::Success).map_err(CliError::from),
        Commands::Verify { file } => match verify::run_verify(file) {
            Ok(()) => Ok(CliStatus::Success),
            Err(VerifyError::Integrity(msg)) => Err(CliError::Integrity(msg)),
            Err(VerifyError::Io(err)) => Err(CliError::Io(err)),
            Err(VerifyError::Parse(msg)) => Err(CliError::User(msg)),
        },
    };

    #[cfg(feature = "audit")]
    if cli.audit {
        eprintln!("AUDIT: Max |z| bitlen: {}", dpw4::max_abs_z_bits());
        eprintln!(
            "AUDIT: Legacy Shift Overflow Risk: {}",
            dpw4::legacy_shift_overflow_risk()
        );
        eprintln!(
            "AUDIT: Integrator Near Overflow:   {}",
            dpw4::integrator_near_overflow()
        );
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quick_validate_gain_mantissa_is_singleton() {
        for scenario in artifacts::quick_validate_scenarios() {
            let gain = artifacts::quick_validate_gain_for_scenario(scenario);
            assert_eq!(
                gain.m4_q63,
                artifacts::GAIN_M4_Q63_QUICK,
                "scenario {} must use singleton quick mantissa",
                scenario.id
            );
        }
    }
}
