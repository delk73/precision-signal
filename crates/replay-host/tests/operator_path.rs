use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use replay_host::{parse_artifact, ParsedArtifact};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root must resolve")
}

fn run_diff(a_rel: &str, b_rel: &str) -> (i32, String, String) {
    let root = repo_root();
    let output = Command::new(env!("CARGO_BIN_EXE_replay-host"))
        .arg("diff")
        .arg(root.join(a_rel))
        .arg(root.join(b_rel))
        .output()
        .expect("replay-host diff command must run");

    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8(output.stdout).expect("stdout must be valid utf8");
    let stderr = String::from_utf8(output.stderr).expect("stderr must be valid utf8");
    (code, stdout, stderr)
}

fn run_import(csv_path: &PathBuf, out_path: &PathBuf) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_replay-host"))
        .arg("import-interval-csv")
        .arg(csv_path)
        .arg(out_path)
        .output()
        .expect("replay-host import command must run");

    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8(output.stdout).expect("stdout must be valid utf8");
    let stderr = String::from_utf8(output.stderr).expect("stderr must be valid utf8");
    (code, stdout, stderr)
}

fn run_validate(csv_path: &PathBuf) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_replay-host"))
        .arg("validate-interval-csv")
        .arg(csv_path)
        .output()
        .expect("replay-host validate command must run");

    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8(output.stdout).expect("stdout must be valid utf8");
    let stderr = String::from_utf8(output.stderr).expect("stderr must be valid utf8");
    (code, stdout, stderr)
}

fn unique_temp_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time must be monotonic enough for tests")
        .as_nanos();
    std::env::temp_dir().join(format!("replay_host_import_test_{nanos}"))
}

fn canonical_capture_csv() -> String {
    let mut csv = String::from("index,interval_us\n");
    for idx in 0..138 {
        let interval = if idx == 0 { 305_564 } else { 304_000 };
        csv.push_str(&format!("{idx},{interval}\n"));
    }
    csv
}

#[test]
fn operator_path_reports_no_divergence_for_identical_real_artifacts() {
    let (code, stdout, stderr) = run_diff(
        "artifacts/demo_v4/header_schema_B.rpl",
        "artifacts/demo_v4/header_schema_B.rpl",
    );

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout, "no divergence\n");
    assert!(stderr.is_empty(), "unexpected stderr: {stderr}");
}

#[test]
fn operator_path_reports_known_real_v1_divergence_at_frame_zero() {
    let (code, stdout, stderr) = run_diff(
        "artifacts/demo_v4/header_schema_B.rpl",
        "artifacts/demo_v4/header_schema_sample_payload_B.rpl",
    );

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout, "first divergence at frame 0\n");
    assert!(stderr.is_empty(), "unexpected stderr: {stderr}");
}

#[test]
fn operator_path_output_is_stable_across_repeated_runs() {
    let first = run_diff(
        "artifacts/demo_v4/header_schema_B.rpl",
        "artifacts/demo_v4/header_schema_sample_payload_B.rpl",
    );
    let second = run_diff(
        "artifacts/demo_v4/header_schema_B.rpl",
        "artifacts/demo_v4/header_schema_sample_payload_B.rpl",
    );

    assert_eq!(first, second, "operator path output must be stable");
    assert_eq!(first.1, "first divergence at frame 0\n");
}

#[test]
fn operator_path_imports_interval_csv_into_canonical_artifact() {
    let temp_dir = unique_temp_dir();
    fs::create_dir_all(&temp_dir).expect("temp dir must be creatable");

    let csv_path = temp_dir.join("intervals.csv");
    let out_path = temp_dir.join("imported.rpl");
    fs::write(&csv_path, canonical_capture_csv()).expect("csv fixture must be writable");

    let (code, stdout, stderr) = run_import(&csv_path, &out_path);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stderr.is_empty(), "unexpected stderr: {stderr}");
    assert!(
        !stdout.contains("validated:"),
        "import should not print validator chatter: {stdout}"
    );
    assert!(
        stdout.contains("wrote:"),
        "stdout should report output path: {stdout}"
    );

    let bytes = fs::read(&out_path).expect("imported artifact must exist");
    let parsed = parse_artifact(&bytes).expect("imported artifact must parse");
    match parsed {
        ParsedArtifact::V1(parsed) => {
            assert_eq!(parsed.header.frame_count, 10_000);
            assert_eq!(parsed.header.frame_size, 16);
        }
        ParsedArtifact::V0(_) => panic!("imported artifact must be v1"),
    }

    let (code, stdout, stderr) = Command::new(env!("CARGO_BIN_EXE_replay-host"))
        .arg("diff")
        .arg(&out_path)
        .arg(&out_path)
        .output()
        .map(|output| {
            (
                output.status.code().unwrap_or(-1),
                String::from_utf8(output.stdout).expect("stdout must be utf8"),
                String::from_utf8(output.stderr).expect("stderr must be utf8"),
            )
        })
        .expect("replay-host diff command must run");

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout, "no divergence\n");
    assert!(stderr.is_empty(), "unexpected stderr: {stderr}");
}

#[test]
fn operator_path_validates_interval_csv_contract() {
    let temp_dir = unique_temp_dir();
    fs::create_dir_all(&temp_dir).expect("temp dir must be creatable");

    let csv_path = temp_dir.join("intervals.csv");
    fs::write(&csv_path, canonical_capture_csv()).expect("csv fixture must be writable");

    let (code, stdout, stderr) = run_validate(&csv_path);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stderr.is_empty(), "unexpected stderr: {stderr}");
    assert!(stdout.contains("validated:"));
    assert!(stdout.contains("rows: 138"));
    assert!(stdout.contains("last_index: 137"));
}

#[test]
fn operator_path_rejects_short_interval_csv() {
    let temp_dir = unique_temp_dir();
    fs::create_dir_all(&temp_dir).expect("temp dir must be creatable");

    let csv_path = temp_dir.join("intervals.csv");
    fs::write(&csv_path, "index,interval_us\n0,305564\n1,304000\n2,304000\n")
        .expect("csv fixture must be writable");

    let (code, stdout, stderr) = run_validate(&csv_path);
    assert_eq!(code, 1, "stdout: {stdout}");
    assert!(stdout.is_empty(), "unexpected stdout: {stdout}");
    assert!(
        stderr.contains("expected 138 interval rows, found 3"),
        "stderr should explain row-count rejection: {stderr}"
    );
}
