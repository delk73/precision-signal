#![cfg(feature = "cli")]

use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time must be after unix epoch")
        .as_nanos();
    let temp_root = std::env::temp_dir().join(format!("precision-authoritative-{unique}"));
    fs::create_dir_all(&temp_root).expect("temp root should be created");

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

    let result_txt = fs::read(artifact_dir.join("result.txt")).expect("result.txt must exist");
    assert_eq!(result_txt, stdout_bytes);

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
