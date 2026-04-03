use super::artifacts::{
    expected_det_hash, generate_forensic_artifacts, generate_header_test_artifact, CanonChannels,
    GoldenPolicy, VerificationScenario, HEADER_TEST_FRAMES, HEADER_TEST_RATE, SCENARIOS,
};
use super::{ValidateArgs, ValidateMode};
use dpw4::verification::HeaderVerifier;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

const TOOLCHAIN_PIN: &str = "1.91.1";

#[derive(Clone, Copy)]
enum CheckStatus {
    Pass,
    Fail,
}

impl CheckStatus {
    fn as_str(self) -> &'static str {
        match self {
            CheckStatus::Pass => "PASS",
            CheckStatus::Fail => "FAIL",
        }
    }
}

struct CheckResult {
    name: &'static str,
    status: CheckStatus,
    details: String,
}

pub(crate) fn run_validate(args: ValidateArgs) -> i32 {
    let out_dir = args.out;
    let run1 = out_dir.join("run1");
    let run2 = out_dir.join("run2");

    let mut checks: Vec<CheckResult> = Vec::new();
    let mut overall_fail = false;

    if let Err(e) = fs::create_dir_all(&out_dir) {
        let msg = format!("failed to create out dir {}: {}", out_dir.display(), e);
        emit_validate_line(args.json, &format!("FAIL setup: {}", msg));
        checks.push(CheckResult {
            name: "setup",
            status: CheckStatus::Fail,
            details: msg,
        });
        return finalize_validate(args.json, out_dir, checks, args.keep, overall_fail || true);
    }

    let workspace_version = read_workspace_version(Path::new("Cargo.toml"));
    let lock_version = read_dpw4_version_from_lock(Path::new("Cargo.lock"));
    let version_details = match (&workspace_version, &lock_version) {
        (Ok(ws), Ok(lock)) if ws == lock => {
            emit_validate_line(
                args.json,
                &format!("PASS version_consistency: workspace={} lock={}", ws, lock),
            );
            CheckResult {
                name: "version_consistency",
                status: CheckStatus::Pass,
                details: format!("workspace={} lock={}", ws, lock),
            }
        }
        (Ok(ws), Ok(lock)) => {
            overall_fail = true;
            let msg = format!("workspace={} lock={}", ws, lock);
            emit_validate_line(args.json, &format!("FAIL version_consistency: {}", msg));
            CheckResult {
                name: "version_consistency",
                status: CheckStatus::Fail,
                details: msg,
            }
        }
        (Err(e), _) => {
            overall_fail = true;
            emit_validate_line(args.json, &format!("FAIL version_consistency: {}", e));
            CheckResult {
                name: "version_consistency",
                status: CheckStatus::Fail,
                details: e.clone(),
            }
        }
        (_, Err(e)) => {
            overall_fail = true;
            emit_validate_line(args.json, &format!("FAIL version_consistency: {}", e));
            CheckResult {
                name: "version_consistency",
                status: CheckStatus::Fail,
                details: e.clone(),
            }
        }
    };
    checks.push(version_details);

    let toolchain = read_toolchain_channel(Path::new("rust-toolchain.toml"));
    let toolchain_result = match toolchain {
        Ok(actual) if actual == TOOLCHAIN_PIN => {
            emit_validate_line(
                args.json,
                &format!("PASS toolchain_pin: channel={}", actual),
            );
            CheckResult {
                name: "toolchain_pin",
                status: CheckStatus::Pass,
                details: format!("expected={} actual={}", TOOLCHAIN_PIN, actual),
            }
        }
        Ok(actual) => {
            overall_fail = true;
            let msg = format!("expected={} actual={}", TOOLCHAIN_PIN, actual);
            emit_validate_line(args.json, &format!("FAIL toolchain_pin: {}", msg));
            CheckResult {
                name: "toolchain_pin",
                status: CheckStatus::Fail,
                details: msg,
            }
        }
        Err(e) => {
            overall_fail = true;
            emit_validate_line(args.json, &format!("FAIL toolchain_pin: {}", e));
            CheckResult {
                name: "toolchain_pin",
                status: CheckStatus::Fail,
                details: e,
            }
        }
    };
    checks.push(toolchain_result);

    let run_header = out_dir.join("run_header");
    if let Err(e) = fs::create_dir_all(&run_header) {
        let msg = format!("failed to create run_header: {}", e);
        emit_validate_line(args.json, &format!("FAIL header_stream_integrity: {}", msg));
        checks.push(CheckResult {
            name: "header_stream_integrity",
            status: CheckStatus::Fail,
            details: msg,
        });
        overall_fail = true;
        return finalize_validate(args.json, out_dir, checks, args.keep, overall_fail);
    }

    let header_test_path = run_header.join("header_stream_integrity_test.bin");
    let header_result = match generate_header_test_artifact(&header_test_path) {
        Ok(_) => match run_header_integrity_direct(&header_test_path) {
            Ok(()) => {
                emit_validate_line(
                    args.json,
                    &format!(
                        "PASS header_stream_integrity: header-only stream valid ({} frames)",
                        HEADER_TEST_FRAMES
                    ),
                );
                CheckResult {
                    name: "header_stream_integrity",
                    status: CheckStatus::Pass,
                    details: format!(
                        "file={} frames={} rate={}",
                        header_test_path.display(),
                        HEADER_TEST_FRAMES,
                        HEADER_TEST_RATE
                    ),
                }
            }
            Err(e) => {
                overall_fail = true;
                let msg = format!("file={} {}", header_test_path.display(), e);
                emit_validate_line(args.json, &format!("FAIL header_stream_integrity: {}", msg));
                CheckResult {
                    name: "header_stream_integrity",
                    status: CheckStatus::Fail,
                    details: msg,
                }
            }
        },
        Err(e) => {
            overall_fail = true;
            let msg = format!("artifact_gen_failed: {}", e);
            emit_validate_line(args.json, &format!("FAIL header_stream_integrity: {}", msg));
            CheckResult {
                name: "header_stream_integrity",
                status: CheckStatus::Fail,
                details: msg,
            }
        }
    };
    checks.push(header_result);

    let determinism_result = match run_determinism_check(&run1, &run2, args.mode) {
        Ok(results) => {
            let mut msg = String::from("PASS determinism_bit_exact:\n");
            let mut details = String::new();

            for (stem, actual) in &results {
                let sc = SCENARIOS.iter().find(|s| s.id == stem);
                let golden_str = match sc.map(|s| s.golden) {
                    Some(GoldenPolicy::Pinned(h)) => h,
                    Some(GoldenPolicy::Unpinned) => "Unpinned",
                    None => "N/A",
                };

                msg.push_str(&format!(
                    "    {}.canon.sig\n      Golden:   {}\n      Actual:   {}\n",
                    stem, golden_str, actual
                ));

                if !details.is_empty() {
                    details.push(' ');
                }
                details.push_str(&format!("{}_sha256={}", stem, actual));
            }

            emit_validate_line(args.json, &msg);

            CheckResult {
                name: "determinism_bit_exact",
                status: CheckStatus::Pass,
                details,
            }
        }
        Err(e) => {
            overall_fail = true;
            emit_validate_line(args.json, &format!("FAIL determinism_bit_exact: {}", e));
            CheckResult {
                name: "determinism_bit_exact",
                status: CheckStatus::Fail,
                details: e,
            }
        }
    };
    checks.push(determinism_result);

    finalize_validate(args.json, out_dir, checks, args.keep, overall_fail)
}

fn finalize_validate(
    json: bool,
    out_dir: PathBuf,
    checks: Vec<CheckResult>,
    keep: bool,
    failed: bool,
) -> i32 {
    let status = if failed { "failed" } else { "passed" };
    if json {
        let json = build_validate_json(status, &out_dir, &checks);
        println!("{}", json);
    } else if failed {
        println!("VERIFICATION FAILED");
    } else {
        println!("VERIFICATION PASSED");
    }

    if !failed && !keep {
        for run_dir in [
            out_dir.join("run1"),
            out_dir.join("run2"),
            out_dir.join("run_header"),
        ] {
            if run_dir.exists() {
                if let Err(e) = fs::remove_dir_all(&run_dir) {
                    emit_validate_line(
                        json,
                        &format!(
                            "WARN cleanup: failed to remove {}: {}",
                            run_dir.display(),
                            e
                        ),
                    );
                }
            }
        }
    }

    if failed {
        1
    } else {
        0
    }
}

fn emit_validate_line(json: bool, line: &str) {
    if json {
        eprintln!("{}", line);
    } else {
        println!("{}", line);
    }
}

fn read_workspace_version(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|e| format!("failed to open {}: {}", path.display(), e))?;
    let reader = BufReader::new(file);
    let mut in_section = false;

    for line in reader.lines() {
        let line = line.map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
        let t = line.trim();
        if t.starts_with('[') && t.ends_with(']') {
            in_section = t == "[workspace.package]";
            continue;
        }
        if in_section && t.starts_with("version") {
            if let Some(v) = parse_toml_string_value(t) {
                return Ok(v);
            }
            return Err("workspace.package.version is malformed".to_string());
        }
    }

    Err("workspace.package.version not found".to_string())
}

fn read_dpw4_version_from_lock(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|e| format!("failed to open {}: {}", path.display(), e))?;
    let reader = BufReader::new(file);

    let mut in_package = false;
    let mut name: Option<String> = None;
    let mut version: Option<String> = None;

    for line in reader.lines() {
        let line = line.map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
        let t = line.trim();

        if t == "[[package]]" {
            if in_package && name.as_deref() == Some("dpw4") {
                if let Some(v) = version {
                    return Ok(v);
                }
                return Err("Cargo.lock package dpw4 missing version".to_string());
            }
            in_package = true;
            name = None;
            version = None;
            continue;
        }

        if in_package {
            if t.starts_with("name") {
                name = parse_toml_string_value(t);
            } else if t.starts_with("version") {
                version = parse_toml_string_value(t);
            }
        }
    }

    if in_package && name.as_deref() == Some("dpw4") {
        if let Some(v) = version {
            return Ok(v);
        }
    }

    Err("dpw4 package entry not found in Cargo.lock".to_string())
}

fn read_toolchain_channel(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|e| format!("failed to open {}: {}", path.display(), e))?;
    let reader = BufReader::new(file);
    let mut in_section = false;

    for line in reader.lines() {
        let line = line.map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
        let t = line.trim();
        if t.starts_with('[') && t.ends_with(']') {
            in_section = t == "[toolchain]";
            continue;
        }
        if in_section && t.starts_with("channel") {
            if let Some(v) = parse_toml_string_value(t) {
                return Ok(v);
            }
            return Err("toolchain.channel is malformed".to_string());
        }
    }

    Err("toolchain.channel not found".to_string())
}

fn parse_toml_string_value(line: &str) -> Option<String> {
    let mut split = line.splitn(2, '=');
    let _ = split.next()?;
    let rhs = split.next()?.trim();
    let quote = rhs.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let chars = rhs.chars().skip(1);
    let mut out = String::new();
    for c in chars {
        if c == quote {
            return Some(out);
        }
        out.push(c);
    }
    None
}

fn run_determinism_check(
    run1: &Path,
    run2: &Path,
    mode: ValidateMode,
) -> Result<Vec<(String, String)>, String> {
    if run1.exists() {
        fs::remove_dir_all(run1)
            .map_err(|e| format!("failed to reset {}: {}", run1.display(), e))?;
    }
    if run2.exists() {
        fs::remove_dir_all(run2)
            .map_err(|e| format!("failed to reset {}: {}", run2.display(), e))?;
    }

    generate_forensic_artifacts(run1, Some(mode)).map_err(|e| {
        format!(
            "artifact generation failed for run1 {}: {}",
            run1.display(),
            e
        )
    })?;
    generate_forensic_artifacts(run2, Some(mode)).map_err(|e| {
        format!(
            "artifact generation failed for run2 {}: {}",
            run2.display(),
            e
        )
    })?;

    let mut results = Vec::new();

    for scenario in SCENARIOS {
        let stem = scenario.id;
        let expected_hash_opt = expected_det_hash(mode, stem);
        let is_normative = expected_hash_opt.is_some();
        let csv_name = format!("{}.det.csv", stem);
        let run1_csv = run1.join(&csv_name);

        if !run1_csv.exists() {
            if is_normative {
                return Err(format!(
                    "FAIL det_baseline: id={} normative artifact missing from run directory",
                    stem
                ));
            }
            continue;
        }

        let det_hash = match verify_artifact_integrity(&run1_csv, &run2.join(&csv_name), None) {
            Ok(h) => h,
            Err(e) => return Err(format!("FAIL det_bytes: file={} {}", csv_name, e)),
        };

        if let Some(expected_det) = expected_hash_opt {
            if expected_det == "PIN_ME" {
                return Err(format!(
                    "FAIL det_baseline: id={} expected=PIN_ME actual={}",
                    stem, det_hash
                ));
            }
            if det_hash != expected_det {
                return Err(format!(
                    "FAIL det_baseline: id={} expected={} actual={}",
                    stem, expected_det, det_hash
                ));
            }
        }

        let sig_name = format!("{}.canon.sig", stem);
        let actual_hash =
            match verify_canon_sig(&run1.join(&sig_name), &run2.join(&sig_name), scenario) {
                Ok(h) => h,
                Err(e) => return Err(format!("FAIL canon_protocol: file={} {}", sig_name, e)),
            };

        match scenario.golden {
            GoldenPolicy::Pinned(expected) => {
                if actual_hash != expected {
                    return Err(format!(
                        "FAIL pinned_regression: id={} expected={} actual={}",
                        scenario.id, expected, actual_hash
                    ));
                }
            }
            GoldenPolicy::Unpinned => {
                eprintln!(
                    "WARN non_normative_canary: id={} actual={}",
                    scenario.id, actual_hash
                );
            }
        }

        results.push((stem.to_string(), actual_hash));
    }

    Ok(results)
}

fn verify_canon_sig(
    path_a: &Path,
    path_b: &Path,
    scenario: &VerificationScenario,
) -> Result<String, String> {
    verify_artifact_integrity(path_a, path_b, None)?;

    let file = File::open(path_a).map_err(|e| format!("open {}: {}", path_a.display(), e))?;
    let mut reader = BufReader::new(file);

    let mut line1 = String::new();
    reader
        .read_line(&mut line1)
        .map_err(|e| format!("read header: {}", e))?;
    let mut line2 = String::new();
    reader
        .read_line(&mut line2)
        .map_err(|e| format!("read hash: {}", e))?;

    let parts: Vec<&str> = line1.split('|').map(|s| s.trim()).collect();
    if parts.len() != 4 {
        return Err(format!("malformed_header: '{}'", line1.trim()));
    }
    if parts[0] != "v1" {
        return Err(format!(
            "version_mismatch: expected 'v1', got '{}'",
            parts[0]
        ));
    }
    if parts[1] != "le-i32" {
        return Err(format!(
            "format_mismatch: expected 'le-i32', got '{}'",
            parts[1]
        ));
    }

    let channels_part = parts[2];
    let expected_channels = match scenario.channels {
        CanonChannels::Mono => "channels=mono",
        CanonChannels::MasterTuple => "channels=saw,pulse,triangle,sine",
    };

    if channels_part != expected_channels {
        return Err(format!(
            "channels_mismatch: expected '{}', got '{}'",
            expected_channels, channels_part
        ));
    }

    let samples_part = parts[3];
    let expected_samples = format!("samples={}", scenario.num_samples);
    if samples_part != expected_samples {
        return Err(format!(
            "samples_mismatch: expected '{}', got '{}'",
            expected_samples, samples_part
        ));
    }

    let hash_hex = line2.trim().to_string();
    if hash_hex.len() != 64 {
        return Err(format!("invalid_hash_len: {}", hash_hex.len()));
    }

    Ok(hash_hex)
}

fn verify_artifact_integrity(
    path_a: &Path,
    path_b: &Path,
    expected_hash: Option<&str>,
) -> Result<String, String> {
    let mut a = File::open(path_a).map_err(|e| format!("open {}: {}", path_a.display(), e))?;
    let mut b = File::open(path_b).map_err(|e| format!("open {}: {}", path_b.display(), e))?;
    let mut hasher = Sha256::new();

    let mut buf_a = [0u8; 8192];
    let mut buf_b = [0u8; 8192];
    let mut offset: u64 = 0;

    loop {
        let n_a = a
            .read(&mut buf_a)
            .map_err(|e| format!("read {}: {}", path_a.display(), e))?;
        let n_b = b
            .read(&mut buf_b)
            .map_err(|e| format!("read {}: {}", path_b.display(), e))?;
        let n = n_a.min(n_b);

        hasher.update(&buf_a[..n_a]);

        for i in 0..n {
            if buf_a[i] != buf_b[i] {
                return Err(format!(
                    "first_diff_offset={} run1_byte=0x{:02x} run2_byte=0x{:02x}",
                    offset + i as u64,
                    buf_a[i],
                    buf_b[i]
                ));
            }
        }

        if n_a != n_b {
            if n_a < n_b {
                return Err(format!(
                    "size_mismatch first_diff_offset={} run1_byte=EOF run2_byte=0x{:02x}",
                    offset + n as u64,
                    buf_b[n]
                ));
            }
            return Err(format!(
                "size_mismatch first_diff_offset={} run1_byte=0x{:02x} run2_byte=EOF",
                offset + n as u64,
                buf_a[n]
            ));
        }

        if n_a == 0 {
            break;
        }
        offset += n_a as u64;
    }

    let hash = hex::encode(hasher.finalize());
    if let Some(expected) = expected_hash {
        if !expected.is_empty() && hash != expected {
            return Err(format!(
                "golden_hash_mismatch expected={} actual={}",
                expected, hash
            ));
        }
    }

    Ok(hash)
}

fn run_header_integrity_direct(path: &Path) -> Result<(), String> {
    let file = File::open(path).map_err(|e| format!("open {}: {}", path.display(), e))?;
    let count = HeaderVerifier::verify_header_stream(file).map_err(|e| e.to_string())?;

    if count == 0 {
        return Err("no_headers_found".to_string());
    }

    Ok(())
}

fn build_validate_json(status: &str, out_dir: &Path, checks: &[CheckResult]) -> String {
    let mut s = String::new();
    s.push('{');
    s.push_str(&format!("\"status\":\"{}\",", status));
    s.push_str(&format!(
        "\"out_dir\":\"{}\",",
        json_escape(&out_dir.display().to_string())
    ));
    s.push_str("\"checks\":[");
    for (i, check) in checks.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"name\":\"{}\",\"status\":\"{}\",\"details\":\"{}\"}}",
            check.name,
            check.status.as_str(),
            json_escape(&check.details)
        ));
    }
    s.push_str("],");
    s.push_str(&format!(
        "\"dpw4_version\":\"{}\",",
        json_escape(env!("CARGO_PKG_VERSION"))
    ));
    s.push_str("\"features\":[");
    for (i, feat) in compiled_features().iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push('"');
        s.push_str(feat);
        s.push('"');
    }
    s.push_str("]}");
    s
}

fn json_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

fn compiled_features() -> Vec<&'static str> {
    let mut features: Vec<&'static str> = Vec::new();
    features.push("cli");
    #[cfg(feature = "audit")]
    {
        features.push("audit");
    }
    features
}
