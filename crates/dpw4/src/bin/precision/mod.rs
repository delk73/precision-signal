use crate::common;
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use common::{ArtifactStaging, CliError, CliResult, ResultBlock};
use dpw4::{DpwGain, Oscillator, Scalar};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const INTERVAL_ROW_COUNT: usize = 138;
const TRACE_SCHEMA: &str = "precision.trace.v1";
const META_SCHEMA: &str = "precision.meta.v1";
const TRANSIENT_FRAME_COUNT: usize = 10_000;
const TRANSIENT_FRAME_SIZE: usize = 16;
const TRANSIENT_HEADER_SIZE: usize = 0x98;
const TRANSIENT_CAPTURE_BOUNDARY: u16 = 0;
const TRANSIENT_BUILD_HASH_INPUT: &[u8] = b"precision:record:interval-csv:v1";
const TRANSIENT_CONFIG_HASH_INPUT: &[u8] =
    b"source=index,interval_us;pad=zero;timer_delta=1000;irq=0x02";

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
    Diff(DiffArgs),
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

#[derive(Args)]
struct DiffArgs {
    /// First authoritative artifact directory
    #[arg(value_name = "TARGET_A")]
    target_a: String,

    /// Second authoritative artifact directory
    #[arg(value_name = "TARGET_B")]
    target_b: String,

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

#[derive(Clone, Debug, Deserialize, Serialize)]
struct SemanticNode {
    id: String,
    values: Vec<Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct SemanticTrace {
    nodes: Vec<SemanticNode>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct TraceArtifact {
    schema: String,
    signal_inputs: Vec<u32>,
    captured_trace: SemanticTrace,
    #[serde(skip_serializing_if = "Option::is_none")]
    replay_trace: Option<SemanticTrace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comparison: Option<ComparisonSummary>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ComparisonSummary {
    equivalence: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    first_divergence: Option<FirstDivergence>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct MetaArtifact {
    schema: String,
    command: String,
    target: String,
    mode: String,
    created_at_unix_s: u64,
    hostname: Option<String>,
    pid: u32,
    source_kind: String,
    signal_input_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    transient_rpl0_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    secondary_target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comparison_performed: Option<bool>,
}

#[derive(Clone, Debug)]
struct LoadedArtifact {
    trace: TraceArtifact,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ArtifactPurpose {
    Replay,
    Diff,
    Envelope,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct FirstDivergence {
    step: u64,
    node: String,
    cause: String,
}

#[derive(Clone, Debug)]
struct RecordCapture {
    intervals: Vec<u32>,
    source_kind: String,
    transient_rpl0_sha256: String,
}

pub(crate) fn is_authoritative_command(command: &str) -> bool {
    matches!(command, "record" | "replay" | "diff" | "envelope")
}

pub(crate) fn help_summary() -> String {
    let mut command = Cli::command().disable_colored_help(true);
    let mut out = Vec::new();
    if command.write_help(&mut out).is_err() {
        return minimal_usage_summary();
    }
    match String::from_utf8(out) {
        Ok(help) => help,
        Err(_) => minimal_usage_summary(),
    }
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
        Commands::Record(args) => run_record(args),
        Commands::Replay(args) => run_replay(args),
        Commands::Diff(args) => run_diff(args),
        Commands::Envelope(args) => run_envelope(args),
    }
}

fn run_record(args: CommandArgs) -> CliResult {
    if matches!(args.mode, ModeArg::Mock) {
        return publish_mock_result("record", args.target, args.mode.as_str());
    }

    let capture = acquire_record_capture(&args.target)?;
    let captured_trace = synthesize_semantic_trace(&capture.intervals);
    let trace = TraceArtifact {
        schema: TRACE_SCHEMA.to_string(),
        signal_inputs: capture.intervals.clone(),
        captured_trace,
        replay_trace: None,
        comparison: None,
    };
    let meta = MetaArtifact {
        schema: META_SCHEMA.to_string(),
        command: "record".to_string(),
        target: args.target.clone(),
        mode: args.mode.as_str().to_string(),
        created_at_unix_s: now_unix_seconds()?,
        hostname: std::env::var("HOSTNAME").ok(),
        pid: std::process::id(),
        source_kind: capture.source_kind,
        signal_input_count: capture.intervals.len(),
        transient_rpl0_sha256: Some(capture.transient_rpl0_sha256),
        secondary_target: None,
        comparison_performed: Some(false),
    };

    // Contract rule: a successful `record` emits PASS/exact/none by definition
    // of successful capture, even though no replay comparison has occurred.
    publish_result(
        success_block("record", args.target, args.mode.as_str()),
        &trace,
        &meta,
    )
}

fn run_replay(args: CommandArgs) -> CliResult {
    if matches!(args.mode, ModeArg::Mock) {
        return publish_mock_result("replay", args.target, args.mode.as_str());
    }

    let loaded = load_authoritative_artifact(&args.target, ArtifactPurpose::Replay)?;
    let replay_trace = synthesize_semantic_trace(&loaded.trace.signal_inputs);
    let divergence = compare_traces(&loaded.trace.captured_trace, &replay_trace);
    let comparison = ComparisonSummary {
        equivalence: if divergence.is_none() {
            "exact".to_string()
        } else {
            "diverged".to_string()
        },
        first_divergence: divergence.clone(),
    };
    let trace = TraceArtifact {
        schema: TRACE_SCHEMA.to_string(),
        signal_inputs: loaded.trace.signal_inputs.clone(),
        captured_trace: loaded.trace.captured_trace,
        replay_trace: Some(replay_trace),
        comparison: Some(comparison),
    };
    let meta = MetaArtifact {
        schema: META_SCHEMA.to_string(),
        command: "replay".to_string(),
        target: args.target.clone(),
        mode: args.mode.as_str().to_string(),
        created_at_unix_s: now_unix_seconds()?,
        hostname: std::env::var("HOSTNAME").ok(),
        pid: std::process::id(),
        source_kind: "authoritative_artifact".to_string(),
        signal_input_count: trace.signal_inputs.len(),
        transient_rpl0_sha256: None,
        secondary_target: None,
        comparison_performed: Some(true),
    };

    let block = result_block_from_divergence("replay", args.target, args.mode.as_str(), divergence);
    publish_result(block, &trace, &meta)
}

fn run_diff(args: DiffArgs) -> CliResult {
    let diff_target = encode_diff_target(&args.target_a, &args.target_b)?;
    if matches!(args.mode, ModeArg::Mock) {
        return publish_mock_result("diff", diff_target, args.mode.as_str());
    }

    let left = load_authoritative_artifact(&args.target_a, ArtifactPurpose::Diff)?;
    let right = load_authoritative_artifact(&args.target_b, ArtifactPurpose::Diff)?;
    let divergence = compare_traces(
        &semantic_trace_for_diff(&left.trace),
        &semantic_trace_for_diff(&right.trace),
    );
    // `diff` publishes a comparison-only artifact shape. It is authoritative
    // for diff/reporting, but intentionally incompatible with replay/envelope.
    let trace_json = json!({
        "schema": TRACE_SCHEMA,
        "signal_inputs": [],
        "captured_trace": {
            "nodes": []
        },
        "replay_trace": Value::Null,
        "comparison": {
            "equivalence": if divergence.is_none() { "exact" } else { "diverged" },
            "first_divergence": divergence,
        },
        "left_target": args.target_a,
        "right_target": args.target_b
    });
    let meta = MetaArtifact {
        schema: META_SCHEMA.to_string(),
        command: "diff".to_string(),
        target: diff_target.clone(),
        mode: args.mode.as_str().to_string(),
        created_at_unix_s: now_unix_seconds()?,
        hostname: std::env::var("HOSTNAME").ok(),
        pid: std::process::id(),
        source_kind: "authoritative_artifact_pair".to_string(),
        signal_input_count: 0,
        transient_rpl0_sha256: None,
        secondary_target: Some(args.target_b.clone()),
        comparison_performed: Some(true),
    };

    let block = result_block_from_divergence(
        "diff",
        diff_target,
        args.mode.as_str(),
        divergence,
    );
    publish_json_result(block, trace_json, &meta)
}

fn run_envelope(args: CommandArgs) -> CliResult {
    if matches!(args.mode, ModeArg::Mock) {
        return publish_mock_result("envelope", args.target, args.mode.as_str());
    }

    let loaded = load_authoritative_artifact(&args.target, ArtifactPurpose::Envelope)?;
    let replay_trace = synthesize_semantic_trace(&loaded.trace.signal_inputs);
    let captured_square = extract_i64_node_values(&loaded.trace.captured_trace, "dpw4.square")?;
    let replay_square = extract_i64_node_values(&replay_trace, "dpw4.square")?;
    let captured_envelope = build_envelope_trace(&captured_square);
    let replay_envelope = build_envelope_trace(&replay_square);
    let divergence = compare_traces(&captured_envelope, &replay_envelope);
    let trace_json = json!({
        "schema": TRACE_SCHEMA,
        "signal_inputs": loaded.trace.signal_inputs,
        "captured_trace": captured_envelope,
        "replay_trace": replay_envelope,
        "comparison": {
            "equivalence": if divergence.is_none() { "exact" } else { "diverged" },
            "first_divergence": divergence,
        }
    });
    let meta = MetaArtifact {
        schema: META_SCHEMA.to_string(),
        command: "envelope".to_string(),
        target: args.target.clone(),
        mode: args.mode.as_str().to_string(),
        created_at_unix_s: now_unix_seconds()?,
        hostname: std::env::var("HOSTNAME").ok(),
        pid: std::process::id(),
        source_kind: "authoritative_artifact".to_string(),
        signal_input_count: loaded.trace.signal_inputs.len(),
        transient_rpl0_sha256: None,
        secondary_target: None,
        comparison_performed: Some(true),
    };

    publish_json_result(
        result_block_from_divergence("envelope", args.target, args.mode.as_str(), divergence),
        trace_json,
        &meta,
    )
}

fn publish_mock_result(command: &str, target: String, mode: &str) -> CliResult {
    let trace_json = json!({
        "schema": TRACE_SCHEMA,
        "signal_inputs": [],
        "captured_trace": { "nodes": [] },
        "comparison": {
            "equivalence": "diverged",
            "first_divergence": null
        }
    });
    let meta = MetaArtifact {
        schema: META_SCHEMA.to_string(),
        command: command.to_string(),
        target: target.clone(),
        mode: mode.to_string(),
        created_at_unix_s: now_unix_seconds()?,
        hostname: std::env::var("HOSTNAME").ok(),
        pid: std::process::id(),
        source_kind: "mock".to_string(),
        signal_input_count: 0,
        transient_rpl0_sha256: None,
        secondary_target: None,
        comparison_performed: Some(false),
    };
    let block = ResultBlock {
        result: "FAIL".to_string(),
        command: command.to_string(),
        target,
        mode: mode.to_string(),
        equivalence: "diverged".to_string(),
        first_divergence: "none".to_string(),
        artifact: "artifacts/PLACEHOLDER".to_string(),
    };
    publish_json_result(block, trace_json, &meta)
}

fn publish_result(result_block: ResultBlock, trace: &TraceArtifact, meta: &MetaArtifact) -> CliResult {
    let trace_json = serde_json::to_vec_pretty(trace)
        .map_err(|err| CliError::Integrity(format!("trace serialization failed: {err}")))?;
    let meta_json = serde_json::to_vec_pretty(meta)
        .map_err(|err| CliError::Integrity(format!("meta serialization failed: {err}")))?;
    ArtifactStaging::new("artifacts").stage_publish_and_emit(&result_block, &trace_json, &meta_json)
}

fn publish_json_result(result_block: ResultBlock, trace: Value, meta: &MetaArtifact) -> CliResult {
    let trace_json = serde_json::to_vec_pretty(&trace)
        .map_err(|err| CliError::Integrity(format!("trace serialization failed: {err}")))?;
    let meta_json = serde_json::to_vec_pretty(meta)
        .map_err(|err| CliError::Integrity(format!("meta serialization failed: {err}")))?;
    ArtifactStaging::new("artifacts").stage_publish_and_emit(&result_block, &trace_json, &meta_json)
}

fn success_block(command: &str, target: String, mode: &str) -> ResultBlock {
    ResultBlock {
        result: "PASS".to_string(),
        command: command.to_string(),
        target,
        mode: mode.to_string(),
        equivalence: "exact".to_string(),
        first_divergence: "none".to_string(),
        artifact: "artifacts/PLACEHOLDER".to_string(),
    }
}

fn result_block_from_divergence(
    command: &str,
    target: String,
    mode: &str,
    divergence: Option<FirstDivergence>,
) -> ResultBlock {
    let (result, equivalence, first_divergence) = match divergence {
        Some(divergence) => (
            "FAIL".to_string(),
            "diverged".to_string(),
            format!(
                "step={} node={} cause={}",
                divergence.step, divergence.node, divergence.cause
            ),
        ),
        None => ("PASS".to_string(), "exact".to_string(), "none".to_string()),
    };
    ResultBlock {
        result,
        command: command.to_string(),
        target,
        mode: mode.to_string(),
        equivalence,
        first_divergence,
        artifact: "artifacts/PLACEHOLDER".to_string(),
    }
}

fn load_authoritative_artifact(
    target: &str,
    purpose: ArtifactPurpose,
) -> Result<LoadedArtifact, CliError> {
    let base = Path::new(target);
    validate_authoritative_artifact_dir(base, target)?;
    let result_path = base.join("result.txt");
    let trace_path = base.join("trace.json");
    let meta_path = base.join("meta.json");
    let result_bytes = fs::read(&result_path)?;
    let trace_bytes = fs::read(&trace_path)?;
    let meta_bytes = fs::read(&meta_path)?;
    let result = parse_published_result_block(target, base, &result_bytes)?;
    let trace: TraceArtifact = serde_json::from_slice(&trace_bytes).map_err(|err| {
        CliError::User(format!("invalid authoritative trace at {}: {err}", trace_path.display()))
    })?;
    let meta: MetaArtifact = serde_json::from_slice(&meta_bytes).map_err(|err| {
        CliError::User(format!("invalid authoritative meta at {}: {err}", meta_path.display()))
    })?;
    validate_loaded_artifact(target, &result, &trace, &meta, purpose)?;
    Ok(LoadedArtifact { trace })
}

fn synthesize_semantic_trace(intervals: &[u32]) -> SemanticTrace {
    let gain = DpwGain::new(1u64 << 63, 0, 0, 0);
    let mut oscillator = Oscillator::new_u32(48_000);
    let mut interval_values = Vec::with_capacity(intervals.len());
    let mut square_values = Vec::with_capacity(intervals.len());
    let mut hash_values = Vec::with_capacity(intervals.len());
    let mut acc = 0x9e37_79b9_7f4a_7c15_u64;

    for (step, interval) in intervals.iter().copied().enumerate() {
        interval_values.push(Value::from(interval));
        let frequency_hz = 1_000_000.0 / interval as f64;
        oscillator.frequency = Scalar::from_num(frequency_hz);
        let sample = oscillator.tick(3, &gain);
        square_values.push(Value::from(sample as i64));
        acc = mix_trace_hash(acc, step as u64, interval as u64, sample as i64);
        hash_values.push(Value::from(acc));
    }

    SemanticTrace {
        nodes: vec![
            SemanticNode {
                id: "signal.interval_us".to_string(),
                values: interval_values,
            },
            SemanticNode {
                id: "dpw4.square".to_string(),
                values: square_values,
            },
            SemanticNode {
                id: "dpw4.hash".to_string(),
                values: hash_values,
            },
        ],
    }
}

fn compare_traces(expected: &SemanticTrace, actual: &SemanticTrace) -> Option<FirstDivergence> {
    let max_nodes = expected.nodes.len().max(actual.nodes.len());
    for node_idx in 0..max_nodes {
        let Some(left) = expected.nodes.get(node_idx) else {
            let right = actual.nodes.get(node_idx)?;
            return Some(FirstDivergence {
                step: 0,
                node: right.id.clone(),
                cause: "OOB".to_string(),
            });
        };
        let Some(right) = actual.nodes.get(node_idx) else {
            return Some(FirstDivergence {
                step: 0,
                node: left.id.clone(),
                cause: "OOB".to_string(),
            });
        };
        if left.id != right.id {
            return Some(FirstDivergence {
                step: 0,
                node: left.id.clone(),
                cause: "TYPE_MISMATCH".to_string(),
            });
        }

        let max_values = left.values.len().max(right.values.len());
        for step in 0..max_values {
            match (left.values.get(step), right.values.get(step)) {
                (Some(a), Some(b)) => {
                    if json_type_tag(a) != json_type_tag(b) {
                        return Some(FirstDivergence {
                            step: step as u64,
                            node: left.id.clone(),
                            cause: "TYPE_MISMATCH".to_string(),
                        });
                    }
                    if a != b {
                        return Some(FirstDivergence {
                            step: step as u64,
                            node: left.id.clone(),
                            cause: "VAL_MISMATCH".to_string(),
                        });
                    }
                }
                _ => {
                    return Some(FirstDivergence {
                        step: step as u64,
                        node: left.id.clone(),
                        cause: "OOB".to_string(),
                    });
                }
            }
        }
    }
    None
}

fn semantic_trace_for_diff(trace: &TraceArtifact) -> SemanticTrace {
    let mut nodes = vec![SemanticNode {
        id: "artifact.signal_inputs".to_string(),
        values: trace
            .signal_inputs
            .iter()
            .copied()
            .map(Value::from)
            .collect(),
    }];
    nodes.extend(trace.captured_trace.nodes.clone());
    SemanticTrace { nodes }
}

fn build_envelope_trace(samples: &[i64]) -> SemanticTrace {
    let mut mins = Vec::with_capacity(samples.len());
    let mut maxs = Vec::with_capacity(samples.len());
    let mut lo = i64::MAX;
    let mut hi = i64::MIN;
    for sample in samples {
        lo = lo.min(*sample);
        hi = hi.max(*sample);
        mins.push(Value::from(lo));
        maxs.push(Value::from(hi));
    }
    SemanticTrace {
        nodes: vec![
            SemanticNode {
                id: "envelope.min".to_string(),
                values: mins,
            },
            SemanticNode {
                id: "envelope.max".to_string(),
                values: maxs,
            },
        ],
    }
}

fn extract_i64_node_values(trace: &SemanticTrace, node_id: &str) -> Result<Vec<i64>, CliError> {
    let node = trace
        .nodes
        .iter()
        .find(|node| node.id == node_id)
        .ok_or_else(|| CliError::User(format!("missing semantic node {node_id}")))?;
    let mut out = Vec::with_capacity(node.values.len());
    for value in &node.values {
        let Some(parsed) = value.as_i64() else {
            return Err(CliError::User(format!("node {node_id} contains non-integer values")));
        };
        out.push(parsed);
    }
    Ok(out)
}

fn validate_loaded_artifact(
    target: &str,
    result: &ResultBlock,
    trace: &TraceArtifact,
    meta: &MetaArtifact,
    purpose: ArtifactPurpose,
) -> Result<(), CliError> {
    if result.command != meta.command {
        return Err(CliError::User(format!(
            "result/meta command mismatch for {target}: {} vs {}",
            result.command, meta.command
        )));
    }
    if result.target != meta.target {
        return Err(CliError::User(format!(
            "result/meta target mismatch for {target}: {} vs {}",
            result.target, meta.target
        )));
    }
    if result.mode != meta.mode {
        return Err(CliError::User(format!(
            "result/meta mode mismatch for {target}: {} vs {}",
            result.mode, meta.mode
        )));
    }
    if trace.schema != TRACE_SCHEMA {
        return Err(CliError::User(format!(
            "invalid trace schema for {target}: expected {TRACE_SCHEMA}, got {}",
            trace.schema
        )));
    }
    if meta.schema != META_SCHEMA {
        return Err(CliError::User(format!(
            "invalid meta schema for {target}: expected {META_SCHEMA}, got {}",
            meta.schema
        )));
    }
    if !matches!(meta.command.as_str(), "record" | "replay" | "diff" | "envelope") {
        return Err(CliError::User(format!(
            "invalid meta command for {target}: {}",
            meta.command
        )));
    }
    if !matches!(meta.mode.as_str(), "runtime_mode" | "mock" | "none") {
        return Err(CliError::User(format!(
            "invalid meta mode for {target}: {}",
            meta.mode
        )));
    }
    if meta.signal_input_count != trace.signal_inputs.len() {
        return Err(CliError::User(format!(
            "meta signal_input_count mismatch for {target}: {} vs {}",
            meta.signal_input_count,
            trace.signal_inputs.len()
        )));
    }
    let expected_len = trace.signal_inputs.len();
    for node in &trace.captured_trace.nodes {
        validate_node(node, expected_len, target)?;
    }
    if let Some(replay_trace) = &trace.replay_trace {
        for node in &replay_trace.nodes {
            validate_node(node, expected_len, target)?;
        }
    }
    validate_comparison_summary(target, trace.comparison.as_ref())?;
    if let Some(hash) = &meta.transient_rpl0_sha256 {
        if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(CliError::User(format!(
                "invalid transient_rpl0_sha256 in meta for {target}"
            )));
        }
    }
    match purpose {
        ArtifactPurpose::Replay => validate_replay_compatibility(target, trace, meta)?,
        ArtifactPurpose::Diff => validate_diff_compatibility(target, trace, meta)?,
        ArtifactPurpose::Envelope => validate_envelope_compatibility(target, trace, meta)?,
    }
    Ok(())
}

fn validate_authoritative_artifact_dir(base: &Path, target: &str) -> Result<(), CliError> {
    let metadata = fs::metadata(base).map_err(|err| match err.kind() {
        std::io::ErrorKind::NotFound => {
            CliError::User(format!("authoritative artifact directory not found for {target}"))
        }
        _ => CliError::Io(err),
    })?;
    if !metadata.is_dir() {
        return Err(CliError::User(format!(
            "authoritative replay requires a published artifact directory, got non-directory target {target}"
        )));
    }
    for name in ["result.txt", "trace.json", "meta.json"] {
        let path = base.join(name);
        let entry = fs::metadata(&path).map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => CliError::User(format!(
                "authoritative artifact directory missing required file {} for {target}",
                path.display()
            )),
            _ => CliError::Io(err),
        })?;
        if !entry.is_file() {
            return Err(CliError::User(format!(
                "authoritative artifact directory contains non-file entry {} for {target}",
                path.display()
            )));
        }
    }
    Ok(())
}

fn parse_published_result_block(
    target: &str,
    base: &Path,
    bytes: &[u8],
) -> Result<ResultBlock, CliError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|err| CliError::User(format!("invalid result.txt for {target}: {err}")))?;
    if !text.ends_with('\n') {
        return Err(CliError::User(format!(
            "invalid result.txt for {target}: missing trailing LF"
        )));
    }

    let lines: Vec<&str> = text.lines().collect();
    if lines.len() != 7 {
        return Err(CliError::User(format!(
            "invalid result.txt for {target}: expected 7 lines, found {}",
            lines.len()
        )));
    }

    let result = parse_result_line(target, lines[0], "RESULT: ")?;
    let command = parse_result_line(target, lines[1], "COMMAND: ")?;
    let result_target = parse_result_line(target, lines[2], "TARGET: ")?;
    let mode = parse_result_line(target, lines[3], "MODE: ")?;
    let equivalence = parse_result_line(target, lines[4], "EQUIVALENCE: ")?;
    let first_divergence = parse_result_line(target, lines[5], "FIRST_DIVERGENCE: ")?;
    let artifact = parse_result_line(target, lines[6], "ARTIFACT: ")?;

    if !matches!(result.as_str(), "PASS" | "FAIL") {
        return Err(CliError::User(format!(
            "invalid result.txt RESULT for {target}: {result}"
        )));
    }
    if !is_authoritative_command(&command) {
        return Err(CliError::User(format!(
            "invalid result.txt COMMAND for {target}: {command}"
        )));
    }
    if !matches!(mode.as_str(), "runtime_mode" | "mock" | "none") {
        return Err(CliError::User(format!(
            "invalid result.txt MODE for {target}: {mode}"
        )));
    }
    if !matches!(equivalence.as_str(), "exact" | "diverged") {
        return Err(CliError::User(format!(
            "invalid result.txt EQUIVALENCE for {target}: {equivalence}"
        )));
    }
    validate_result_block_invariants(target, &result, &equivalence, &first_divergence)?;
    validate_result_artifact_path(target, base, &artifact)?;

    Ok(ResultBlock {
        result,
        command,
        target: result_target,
        mode,
        equivalence,
        first_divergence,
        artifact,
    })
}

fn parse_result_line(target: &str, line: &str, prefix: &str) -> Result<String, CliError> {
    line.strip_prefix(prefix)
        .map(str::to_string)
        .ok_or_else(|| CliError::User(format!("invalid result.txt line for {target}: {line}")))
}

fn validate_result_block_invariants(
    target: &str,
    result: &str,
    equivalence: &str,
    first_divergence: &str,
) -> Result<(), CliError> {
    let is_pass = result == "PASS";
    let invariants_hold = if is_pass {
        equivalence == "exact" && first_divergence == "none"
    } else {
        equivalence == "diverged"
    };
    if !invariants_hold {
        return Err(CliError::User(format!(
            "invalid result.txt invariants for {target}"
        )));
    }
    Ok(())
}

fn validate_result_artifact_path(
    target: &str,
    base: &Path,
    artifact: &str,
) -> Result<(), CliError> {
    let Some((parent, run_id)) = artifact.split_once('/') else {
        return Err(CliError::User(format!(
            "invalid result.txt ARTIFACT path for {target}: {artifact}"
        )));
    };
    if parent != "artifacts" || run_id.is_empty() || run_id.contains('/') {
        return Err(CliError::User(format!(
            "invalid result.txt ARTIFACT path for {target}: {artifact}"
        )));
    }
    let dir_name = base.file_name().and_then(|name| name.to_str()).ok_or_else(|| {
        CliError::User(format!(
            "invalid authoritative artifact directory name for {target}"
        ))
    })?;
    if dir_name != run_id {
        return Err(CliError::User(format!(
            "result.txt ARTIFACT path does not match directory name for {target}: {artifact}"
        )));
    }
    Ok(())
}

fn validate_node(node: &SemanticNode, expected_len: usize, target: &str) -> Result<(), CliError> {
    if node.id.is_empty()
        || !node
            .id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b':' | b'-'))
    {
        return Err(CliError::User(format!(
            "invalid node id in authoritative trace for {target}: {}",
            node.id
        )));
    }
    if node.values.len() != expected_len {
        return Err(CliError::User(format!(
            "node length mismatch in authoritative trace for {target}: {} has {} values, expected {}",
            node.id,
            node.values.len(),
            expected_len
        )));
    }
    Ok(())
}

fn validate_comparison_summary(
    target: &str,
    comparison: Option<&ComparisonSummary>,
) -> Result<(), CliError> {
    let Some(comparison) = comparison else {
        return Ok(());
    };
    if !matches!(comparison.equivalence.as_str(), "exact" | "diverged") {
        return Err(CliError::User(format!(
            "invalid comparison equivalence in authoritative trace for {target}: {}",
            comparison.equivalence
        )));
    }
    if let Some(divergence) = &comparison.first_divergence {
        validate_divergence(target, divergence)?;
    }
    Ok(())
}

fn validate_divergence(target: &str, divergence: &FirstDivergence) -> Result<(), CliError> {
    if divergence.node.is_empty()
        || !divergence
            .node
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b':' | b'-'))
    {
        return Err(CliError::User(format!(
            "invalid first_divergence node in authoritative trace for {target}: {}",
            divergence.node
        )));
    }
    if !matches!(
        divergence.cause.as_str(),
        "VAL_MISMATCH" | "TYPE_MISMATCH" | "OOB"
    ) {
        return Err(CliError::User(format!(
            "invalid first_divergence cause in authoritative trace for {target}: {}",
            divergence.cause
        )));
    }
    Ok(())
}

fn validate_replay_compatibility(
    target: &str,
    trace: &TraceArtifact,
    meta: &MetaArtifact,
) -> Result<(), CliError> {
    if !matches!(meta.command.as_str(), "record" | "replay") {
        return Err(CliError::User(format!(
            "artifact incompatible with replay for {target}: command {}",
            meta.command
        )));
    }
    if trace.signal_inputs.is_empty() {
        return Err(CliError::User(format!(
            "artifact incompatible with replay for {target}: missing signal inputs"
        )));
    }
    require_node(trace, "signal.interval_us", target, "replay")?;
    require_node(trace, "dpw4.square", target, "replay")?;
    require_node(trace, "dpw4.hash", target, "replay")?;
    Ok(())
}

fn validate_diff_compatibility(
    target: &str,
    trace: &TraceArtifact,
    meta: &MetaArtifact,
) -> Result<(), CliError> {
    if !matches!(meta.command.as_str(), "record" | "replay" | "envelope" | "diff") {
        return Err(CliError::User(format!(
            "artifact incompatible with diff for {target}: command {}",
            meta.command
        )));
    }
    if meta.command == "diff" {
        validate_comparison_summary(target, trace.comparison.as_ref())?;
        return Ok(());
    }
    if trace.captured_trace.nodes.is_empty() {
        return Err(CliError::User(format!(
            "artifact incompatible with diff for {target}: missing captured trace"
        )));
    }
    Ok(())
}

fn validate_envelope_compatibility(
    target: &str,
    trace: &TraceArtifact,
    meta: &MetaArtifact,
) -> Result<(), CliError> {
    if !matches!(meta.command.as_str(), "record" | "replay") {
        return Err(CliError::User(format!(
            "artifact incompatible with envelope for {target}: command {}",
            meta.command
        )));
    }
    require_node(trace, "dpw4.square", target, "envelope")?;
    Ok(())
}

fn require_node(
    trace: &TraceArtifact,
    node_id: &str,
    target: &str,
    command: &str,
) -> Result<(), CliError> {
    if trace.captured_trace.nodes.iter().any(|node| node.id == node_id) {
        Ok(())
    } else {
        Err(CliError::User(format!(
            "artifact incompatible with {command} for {target}: missing node {node_id}"
        )))
    }
}

fn encode_diff_target(left: &str, right: &str) -> Result<String, CliError> {
    serde_json::to_string(&vec![left, right])
        .map_err(|err| CliError::Integrity(format!("diff target encoding failed: {err}")))
}

fn json_type_tag(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn acquire_record_capture(target: &str) -> Result<RecordCapture, CliError> {
    let intervals = if target.starts_with("fixture://") {
        fixture_intervals()
    } else if target.ends_with(".csv") || Path::new(target).is_file() {
        parse_interval_csv_text(&fs::read_to_string(target)?)
            .map_err(CliError::User)?
    } else {
        capture_serial_intervals(target)?
    };

    let transient_rpl0 = build_transient_rpl0(&intervals);
    let transient_rpl0_sha256 = hex::encode(Sha256::digest(&transient_rpl0));
    let source_kind = if target.starts_with("fixture://") {
        "fixture".to_string()
    } else if target.ends_with(".csv") || Path::new(target).is_file() {
        "interval_csv".to_string()
    } else {
        "serial_capture".to_string()
    };

    Ok(RecordCapture {
        intervals,
        source_kind,
        transient_rpl0_sha256,
    })
}

fn capture_serial_intervals(target: &str) -> Result<Vec<u32>, CliError> {
    let temp_csv = serial_capture_temp_path()?;
    let script_path = repo_root()?.join("scripts/csv_capture.py");
    let output = Command::new("python3")
        .arg(script_path)
        .arg("--serial")
        .arg(target)
        .arg("--out")
        .arg(&temp_csv)
        .arg("--reset-mode")
        .arg("manual")
        .output()?;

    let mut stderr = std::io::stderr().lock();
    stderr.write_all(&output.stdout)?;
    stderr.write_all(&output.stderr)?;
    stderr.flush()?;

    if !output.status.success() {
        let _ = fs::remove_file(&temp_csv);
        return Err(CliError::User(format!("serial capture failed for {target}")));
    }

    let csv_text = fs::read_to_string(&temp_csv)?;
    let _ = fs::remove_file(&temp_csv);
    parse_interval_csv_text(&csv_text).map_err(CliError::User)
}

fn parse_interval_csv_text(text: &str) -> Result<Vec<u32>, String> {
    let mut lines = text.lines();
    let Some(header) = lines.next() else {
        return Err("empty csv".to_string());
    };
    if header != "index,interval_us" {
        return Err("invalid header, expected index,interval_us".to_string());
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
            return Err(format!("interval must be > 0 at row {}", expected_index + 1));
        }
        intervals.push(interval);
    }

    if intervals.len() != INTERVAL_ROW_COUNT {
        return Err(format!(
            "expected {INTERVAL_ROW_COUNT} interval rows, found {}",
            intervals.len()
        ));
    }
    Ok(intervals)
}

fn fixture_intervals() -> Vec<u32> {
    (0..INTERVAL_ROW_COUNT)
        .map(|idx| if idx == 0 { 305_564 } else { 304_000 })
        .collect()
}

fn build_transient_rpl0(intervals: &[u32]) -> Vec<u8> {
    let empty_schema = b"";
    let schema_hash = Sha256::digest(empty_schema);
    let build_hash = Sha256::digest(TRANSIENT_BUILD_HASH_INPUT);
    let config_hash = Sha256::digest(TRANSIENT_CONFIG_HASH_INPUT);
    let mut header = vec![0u8; TRANSIENT_HEADER_SIZE];
    header[0..4].copy_from_slice(b"RPL0");
    header[4..6].copy_from_slice(&1u16.to_le_bytes());
    header[6..8].copy_from_slice(&(TRANSIENT_HEADER_SIZE as u16).to_le_bytes());
    header[8..12].copy_from_slice(&(TRANSIENT_FRAME_COUNT as u32).to_le_bytes());
    header[12..14].copy_from_slice(&(TRANSIENT_FRAME_SIZE as u16).to_le_bytes());
    header[14..16].copy_from_slice(&0u16.to_le_bytes());
    header[16..20].copy_from_slice(&(empty_schema.len() as u32).to_le_bytes());
    header[20..52].copy_from_slice(&schema_hash);
    header[52..84].copy_from_slice(&build_hash);
    header[84..116].copy_from_slice(&config_hash);
    header[116..132].copy_from_slice(b"precision-runner");
    header[132..148].copy_from_slice(b"offline-fixed-v1");
    header[148..150].copy_from_slice(&TRANSIENT_CAPTURE_BOUNDARY.to_le_bytes());
    header[150..152].copy_from_slice(&0u16.to_le_bytes());

    let mut out =
        Vec::with_capacity(TRANSIENT_HEADER_SIZE + TRANSIENT_FRAME_COUNT * TRANSIENT_FRAME_SIZE);
    out.extend_from_slice(&header);
    for frame_idx in 0..TRANSIENT_FRAME_COUNT {
        let input_sample = if frame_idx < intervals.len() {
            intervals[frame_idx] as i32
        } else {
            0
        };
        out.extend_from_slice(&(frame_idx as u32).to_le_bytes());
        out.push(0x02);
        out.push(0);
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&1000u32.to_le_bytes());
        out.extend_from_slice(&input_sample.to_le_bytes());
    }
    out
}

fn mix_trace_hash(acc: u64, step: u64, interval: u64, sample: i64) -> u64 {
    let mut x = acc
        ^ step.rotate_left(7)
        ^ interval.rotate_left(17)
        ^ (sample as u64).rotate_left(29);
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

fn now_unix_seconds() -> Result<u64, CliError> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| CliError::Integrity(format!("clock error: {err}")))?
        .as_secs())
}

fn repo_root() -> Result<PathBuf, CliError> {
    let root = std::env::var_os("PRECISION_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    root
        .canonicalize()
        .map_err(CliError::Io)
}

fn serial_capture_temp_path() -> Result<PathBuf, CliError> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| CliError::Integrity(format!("clock error: {err}")))?
        .as_nanos();
    Ok(std::env::temp_dir().join(format!("precision-capture-{nanos}.csv")))
}
