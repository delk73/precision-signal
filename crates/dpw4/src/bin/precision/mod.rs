use crate::common;
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use common::{ArtifactStaging, CliResult, ResultBlock};
use std::ffi::OsString;

#[derive(Parser)]
#[command(name = "precision")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Authoritative operator surface", long_about = None)]
#[command(disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Emit a record result block and artifact
    Record(CommandArgs),
    /// Emit a replay result block and artifact
    Replay(CommandArgs),
    /// Emit a diff result block and artifact
    Diff(CommandArgs),
    /// Emit an envelope result block and artifact
    Envelope(CommandArgs),
}

#[derive(Args)]
struct CommandArgs {
    /// User-supplied target path or identifier
    #[arg(value_name = "TARGET")]
    target: String,

    /// Authoritative execution mode
    #[arg(long, value_enum, value_name = "MODE")]
    mode: ModeArg,
}

#[derive(Clone, Copy, ValueEnum)]
enum ModeArg {
    #[value(name = "runtime_mode")]
    RuntimeMode,
    Mock,
    None,
}

impl ModeArg {
    fn as_str(self) -> &'static str {
        match self {
            ModeArg::RuntimeMode => "runtime_mode",
            ModeArg::Mock => "mock",
            ModeArg::None => "none",
        }
    }
}

pub(crate) fn is_authoritative_command(command: &str) -> bool {
    matches!(command, "record" | "replay" | "diff" | "envelope")
}

pub(crate) fn help_summary() -> String {
    let mut command = Cli::command().disable_colored_help(true);
    let mut out = Vec::new();
    command.write_help(&mut out).expect("render precision help");
    String::from_utf8(out).expect("precision help is utf-8")
}

pub(crate) fn minimal_usage_summary() -> String {
    concat!(
        "usage: precision <command>\n",
        "commands:\n",
        "  record\n",
        "  replay\n",
        "  diff\n",
        "  envelope\n",
    )
    .to_string()
}

pub(crate) fn version_summary() -> String {
    format!("precision {}\n", env!("CARGO_PKG_VERSION"))
}

pub(crate) fn run<I, U>(itr: I) -> CliResult
where
    I: IntoIterator<Item = U>,
    U: Into<OsString> + Clone,
{
    let cli = common::parse_from_args::<Cli, _, _>(itr)?;
    match cli.command {
        Commands::Record(args) => run_stub("record", args),
        Commands::Replay(args) => run_stub("replay", args),
        Commands::Diff(args) => run_stub("diff", args),
        Commands::Envelope(args) => run_stub("envelope", args),
    }
}

fn run_stub(command: &str, args: CommandArgs) -> CliResult {
    let result_block = ResultBlock {
        result: "PASS".to_string(),
        command: command.to_string(),
        target: args.target,
        mode: args.mode.as_str().to_string(),
        equivalence: "exact".to_string(),
        first_divergence: "none".to_string(),
        artifact: "artifacts/PLACEHOLDER".to_string(),
    };

    ArtifactStaging::new("artifacts").stage_publish_and_emit(&result_block, b"{}", b"{}")
}
