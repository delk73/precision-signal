#![cfg(feature = "cli")]

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn generate_writes_binary_header_to_stdout_without_stderr_noise() {
    let output = Command::new(env!("CARGO_BIN_EXE_precision"))
        .args(["generate", "--seconds", "1"])
        .output()
        .expect("precision generate should run");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(output.stderr.is_empty(), "generate must not emit text on stderr for default shape");
    assert!(output.stdout.len() > 4, "generate must emit a non-empty binary stream");
    assert_eq!(&output.stdout[..4], b"DP32", "binary stream must start with the DP32 header magic");
}

#[test]
fn generate_sends_triangle_dpw1_advisory_to_stderr_only() {
    let output = Command::new(env!("CARGO_BIN_EXE_precision"))
        .args(["generate", "--shape", "triangle-dpw1", "--seconds", "1"])
        .output()
        .expect("precision generate triangle-dpw1 should run");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(&output.stdout[..4], b"DP32", "stdout must remain the binary stream");

    let stderr = String::from_utf8(output.stderr).expect("stderr must be valid utf-8 text");
    assert!(
        stderr.contains("ADVISORY: Triangle (DPW1 Naive) is non-band-limited and will alias at high frequencies."),
        "expected triangle-dpw1 advisory on stderr, got: {stderr}"
    );
}

#[test]
fn generate_writes_to_out_path_without_stdout_flooding() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time must be after unix epoch")
        .as_nanos();
    let out_path = std::env::temp_dir().join(format!("precision-generate-{unique}.bin"));

    let output = Command::new(env!("CARGO_BIN_EXE_precision"))
        .args([
            "generate",
            "--seconds",
            "1",
            "--out",
            out_path.to_str().expect("temp path must be utf-8"),
        ])
        .output()
        .expect("precision generate --out should run");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(output.stdout.is_empty(), "--out path must keep stdout empty");

    let bytes = std::fs::read(&out_path).expect("output file must be created");
    assert!(bytes.len() > 4, "output file must contain binary data");
    assert_eq!(&bytes[..4], b"DP32", "output file must start with the DP32 header magic");

    let _ = std::fs::remove_file(out_path);
}
