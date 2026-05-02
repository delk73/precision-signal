#![cfg(feature = "cli")]

use serde_json::Value;
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
    assert!(
        record.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&record.stderr)
    );
    let recorded = artifact_path_from_stdout(&record.stdout);

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(temp_root)
        .args(["replay", &recorded, "--mode", "runtime_mode"])
        .output()
        .expect("precision replay should run");
    assert!(
        replay.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&replay.stderr)
    );
    let replayed = artifact_path_from_stdout(&replay.stdout);
    (recorded, replayed)
}

fn make_record_artifact(temp_root: &std::path::Path) -> String {
    let record = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(temp_root)
        .args(["record", "fixture://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record should run");
    assert!(
        record.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&record.stderr)
    );
    artifact_path_from_stdout(&record.stdout)
}

fn assert_stdout_has_exactly_seven_lines(stdout: &[u8]) {
    let stdout = String::from_utf8(stdout.to_vec()).expect("stdout must be utf8");
    assert_eq!(
        stdout.lines().count(),
        7,
        "stdout must contain exactly 7 lines"
    );
}

fn scrub_volatile_result_block_fields(stdout: &[u8]) -> String {
    String::from_utf8(stdout.to_vec())
        .expect("stdout must be utf8")
        .lines()
        .map(|line| {
            if line.starts_with("ARTIFACT: ") {
                "ARTIFACT: <volatile>".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn read_json(path: &std::path::Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).expect("json file must exist"))
        .expect("json must parse")
}

fn write_json(path: &std::path::Path, value: &Value) {
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("json should serialize"),
    )
    .expect("json should be written");
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

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "authoritative stub must not emit stderr"
    );

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
    assert!(
        artifact_dir.is_dir(),
        "artifact dir must exist at {}",
        artifact_dir.display()
    );
    assert_stdout_has_exactly_seven_lines(&stdout_bytes);

    let meta = read_json(&artifact_dir.join("meta.json"));
    assert_eq!(
        meta["schema"],
        Value::from("precision.meta.v2"),
        "record output must emit the v2 meta schema"
    );
    assert!(
        meta.get("transient_rpl0_payload_sha256").is_some(),
        "record output must name the transient payload hash explicitly"
    );
    assert!(
        meta.get("transient_rpl0_sha256").is_none(),
        "record output must not emit the ambiguous legacy field name"
    );

    let result_txt = fs::read(artifact_dir.join("result.txt")).expect("result.txt must exist");
    assert_eq!(result_txt, stdout_bytes);

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_replay_accepts_v1_legacy_transient_rpl0_sha256_field() {
    let temp_root = unique_temp_root("precision-replay-v1-legacy-transient-hash");
    let artifact_rel = make_record_artifact(&temp_root);
    let meta_path = temp_root.join(&artifact_rel).join("meta.json");
    let mut meta = read_json(&meta_path);

    let payload_hash = meta
        .as_object_mut()
        .and_then(|obj| obj.remove("transient_rpl0_payload_sha256"))
        .expect("new payload hash field must exist in meta.json");
    meta["schema"] = Value::from("precision.meta.v1");
    meta["transient_rpl0_sha256"] = payload_hash;
    write_json(&meta_path, &meta);

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["replay", &artifact_rel, "--mode", "runtime_mode"])
        .output()
        .expect("precision replay should run");

    assert!(
        replay.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&replay.stderr)
    );

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_replay_rejects_v2_legacy_transient_rpl0_sha256_field() {
    let temp_root = unique_temp_root("precision-replay-v2-legacy-transient-hash");
    let artifact_rel = make_record_artifact(&temp_root);
    let meta_path = temp_root.join(&artifact_rel).join("meta.json");
    let mut meta = read_json(&meta_path);

    let payload_hash = meta
        .as_object_mut()
        .and_then(|obj| obj.remove("transient_rpl0_payload_sha256"))
        .expect("new payload hash field must exist in meta.json");
    meta["transient_rpl0_sha256"] = payload_hash;
    write_json(&meta_path, &meta);

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["replay", &artifact_rel, "--mode", "runtime_mode"])
        .output()
        .expect("precision replay should run");

    assert_eq!(replay.status.code(), Some(2));
    assert!(replay.stdout.is_empty());
    let stderr = String::from_utf8(replay.stderr).expect("stderr must be utf8");
    assert!(stderr.contains("ERROR: invalid authoritative meta"));

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_replay_rejects_v1_payload_sha256_field() {
    let temp_root = unique_temp_root("precision-replay-v1-payload-hash");
    let artifact_rel = make_record_artifact(&temp_root);
    let meta_path = temp_root.join(&artifact_rel).join("meta.json");
    let mut meta = read_json(&meta_path);

    meta["schema"] = Value::from("precision.meta.v1");
    write_json(&meta_path, &meta);

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["replay", &artifact_rel, "--mode", "runtime_mode"])
        .output()
        .expect("precision replay should run");

    assert_eq!(replay.status.code(), Some(2));
    assert!(replay.stdout.is_empty());
    let stderr = String::from_utf8(replay.stderr).expect("stderr must be utf8");
    assert!(stderr.contains("ERROR: invalid authoritative meta"));

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
    assert!(
        record.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&record.stderr)
    );

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

    assert_eq!(
        replay.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&replay.stderr)
    );
    assert!(
        replay.stderr.is_empty(),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&replay.stderr)
    );
    let stdout = String::from_utf8(replay.stdout).expect("replay stdout must be utf8");
    assert!(stdout.contains("RESULT: PASS\n"));
    assert!(stdout.contains("EQUIVALENCE: exact\n"));
    assert!(stdout.contains("FIRST_DIVERGENCE: none\n"));
    assert_stdout_has_exactly_seven_lines(stdout.as_bytes());

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_replay_repeated_runs_are_deterministic_for_same_artifact_directory() {
    let temp_root = unique_temp_root("precision-replay-deterministic");
    let recorded = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["record", "fixture://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record should run");
    assert!(
        recorded.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&recorded.stderr)
    );
    let artifact_rel = artifact_path_from_stdout(&recorded.stdout);

    let first = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["replay", &artifact_rel, "--mode", "runtime_mode"])
        .output()
        .expect("first replay should run");
    let second = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["replay", &artifact_rel, "--mode", "runtime_mode"])
        .output()
        .expect("second replay should run");

    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert!(first.stderr.is_empty());
    assert!(second.stderr.is_empty());
    assert_eq!(
        scrub_volatile_result_block_fields(&first.stdout),
        scrub_volatile_result_block_fields(&second.stdout)
    );

    let first_artifact = artifact_path_from_stdout(&first.stdout);
    let second_artifact = artifact_path_from_stdout(&second.stdout);
    let first_trace =
        fs::read(temp_root.join(first_artifact).join("trace.json")).expect("first trace.json");
    let second_trace =
        fs::read(temp_root.join(second_artifact).join("trace.json")).expect("second trace.json");
    assert_eq!(first_trace, second_trace);

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
    assert!(
        record_a.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&record_a.stderr)
    );

    let record_b = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["record", "fixture://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record B should run");
    assert!(
        record_b.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&record_b.stderr)
    );

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

    assert_eq!(
        diff.status.code(),
        Some(1),
        "stderr: {}",
        String::from_utf8_lossy(&diff.stderr)
    );
    assert!(
        diff.stderr.is_empty(),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&diff.stderr)
    );
    let stdout = String::from_utf8(diff.stdout).expect("diff stdout must be utf8");
    assert!(stdout.contains("RESULT: FAIL\n"));
    assert!(stdout.contains("EQUIVALENCE: diverged\n"));
    assert!(stdout
        .contains("FIRST_DIVERGENCE: step=1 node=artifact.signal_inputs cause=VAL_MISMATCH\n"));
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
    assert!(
        record.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&record.stderr)
    );

    let artifact_rel = String::from_utf8(record.stdout)
        .expect("record stdout must be utf8")
        .lines()
        .find(|line| line.starts_with("ARTIFACT: "))
        .and_then(|line| line.strip_prefix("ARTIFACT: "))
        .expect("artifact path")
        .to_string();

    let meta_path = temp_root.join(&artifact_rel).join("meta.json");
    let original = fs::read_to_string(&meta_path).expect("meta.json must exist");
    let invalid = original.replacen("\"precision.meta.v2\"", "\"broken.meta.v2\"", 1);
    fs::write(&meta_path, invalid).expect("invalid meta must be written");

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["replay", &artifact_rel, "--mode", "runtime_mode"])
        .output()
        .expect("precision replay should run");

    assert_eq!(replay.status.code(), Some(2));
    assert!(
        replay.stdout.is_empty(),
        "stdout must be suppressed on load failure"
    );
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
fn precision_replay_rejects_missing_required_artifact_files_with_exit_2() {
    for missing in ["result.txt", "trace.json", "meta.json"] {
        let temp_root = unique_temp_root(&format!("precision-replay-missing-{missing}"));
        let artifact_rel = make_record_artifact(&temp_root);
        fs::remove_file(temp_root.join(&artifact_rel).join(missing))
            .expect("required file should be removed");

        let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
            .current_dir(&temp_root)
            .args(["replay", &artifact_rel, "--mode", "runtime_mode"])
            .output()
            .expect("precision replay should run");

        assert_eq!(replay.status.code(), Some(2), "missing={missing}");
        assert!(replay.stdout.is_empty(), "missing={missing}");
        let stderr = String::from_utf8(replay.stderr).expect("stderr must be utf8");
        assert!(
            stderr.contains("missing required file"),
            "missing={missing}, stderr={stderr}"
        );

        fs::remove_dir_all(&temp_root).expect("temp root cleanup");
    }
}

#[test]
fn precision_replay_rejects_malformed_result_txt_with_exit_2() {
    let temp_root = unique_temp_root("precision-replay-malformed-result");
    let artifact_rel = make_record_artifact(&temp_root);
    fs::write(
        temp_root.join(&artifact_rel).join("result.txt"),
        concat!(
            "RESULT: PASS\n",
            "COMMAND: record\n",
            "TARGET: fixture://target\n",
            "MODE: runtime_mode\n",
            "EQUIVALENCE: exact\n",
        ),
    )
    .expect("malformed result must be written");

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["replay", &artifact_rel, "--mode", "runtime_mode"])
        .output()
        .expect("precision replay should run");

    assert_eq!(replay.status.code(), Some(2));
    assert!(replay.stdout.is_empty());
    let stderr = String::from_utf8(replay.stderr).expect("stderr must be utf8");
    assert!(stderr.contains("invalid result.txt"));

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_replay_rejects_result_meta_command_target_and_mode_mismatches_with_exit_2() {
    let cases = [
        (
            "COMMAND: record\n",
            "COMMAND: replay\n",
            "result/meta command mismatch",
        ),
        (
            "TARGET: fixture://target\n",
            "TARGET: fixture://other\n",
            "result/meta target mismatch",
        ),
        (
            "MODE: runtime_mode\n",
            "MODE: none\n",
            "result/meta mode mismatch",
        ),
    ];

    for (from, to, needle) in cases {
        let temp_root = unique_temp_root(&format!(
            "precision-replay-result-mismatch-{}",
            needle.replace(' ', "-")
        ));
        let artifact_rel = make_record_artifact(&temp_root);
        let result_path = temp_root.join(&artifact_rel).join("result.txt");
        let original = fs::read_to_string(&result_path).expect("result.txt must exist");
        let invalid = original.replacen(from, to, 1);
        fs::write(&result_path, invalid).expect("invalid result must be written");

        let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
            .current_dir(&temp_root)
            .args(["replay", &artifact_rel, "--mode", "runtime_mode"])
            .output()
            .expect("precision replay should run");

        assert_eq!(replay.status.code(), Some(2), "needle={needle}");
        assert!(replay.stdout.is_empty(), "needle={needle}");
        let stderr = String::from_utf8(replay.stderr).expect("stderr must be utf8");
        assert!(stderr.contains(needle), "needle={needle}, stderr={stderr}");

        fs::remove_dir_all(&temp_root).expect("temp root cleanup");
    }
}

#[test]
fn precision_replay_rejects_malformed_meta_json_with_exit_2() {
    let temp_root = unique_temp_root("precision-replay-malformed-meta-json");
    let artifact_rel = make_record_artifact(&temp_root);
    fs::write(
        temp_root.join(&artifact_rel).join("meta.json"),
        b"{not-json",
    )
    .expect("broken meta must be written");

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["replay", &artifact_rel, "--mode", "runtime_mode"])
        .output()
        .expect("precision replay should run");

    assert_eq!(replay.status.code(), Some(2));
    assert!(replay.stdout.is_empty());
    let stderr = String::from_utf8(replay.stderr).expect("stderr must be utf8");
    assert!(stderr.contains("invalid authoritative meta"));

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_replay_rejects_inconsistent_meta_json_with_exit_2() {
    let temp_root = unique_temp_root("precision-replay-inconsistent-meta");
    let artifact_rel = make_record_artifact(&temp_root);
    let meta_path = temp_root.join(&artifact_rel).join("meta.json");
    let mut meta = read_json(&meta_path);
    let signal_input_count = meta
        .as_object_mut()
        .and_then(|meta| meta.get_mut("signal_input_count"))
        .and_then(|value| value.as_u64())
        .expect("meta signal_input_count must be a u64");
    meta["signal_input_count"] = Value::from(signal_input_count - 1);
    write_json(&meta_path, &meta);

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["replay", &artifact_rel, "--mode", "runtime_mode"])
        .output()
        .expect("precision replay should run");

    assert_eq!(replay.status.code(), Some(2));
    assert!(replay.stdout.is_empty());
    let stderr = String::from_utf8(replay.stderr).expect("stderr must be utf8");
    assert!(stderr.contains("meta signal_input_count mismatch"));

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_replay_rejects_artifact_directory_outside_artifacts_parent_with_exit_2() {
    let temp_root = unique_temp_root("precision-replay-non-artifacts-parent");
    let artifact_rel = make_record_artifact(&temp_root);
    let run_id = artifact_rel
        .rsplit('/')
        .next()
        .expect("artifact path must contain run id");
    let relocated_parent = temp_root.join("published");
    fs::create_dir_all(&relocated_parent).expect("published dir must be created");
    let relocated_artifact = relocated_parent.join(run_id);
    fs::rename(temp_root.join(&artifact_rel), &relocated_artifact)
        .expect("artifact dir must be moved outside artifacts");

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args([
            "replay",
            relocated_artifact
                .strip_prefix(&temp_root)
                .expect("relocated path must stay under temp root")
                .to_str()
                .expect("relocated path must be utf8"),
            "--mode",
            "runtime_mode",
        ])
        .output()
        .expect("precision replay should run");

    assert_eq!(replay.status.code(), Some(2));
    assert!(replay.stdout.is_empty());
    let stderr = String::from_utf8(replay.stderr).expect("stderr must be utf8");
    assert!(stderr.contains("published artifact directories under artifacts/"));

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_replay_rejects_malformed_trace_json_with_exit_2() {
    let temp_root = unique_temp_root("precision-replay-malformed-trace-json");
    let artifact_rel = make_record_artifact(&temp_root);
    fs::write(
        temp_root.join(&artifact_rel).join("trace.json"),
        b"{not-json",
    )
    .expect("broken trace must be written");

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["replay", &artifact_rel, "--mode", "runtime_mode"])
        .output()
        .expect("precision replay should run");

    assert_eq!(replay.status.code(), Some(2));
    assert!(replay.stdout.is_empty());
    let stderr = String::from_utf8(replay.stderr).expect("stderr must be utf8");
    assert!(stderr.contains("invalid authoritative trace"));

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_replay_rejects_inconsistent_trace_json_with_exit_2() {
    let temp_root = unique_temp_root("precision-replay-inconsistent-trace");
    let artifact_rel = make_record_artifact(&temp_root);
    let trace_path = temp_root.join(&artifact_rel).join("trace.json");
    let mut trace = read_json(&trace_path);
    trace["captured_trace"]["nodes"][0]["values"]
        .as_array_mut()
        .expect("values array")
        .push(Value::from(123456_u64));
    write_json(&trace_path, &trace);

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["replay", &artifact_rel, "--mode", "runtime_mode"])
        .output()
        .expect("precision replay should run");

    assert_eq!(replay.status.code(), Some(2));
    assert!(replay.stdout.is_empty());
    let stderr = String::from_utf8(replay.stderr).expect("stderr must be utf8");
    assert!(stderr.contains("node length mismatch"));

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_replay_reports_first_divergence_for_induced_recorded_mismatch() {
    let temp_root = unique_temp_root("precision-replay-mismatch");
    let record = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["record", "fixture://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record should run");
    assert!(
        record.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&record.stderr)
    );
    let artifact_rel = artifact_path_from_stdout(&record.stdout);

    let trace_path = temp_root.join(&artifact_rel).join("trace.json");
    let original = fs::read_to_string(&trace_path).expect("trace.json must exist");
    let perturbed = original.replacen("304000", "304001", 1);
    fs::write(&trace_path, perturbed).expect("perturbed trace must be written");

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["replay", &artifact_rel, "--mode", "runtime_mode"])
        .output()
        .expect("precision replay should run");

    assert_eq!(
        replay.status.code(),
        Some(1),
        "stderr: {}",
        String::from_utf8_lossy(&replay.stderr)
    );
    assert!(
        replay.stderr.is_empty(),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&replay.stderr)
    );
    let stdout = String::from_utf8(replay.stdout).expect("replay stdout must be utf8");
    assert!(stdout.contains("RESULT: FAIL\n"));
    assert!(stdout.contains("EQUIVALENCE: diverged\n"));
    assert!(
        stdout.contains("FIRST_DIVERGENCE: step=1 node=signal.interval_us cause=VAL_MISMATCH\n")
    );

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_replay_stdout_matches_published_result_file_byte_for_byte() {
    let temp_root = unique_temp_root("precision-replay-byte-identity");
    let record = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["record", "fixture://target", "--mode", "runtime_mode"])
        .output()
        .expect("precision record should run");
    let artifact_rel = artifact_path_from_stdout(&record.stdout);

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["replay", &artifact_rel, "--mode", "runtime_mode"])
        .output()
        .expect("precision replay should run");

    assert_eq!(
        replay.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&replay.stderr)
    );
    let replay_artifact = artifact_path_from_stdout(&replay.stdout);
    let result_txt = fs::read(temp_root.join(replay_artifact).join("result.txt"))
        .expect("result.txt must exist");
    assert_eq!(result_txt, replay.stdout);

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_replay_rejects_rpl_file_input_with_exit_2() {
    let temp_root = unique_temp_root("precision-replay-rpl-reject");
    let rpl_path = temp_root.join("retained-proof.rpl");
    fs::write(&rpl_path, b"RPL0").expect("rpl proof file should be written");

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args([
            "replay",
            rpl_path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("utf8 path"),
            "--mode",
            "runtime_mode",
        ])
        .output()
        .expect("precision replay should run");

    assert_eq!(replay.status.code(), Some(2));
    assert!(replay.stdout.is_empty());
    let stderr = String::from_utf8(replay.stderr).expect("stderr must be utf8");
    assert!(stderr.contains("published artifact directory"));

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_replay_rejects_malformed_comparison_payload_with_exit_2() {
    let temp_root = unique_temp_root("precision-replay-invalid-comparison");
    let (_, replay_artifact) = make_replay_artifact(&temp_root);

    let trace_path = temp_root.join(&replay_artifact).join("trace.json");
    let mut trace = read_json(&trace_path);
    trace["comparison"]["equivalence"] = Value::from("maybe");
    write_json(&trace_path, &trace);

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
fn precision_replay_rejects_incomplete_replay_comparison_payload_with_exit_2() {
    let temp_root = unique_temp_root("precision-replay-incomplete-comparison");
    let (_, replay_artifact) = make_replay_artifact(&temp_root);

    let trace_path = temp_root.join(&replay_artifact).join("trace.json");
    let mut trace = read_json(&trace_path);
    trace
        .as_object_mut()
        .expect("trace root object")
        .remove("comparison");
    write_json(&trace_path, &trace);

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["replay", &replay_artifact, "--mode", "runtime_mode"])
        .output()
        .expect("precision replay should run");

    assert_eq!(replay.status.code(), Some(2));
    assert!(replay.stdout.is_empty());
    let stderr = String::from_utf8(replay.stderr).expect("stderr must be utf8");
    assert!(stderr.contains("replay artifact must contain replay comparison payload"));

    fs::remove_dir_all(&temp_root).expect("temp root cleanup");
}

#[test]
fn precision_replay_rejects_inconsistent_replay_comparison_payload_with_exit_2() {
    let temp_root = unique_temp_root("precision-replay-inconsistent-comparison");
    let (_, replay_artifact) = make_replay_artifact(&temp_root);

    let trace_path = temp_root.join(&replay_artifact).join("trace.json");
    let mut trace = read_json(&trace_path);
    trace["comparison"]["first_divergence"] = serde_json::json!({
        "step": 9,
        "node": "signal.interval_us",
        "cause": "VAL_MISMATCH"
    });
    write_json(&trace_path, &trace);

    let replay = Command::new(env!("CARGO_BIN_EXE_precision"))
        .current_dir(&temp_root)
        .args(["replay", &replay_artifact, "--mode", "runtime_mode"])
        .output()
        .expect("precision replay should run");

    assert_eq!(replay.status.code(), Some(2));
    assert!(replay.stdout.is_empty());
    let stderr = String::from_utf8(replay.stderr).expect("stderr must be utf8");
    assert!(stderr.contains("inconsistent comparison first_divergence"));

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
        .args([
            "diff",
            &diff_artifact,
            &diff_artifact,
            "--mode",
            "runtime_mode",
        ])
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
    assert!(
        record.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&record.stderr)
    );

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
    assert!(
        envelope.stderr.is_empty(),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&envelope.stderr)
    );
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
        .args([
            "record",
            "/dev/does-not-exist-precision",
            "--mode",
            "runtime_mode",
        ])
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
        .args([
            "diff",
            &diff_artifact,
            &diff_artifact,
            "--mode",
            "runtime_mode",
        ])
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
        "NON-AUTHORITATIVE: This utility exists outside the active high-integrity contract."
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
