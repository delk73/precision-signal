use std::path::PathBuf;
use std::process::Command;

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
