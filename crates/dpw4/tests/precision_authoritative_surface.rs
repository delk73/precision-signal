#![cfg(feature = "cli")]

use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_root(label: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time must be after unix epoch")
        .as_nanos();
    let temp_root = std::env::temp_dir().join(format!("{label}-{unique}"));
    fs::create_dir_all(&temp_root).expect("temp root should be created");
    temp_root
}

fn artifact_path_from_stdout(stdout: &[u8]) -> String {
    String::from_utf8(stdout.to_vec())
        .expect("stdout must be utf8")
        .lines()
        .find(|line| line.starts_with("ARTIFACT: "))
        .and_then(|line| line.strip_prefix("ARTIFACT: "))
        .expect("artifact path")
        .to_string()
}

fn make_replay_artifact(temp_root: &std::path::Path) -> (String, String) {
    let record = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(temp_root)
        .args(["record", "fixture://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record should run");
    assert!(record.status.success(), "stderr: {}", String::from_utf8_lossy(&record.stderr));
    let recorded = artifact_path_from_stdout(&record.stdout);

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(temp_root)
        .args(["replay", &recorded, "--mode", "runtime_mode"])
        .output()
        .expect("precision replay should run");
    assert!(replay.status.success(), "stderr: {}", String::from_utf8_lossy(&replay.stderr));
    let replayed = artifact_path_from_stdout(&replay.stdout);
    (recorded, replayed)
}

fn assert_stdout_has_exactly_seven_lines(stdout: &[u8]) {
    let stdout = String::from_utf8(stdout.to_vec()).expect("stdout must be utf8");
    assert_eq!(stdout.lines().count(), 7, "stdout must contain exactly 7 lines");
}

#[test]
fn precision_help_only_lists_authoritative_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_precision"))
        .arg("--help")
        .output()
        .expect("precision --help should run");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let help = String::from_utf8_lossy(&output.stdout);

    assert!(help.contains("record"));
    assert!(help.contains("replay"));
    assert!(help.contains("diff"));
    assert!(help.contains("envelope"));
    assert!(!help.contains("generate"));
    assert!(!help.contains("inspect"));
    assert!(!help.contains("verify"));
    assert!(!help.contains("artifacts"));
    assert!(!help.contains("validate"));
}

#[test]
fn precision_without_args_exits_2_and_keeps_stdout_empty() {
    let output = Command::new(env!("CARGO_BIN_EXE_precision"))
        .output()
        .expect("bare precision should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        stderr,
        "usage: precision <command>\ncommands:\n  record\n  replay\n  diff\n  envelope\n"
    );
}

#[test]
fn precision_unknown_command_exits_2_with_minimal_usage() {
    let output = Command::new(env!("CARGO_BIN_EXE_precision"))
        .arg("generate")
        .output()
        .expect("precision unknown command should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        stderr,
        "unknown command\nusage: precision <command>\ncommands:\n  record\n  replay\n  diff\n  envelope\n"
    );
}

#[test]
fn precision_version_writes_to_stdout() {
    let output = Command::new(env!("CARGO_BIN_EXE_precision"))
        .arg("--version")
        .output()
        .expect("precision --version should run");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("precision {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn precision_record_publishes_real_artifact_and_stdout_matches_result_file() {
    let temp_root = unique_temp_root("precision-authoritative");

    let output = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["record", "fixture://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record should run");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(output.stderr.is_empty(), "authoritative stub must not emit stderr");

    let stdout_bytes = output.stdout;
    let stdout = String::from_utf8(stdout_bytes.clone()).expect("stdout must be utf-8");
    let artifact_line = stdout
        .lines()
        .find(|line| line.starts_with("ARTIFACT: "))
        .expect("result block must contain artifact line");
    let artifact_rel = artifact_line
        .strip_prefix("ARTIFACT: ")
        .expect("artifact line prefix");
    assert_ne!(artifact_rel, "artifacts/PLACEHOLDER");

    let artifact_dir = temp_root.join(artifact_rel);
    assert!(artifact_dir.is_dir(), "artifact dir must exist at {}", artifact_dir.display());
    assert_stdout_has_exactly_seven_lines(&stdout_bytes);

    let result_txt = fs::read(artifact_dir.join("result.txt")).expect("result.txt must exist");
    assert_eq!(result_txt, stdout_bytes);

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_replay_passes_for_known_good_capture() {
    let temp_root = unique_temp_root("precision-replay-pass");

    let record = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["record", "fixture://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record should run");
    assert!(record.status.success(), "stderr: {}", String::from_utf8_lossy(&record.stderr));

    let record_stdout = String::from_utf8(record.stdout).expect("record stdout must be utf8");
    let artifact_rel = record_stdout
        .lines()
        .find(|line| line.starts_with("ARTIFACT: "))
        .and_then(|line| line.strip_prefix("ARTIFACT: "))
        .expect("record result must include artifact path");

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["replay", artifact_rel, "--mode", "runtime_mode"])
        .output()
        .expect("precision replay should run");

    assert_eq!(replay.status.code(), Some(0), "stderr: {}", String::from_utf8_lossy(&replay.stderr));
    assert!(replay.stderr.is_empty(), "unexpected stderr: {}", String::from_utf8_lossy(&replay.stderr));
    let stdout = String::from_utf8(replay.stdout).expect("replay stdout must be utf8");
    assert!(stdout.contains("RESULT: PASS\n"));
    assert!(stdout.contains("EQUIVALENCE: exact\n"));
    assert!(stdout.contains("FIRST_DIVERGENCE: none\n"));
    assert_stdout_has_exactly_seven_lines(stdout.as_bytes());

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_diff_reports_semantic_divergence_for_perturbed_capture() {
    let temp_root = unique_temp_root("precision-diff-fail");

    let record_a = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["record", "fixture://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record A should run");
    assert!(record_a.status.success(), "stderr: {}", String::from_utf8_lossy(&record_a.stderr));

    let record_b = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["record", "fixture://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record B should run");
    assert!(record_b.status.success(), "stderr: {}", String::from_utf8_lossy(&record_b.stderr));

    let artifact_a = String::from_utf8(record_a.stdout)
        .expect("record A stdout must be utf8")
        .lines()
        .find(|line| line.starts_with("ARTIFACT: "))
        .and_then(|line| line.strip_prefix("ARTIFACT: "))
        .expect("record A artifact path")
        .to_string();
    let artifact_b = String::from_utf8(record_b.stdout)
        .expect("record B stdout must be utf8")
        .lines()
        .find(|line| line.starts_with("ARTIFACT: "))
        .and_then(|line| line.strip_prefix("ARTIFACT: "))
        .expect("record B artifact path")
        .to_string();

    let trace_path = temp_root.join(&artifact_b).join("trace.json");
    let original = fs::read_to_string(&trace_path).expect("trace.json must exist");
    let perturbed = original.replacen("304000", "304001", 1);
    fs::write(&trace_path, perturbed).expect("perturbed trace must be written");

    let diff = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["diff", &artifact_a, &artifact_b, "--mode", "runtime_mode"])
        .output()
        .expect("precision diff should run");

    assert_eq!(diff.status.code(), Some(1), "stderr: {}", String::from_utf8_lossy(&diff.stderr));
    assert!(diff.stderr.is_empty(), "unexpected stderr: {}", String::from_utf8_lossy(&diff.stderr));
    let stdout = String::from_utf8(diff.stdout).expect("diff stdout must be utf8");
    assert!(stdout.contains("RESULT: FAIL\n"));
    assert!(stdout.contains("EQUIVALENCE: diverged\n"));
    assert!(stdout.contains("FIRST_DIVERGENCE: step=1 node=artifact.signal_inputs cause=VAL_MISMATCH\n"));
    assert_stdout_has_exactly_seven_lines(stdout.as_bytes());

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_replay_rejects_invalid_meta_schema_with_exit_2() {
    let temp_root = unique_temp_root("precision-replay-invalid-meta");

    let record = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["record", "fixture://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record should run");
    assert!(record.status.success(), "stderr: {}", String::from_utf8_lossy(&record.stderr));

    let artifact_rel = String::from_utf8(record.stdout)
        .expect("record stdout must be utf8")
        .lines()
        .find(|line| line.starts_with("ARTIFACT: "))
        .and_then(|line| line.strip_prefix("ARTIFACT: "))
        .expect("artifact path")
        .to_string();

    let meta_path = temp_root.join(&artifact_rel).join("meta.json");
    let original = fs::read_to_string(&meta_path).expect("meta.json must exist");
    let invalid = original.replacen("\"precision.meta.v1\"", "\"broken.meta.v1\"", 1);
    fs::write(&meta_path, invalid).expect("invalid meta must be written");

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["replay", &artifact_rel, "--mode", "runtime_mode"])
        .output()
        .expect("precision replay should run");

    assert_eq!(replay.status.code(), Some(2));
    assert!(replay.stdout.is_empty(), "stdout must be suppressed on load failure");
    let stderr = String::from_utf8(replay.stderr).expect("stderr must be utf8");
    assert!(stderr.contains("ERROR: invalid meta schema"));

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_replay_rejects_invalid_trace_schema_with_exit_2() {
    let temp_root = unique_temp_root("precision-replay-invalid-trace");
    let record = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["record", "fixture://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record should run");
    let artifact_rel = artifact_path_from_stdout(&record.stdout);

    let trace_path = temp_root.join(&artifact_rel).join("trace.json");
    let original = fs::read_to_string(&trace_path).expect("trace.json must exist");
    let invalid = original.replacen("\"precision.trace.v1\"", "\"broken.trace.v1\"", 1);
    fs::write(&trace_path, invalid).expect("invalid trace must be written");

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["replay", &artifact_rel, "--mode", "runtime_mode"])
        .output()
        .expect("precision replay should run");

    assert_eq!(replay.status.code(), Some(2));
    assert!(replay.stdout.is_empty());
    let stderr = String::from_utf8(replay.stderr).expect("stderr must be utf8");
    assert!(stderr.contains("ERROR: invalid trace schema"));

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_replay_rejects_malformed_comparison_payload_with_exit_2() {
    let temp_root = unique_temp_root("precision-replay-invalid-comparison");
    let (_, replay_artifact) = make_replay_artifact(&temp_root);

    let trace_path = temp_root.join(&replay_artifact).join("trace.json");
    let original = fs::read_to_string(&trace_path).expect("trace.json must exist");
    let invalid = original.replacen("\"equivalence\": \"exact\"", "\"equivalence\": \"maybe\"", 1);
    fs::write(&trace_path, invalid).expect("invalid trace must be written");

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["replay", &replay_artifact, "--mode", "runtime_mode"])
        .output()
        .expect("precision replay should run");

    assert_eq!(replay.status.code(), Some(2));
    assert!(replay.stdout.is_empty());
    let stderr = String::from_utf8(replay.stderr).expect("stderr must be utf8");
    assert!(stderr.contains("ERROR: invalid comparison equivalence"));

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_replay_rejects_invalid_divergence_cause_with_exit_2() {
    let temp_root = unique_temp_root("precision-replay-invalid-divergence");

    let record_a = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["record", "fixture://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record A should run");
    let record_b = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["record", "fixture://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record B should run");
    let artifact_a = artifact_path_from_stdout(&record_a.stdout);
    let artifact_b = artifact_path_from_stdout(&record_b.stdout);

    let trace_path = temp_root.join(&artifact_b).join("trace.json");
    let original = fs::read_to_string(&trace_path).expect("trace.json must exist");
    let perturbed = original.replacen("304000", "304001", 1);
    fs::write(&trace_path, perturbed).expect("perturbed trace must be written");

    let diff = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["diff", &artifact_a, &artifact_b, "--mode", "runtime_mode"])
        .output()
        .expect("precision diff should run");
    let diff_artifact = artifact_path_from_stdout(&diff.stdout);

    let diff_trace = temp_root.join(&diff_artifact).join("trace.json");
    let diff_original = fs::read_to_string(&diff_trace).expect("diff trace must exist");
    let invalid = diff_original.replacen(
        "\"cause\": \"VAL_MISMATCH\"",
        "\"cause\": \"NOT_A_CAUSE\"",
        1,
    );
    fs::write(&diff_trace, invalid).expect("invalid diff trace must be written");

    let diff_reload = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["diff", &diff_artifact, &diff_artifact, "--mode", "runtime_mode"])
        .output()
        .expect("precision diff reload should run");

    assert_eq!(diff_reload.status.code(), Some(2));
    assert!(diff_reload.stdout.is_empty());
    let stderr = String::from_utf8(diff_reload.stderr).expect("stderr must be utf8");
    assert!(stderr.contains("ERROR: invalid first_divergence cause"));

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_envelope_passes_for_known_good_capture() {
    let temp_root = unique_temp_root("precision-envelope-pass");

    let record = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["record", "fixture://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record should run");
    assert!(record.status.success(), "stderr: {}", String::from_utf8_lossy(&record.stderr));

    let artifact_rel = String::from_utf8(record.stdout)
        .expect("record stdout must be utf8")
        .lines()
        .find(|line| line.starts_with("ARTIFACT: "))
        .and_then(|line| line.strip_prefix("ARTIFACT: "))
        .expect("artifact path")
        .to_string();

    let envelope = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["envelope", &artifact_rel, "--mode", "runtime_mode"])
        .output()
        .expect("precision envelope should run");

    assert_eq!(envelope.status.code(), Some(0));
    assert!(envelope.stderr.is_empty(), "unexpected stderr: {}", String::from_utf8_lossy(&envelope.stderr));
    let stdout = String::from_utf8(envelope.stdout).expect("envelope stdout must be utf8");
    assert!(stdout.contains("RESULT: PASS\n"));
    assert!(stdout.contains("COMMAND: envelope\n"));
    assert!(stdout.contains("EQUIVALENCE: exact\n"));
    assert_stdout_has_exactly_seven_lines(stdout.as_bytes());

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_diff_target_encoding_is_stable_across_runs() {
    let temp_root = unique_temp_root("precision-diff-target-stable");

    let record_a = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["record", "fixture://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record A should run");
    let record_b = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["record", "fixture://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record B should run");

    let artifact_a = artifact_path_from_stdout(&record_a.stdout);
    let artifact_b = artifact_path_from_stdout(&record_b.stdout);

    let first = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["diff", &artifact_a, &artifact_b, "--mode", "runtime_mode"])
        .output()
        .expect("first diff should run");
    let second = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["diff", &artifact_a, &artifact_b, "--mode", "runtime_mode"])
        .output()
        .expect("second diff should run");

    assert_eq!(first.status.code(), second.status.code());
    let first_stdout = String::from_utf8(first.stdout).expect("first stdout must be utf8");
    let second_stdout = String::from_utf8(second.stdout).expect("second stdout must be utf8");
    let first_target = first_stdout
        .lines()
        .find(|line| line.starts_with("TARGET: "))
        .expect("first target line");
    let second_target = second_stdout
        .lines()
        .find(|line| line.starts_with("TARGET: "))
        .expect("second target line");
    assert_eq!(first_target, second_target);
    assert_eq!(
        first_target,
        format!("TARGET: [\"{}\",\"{}\"]", artifact_a, artifact_b)
    );

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_replay_rejects_incompatible_diff_artifact_with_exit_2() {
    let temp_root = unique_temp_root("precision-replay-incompatible");

    let record_a = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["record", "fixture://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record A should run");
    let record_b = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["record", "fixture://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record B should run");
    let artifact_a = artifact_path_from_stdout(&record_a.stdout);
    let artifact_b = artifact_path_from_stdout(&record_b.stdout);

    let diff = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["diff", &artifact_a, &artifact_b, "--mode", "runtime_mode"])
        .output()
        .expect("precision diff should run");
    assert_eq!(diff.status.code(), Some(0));
    let diff_artifact = artifact_path_from_stdout(&diff.stdout);

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["replay", &diff_artifact, "--mode", "runtime_mode"])
        .output()
        .expect("precision replay should run");

    assert_eq!(replay.status.code(), Some(2));
    assert!(replay.stdout.is_empty());
    let stderr = String::from_utf8(replay.stderr).expect("stderr must be utf8");
    assert!(stderr.contains("artifact incompatible with replay"));

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_record_serial_failure_returns_exit_2_without_stdout() {
    let temp_root = unique_temp_root("precision-record-serial-failure");

    let output = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["record", "/dev/does-not-exist-precision", "--mode", "runtime_mode"])
        .output()
        .expect("precision record should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("stderr must be utf8");
    assert!(stderr.contains("ERROR: serial capture failed"));

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_record_helper_failure_returns_exit_2_without_stdout() {
    let temp_root = unique_temp_root("precision-record-helper-failure");

    let output = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .env("PRECISION_REPO_ROOT", "/definitely/missing/precision-root")
        .args(["record", "serial://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("stderr must be utf8");
    assert!(stderr.contains("ERROR:"));

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_diff_artifact_is_comparison_only_and_reloads_for_diff_only() {
    let temp_root = unique_temp_root("precision-diff-artifact-shape");

    let record_a = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["record", "fixture://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record A should run");
    let record_b = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["record", "fixture://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record B should run");
    let artifact_a = artifact_path_from_stdout(&record_a.stdout);
    let artifact_b = artifact_path_from_stdout(&record_b.stdout);

    let diff = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["diff", &artifact_a, &artifact_b, "--mode", "runtime_mode"])
        .output()
        .expect("precision diff should run");
    let diff_artifact = artifact_path_from_stdout(&diff.stdout);

    let diff_trace = fs::read_to_string(temp_root.join(&diff_artifact).join("trace.json"))
        .expect("diff trace must exist");
    assert!(diff_trace.contains("\"signal_inputs\": []"));
    assert!(diff_trace.contains("\"captured_trace\": {\n    \"nodes\": []\n  }"));

    let diff_reload = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["diff", &diff_artifact, &diff_artifact, "--mode", "runtime_mode"])
        .output()
        .expect("precision diff reload should run");
    assert_eq!(diff_reload.status.code(), Some(0));
    assert_stdout_has_exactly_seven_lines(&diff_reload.stdout);

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["replay", &diff_artifact, "--mode", "runtime_mode"])
        .output()
        .expect("precision replay should run");
    assert_eq!(replay.status.code(), Some(2));

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_envelope_rejects_malformed_source_nodes_with_exit_2() {
    let temp_root = unique_temp_root("precision-envelope-malformed");

    let record = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["record", "fixture://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record should run");
    let artifact_rel = artifact_path_from_stdout(&record.stdout);

    let trace_path = temp_root.join(&artifact_rel).join("trace.json");
    let original = fs::read_to_string(&trace_path).expect("trace.json must exist");
    let malformed = original.replace("\"dpw4.square\"", "\"dpw4.square.broken\"");
    fs::write(&trace_path, malformed).expect("malformed trace must be written");

    let envelope = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["envelope", &artifact_rel, "--mode", "runtime_mode"])
        .output()
        .expect("precision envelope should run");

    assert_eq!(envelope.status.code(), Some(2));
    assert!(envelope.stdout.is_empty());
    let stderr = String::from_utf8(envelope.stderr).expect("stderr must be utf8");
    assert!(stderr.contains("artifact incompatible with envelope"));

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn sig_util_help_starts_with_non_authoritative_warning() {
    let output = Command::new(env!("CARGO_BIN_EXE_sig-util"))
        .arg("--help")
        .output()
        .expect("sig-util --help should run");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let help = String::from_utf8_lossy(&output.stdout);

    assert!(help.starts_with(
        "NON-AUTHORITATIVE: This utility exists outside the 1.6.0 high-integrity contract."
    ));
}

#[test]
fn sig_util_without_args_exits_2_with_usage_on_stderr() {
    let output = Command::new(env!("CARGO_BIN_EXE_sig-util"))
        .output()
        .expect("bare sig-util should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "usage: sig-util <command>\ncommands:\n  generate\n  inspect\n  verify\n  artifacts\n  validate\n  header-audit\n"
    );
}
