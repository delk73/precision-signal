use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use replay_core::artifact::{
    V1_OFF_BOARD_ID, V1_OFF_BUILD_HASH, V1_OFF_CAPTURE_BOUNDARY, V1_OFF_CLOCK_PROFILE,
    V1_OFF_CONFIG_HASH, V1_OFF_FLAGS, V1_OFF_FRAME_SIZE, V1_OFF_HEADER_LEN, V1_OFF_RESERVED,
    V1_OFF_SCHEMA_HASH, V1_OFF_SCHEMA_LEN, V1_OFF_VERSION,
};
use replay_host::{
    diff_artifacts0, parse_artifact, parse_artifact_allow_trailing, parse_replay_frames_legacy0,
    ParseError, ParsedArtifact,
};

static NEXT_TEMP_DIR: AtomicUsize = AtomicUsize::new(0);

struct ParseExpectation {
    label: &'static str,
    path: PathBuf,
    strict_ok: bool,
    allow_trailing_ok: bool,
    canonical_len: Option<usize>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root must resolve")
}

fn real_v1_fixture_paths() -> [PathBuf; 2] {
    let root = repo_root();
    [
        root.join("artifacts/demo_v4/header_schema_B.rpl"),
        root.join("artifacts/demo_v4/header_schema_sample_payload_B.rpl"),
    ]
}

fn unique_temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "replay-host-corpus-{}-{}",
        std::process::id(),
        NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&dir).expect("temp corpus dir must be created");
    dir
}

fn write_case(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, bytes).expect("fixture bytes must be written");
    path
}

// The checked-in real v1 parser/replay corpus currently consists of the demo_v4
// extended-header fixtures. The adversarial and mutation suites in Python keep
// their case bytes in-memory, so the remaining path-bound cases below mirror
// those exact case names against temp files derived from the checked-in base
// fixture.
fn mutate_real_v1_cases() -> (Vec<ParseExpectation>, Vec<ParseExpectation>) {
    let base_path = real_v1_fixture_paths()[0].clone();
    let base_bytes = fs::read(&base_path).expect("base fixture must be readable");
    let base_len = base_bytes.len();
    let dir = unique_temp_dir();

    let mut adversarial = Vec::new();
    let mut bytes = base_bytes.clone();
    bytes[0] ^= 0x01;
    adversarial.push(ParseExpectation {
        label: "bad_magic",
        path: write_case(&dir, "bad_magic.rpl", &bytes),
        strict_ok: false,
        allow_trailing_ok: false,
        canonical_len: None,
    });

    let mut bytes = base_bytes.clone();
    bytes[V1_OFF_VERSION..V1_OFF_VERSION + 2].copy_from_slice(&2u16.to_le_bytes());
    adversarial.push(ParseExpectation {
        label: "nonzero_u32_invalid_v1_u16_version",
        path: write_case(&dir, "nonzero_u32_invalid_v1_u16_version.rpl", &bytes),
        strict_ok: false,
        allow_trailing_ok: false,
        canonical_len: None,
    });

    let mut bytes = base_bytes.clone();
    bytes[V1_OFF_HEADER_LEN..V1_OFF_HEADER_LEN + 2].copy_from_slice(&0x0090u16.to_le_bytes());
    adversarial.push(ParseExpectation {
        label: "header_len_too_small",
        path: write_case(&dir, "header_len_too_small.rpl", &bytes),
        strict_ok: false,
        allow_trailing_ok: false,
        canonical_len: None,
    });

    let mut bytes = base_bytes.clone();
    bytes[V1_OFF_HEADER_LEN..V1_OFF_HEADER_LEN + 2].copy_from_slice(&0x0200u16.to_le_bytes());
    adversarial.push(ParseExpectation {
        label: "oversized_header_len",
        path: write_case(&dir, "oversized_header_len.rpl", &bytes),
        strict_ok: false,
        allow_trailing_ok: false,
        canonical_len: None,
    });

    let mut bytes = base_bytes.clone();
    bytes[V1_OFF_FRAME_SIZE..V1_OFF_FRAME_SIZE + 2].copy_from_slice(&8u16.to_le_bytes());
    adversarial.push(ParseExpectation {
        label: "invalid_frame_size",
        path: write_case(&dir, "invalid_frame_size.rpl", &bytes),
        strict_ok: false,
        allow_trailing_ok: false,
        canonical_len: None,
    });

    let mut bytes = base_bytes.clone();
    bytes[V1_OFF_FLAGS..V1_OFF_FLAGS + 2].copy_from_slice(&1u16.to_le_bytes());
    adversarial.push(ParseExpectation {
        label: "v1_nonzero_flags",
        path: write_case(&dir, "v1_nonzero_flags.rpl", &bytes),
        strict_ok: false,
        allow_trailing_ok: false,
        canonical_len: None,
    });

    let mut bytes = base_bytes.clone();
    bytes[V1_OFF_RESERVED..V1_OFF_RESERVED + 2].copy_from_slice(&1u16.to_le_bytes());
    adversarial.push(ParseExpectation {
        label: "v1_nonzero_reserved",
        path: write_case(&dir, "v1_nonzero_reserved.rpl", &bytes),
        strict_ok: false,
        allow_trailing_ok: false,
        canonical_len: None,
    });

    let mut bytes = base_bytes.clone();
    bytes[V1_OFF_SCHEMA_HASH] ^= 0x01;
    adversarial.push(ParseExpectation {
        label: "schema_hash_mismatch",
        path: write_case(&dir, "schema_hash_mismatch.rpl", &bytes),
        strict_ok: false,
        allow_trailing_ok: false,
        canonical_len: None,
    });

    let mut bytes = base_bytes.clone();
    bytes[V1_OFF_SCHEMA_LEN..V1_OFF_SCHEMA_LEN + 4].copy_from_slice(&0xffff_fff0u32.to_le_bytes());
    adversarial.push(ParseExpectation {
        label: "corrupted_schema_len",
        path: write_case(&dir, "corrupted_schema_len.rpl", &bytes),
        strict_ok: false,
        allow_trailing_ok: false,
        canonical_len: None,
    });

    let bytes = base_bytes[..base_len - 1].to_vec();
    adversarial.push(ParseExpectation {
        label: "partial_frame_region",
        path: write_case(&dir, "partial_frame_region.rpl", &bytes),
        strict_ok: false,
        allow_trailing_ok: false,
        canonical_len: None,
    });

    let mut bytes = base_bytes.clone();
    bytes.extend_from_slice(b"\xAA\xBB");
    adversarial.push(ParseExpectation {
        label: "trailing_bytes_rejected_strict_mode",
        path: write_case(&dir, "trailing_bytes_rejected_strict_mode.rpl", &bytes),
        strict_ok: false,
        allow_trailing_ok: true,
        canonical_len: Some(base_len),
    });

    let mut mutation = Vec::new();
    for (label, filename, offset, mask) in [
        ("modify_build_hash_bytes", "modify_build_hash_bytes.rpl", V1_OFF_BUILD_HASH, 0x80),
        (
            "modify_config_hash_bytes",
            "modify_config_hash_bytes.rpl",
            V1_OFF_CONFIG_HASH + 1,
            0x40,
        ),
        (
            "modify_board_id_bytes",
            "modify_board_id_bytes.rpl",
            V1_OFF_BOARD_ID + 2,
            0x20,
        ),
        (
            "modify_clock_profile_bytes",
            "modify_clock_profile_bytes.rpl",
            V1_OFF_CLOCK_PROFILE + 3,
            0x10,
        ),
        (
            "modify_capture_boundary",
            "modify_capture_boundary.rpl",
            V1_OFF_CAPTURE_BOUNDARY,
            0x01,
        ),
    ] {
        let mut bytes = base_bytes.clone();
        bytes[offset] ^= mask;
        mutation.push(ParseExpectation {
            label,
            path: write_case(&dir, filename, &bytes),
            strict_ok: true,
            allow_trailing_ok: true,
            canonical_len: Some(base_len),
        });
    }

    let mut bytes = base_bytes.clone();
    bytes.extend_from_slice(b"\xAA");
    mutation.push(ParseExpectation {
        label: "append_trailing_bytes_strict_mode",
        path: write_case(&dir, "append_trailing_bytes_strict_mode.rpl", &bytes),
        strict_ok: false,
        allow_trailing_ok: true,
        canonical_len: Some(base_len),
    });

    let bytes = base_bytes[..base_len - 1].to_vec();
    mutation.push(ParseExpectation {
        label: "truncate_artifact_one_byte",
        path: write_case(&dir, "truncate_artifact_one_byte.rpl", &bytes),
        strict_ok: false,
        allow_trailing_ok: false,
        canonical_len: None,
    });

    (adversarial, mutation)
}

fn assert_parse_case(case: &ParseExpectation) {
    let bytes = fs::read(&case.path).unwrap_or_else(|err| {
        panic!("{}: failed to read {}: {}", case.label, case.path.display(), err)
    });

    let strict = parse_artifact(&bytes);
    let strict_ok = strict.is_ok();
    let allow = parse_artifact_allow_trailing(&bytes);
    let allow_ok = allow.is_ok();
    if strict_ok != case.strict_ok || allow_ok != case.allow_trailing_ok {
        panic!(
            "{}: path={} strict_actual={} strict_expected={} allow_actual={} allow_expected={}",
            case.label,
            case.path.display(),
            strict_ok,
            case.strict_ok,
            allow_ok,
            case.allow_trailing_ok
        );
    }
    if let (Ok(parsed), Some(expected_len)) = (&strict, case.canonical_len) {
        match parsed {
            ParsedArtifact::V0(parsed) => assert_eq!(
                parsed.canonical_len,
                expected_len,
                "{}: strict canonical_len mismatch for {}",
                case.label,
                case.path.display()
            ),
            ParsedArtifact::V1(parsed) => assert_eq!(
                parsed.canonical_len,
                expected_len,
                "{}: strict canonical_len mismatch for {}",
                case.label,
                case.path.display()
            ),
        }
    }
    if let (Ok(parsed), Some(expected_len)) = (&allow, case.canonical_len) {
        match parsed {
            ParsedArtifact::V0(parsed) => assert_eq!(
                parsed.canonical_len,
                expected_len,
                "{}: allow-trailing canonical_len mismatch for {}",
                case.label,
                case.path.display()
            ),
            ParsedArtifact::V1(parsed) => assert_eq!(
                parsed.canonical_len,
                expected_len,
                "{}: allow-trailing canonical_len mismatch for {}",
                case.label,
                case.path.display()
            ),
        }
    }
}

#[test]
fn valid_v1_real_corpus_matches_python_acceptance() {
    for path in real_v1_fixture_paths() {
        let bytes = fs::read(&path).expect("fixture must be readable");
        let parsed = parse_artifact(&bytes)
            .unwrap_or_else(|err| panic!("strict parse failed for {}: {:?}", path.display(), err));
        let allow = parse_artifact_allow_trailing(&bytes).unwrap_or_else(|err| {
            panic!(
                "allow-trailing parse failed for {}: {:?}",
                path.display(),
                err
            )
        });

        match parsed {
            ParsedArtifact::V1(parsed) => assert_eq!(parsed.canonical_len, bytes.len()),
            ParsedArtifact::V0(_) => panic!("expected v1 fixture at {}", path.display()),
        }
        match allow {
            ParsedArtifact::V1(parsed) => assert_eq!(parsed.canonical_len, bytes.len()),
            ParsedArtifact::V0(_) => panic!("expected v1 fixture at {}", path.display()),
        }
    }
}

#[test]
fn adversarial_v1_real_path_corpus_matches_python_rejection() {
    let (adversarial, _) = mutate_real_v1_cases();
    for case in &adversarial {
        assert_parse_case(case);
    }
}

#[test]
fn mutation_v1_real_path_corpus_matches_python_policy() {
    let (_, mutation) = mutate_real_v1_cases();
    for case in &mutation {
        assert_parse_case(case);
    }
}

#[test]
fn v1_replay_support_is_explicit_legacy_frame_replay() {
    let [base_path, candidate_path] = real_v1_fixture_paths();
    let base_bytes = fs::read(&base_path).expect("base fixture must be readable");
    let candidate_bytes = fs::read(&candidate_path).expect("candidate fixture must be readable");

    let base_frames = parse_replay_frames_legacy0(&base_bytes).unwrap_or_else(|err| {
        panic!(
            "legacy replay decode failed for {}: {:?}",
            base_path.display(),
            err
        )
    });
    let candidate_frames = parse_replay_frames_legacy0(&candidate_bytes).unwrap_or_else(|err| {
        panic!(
            "legacy replay decode failed for {}: {:?}",
            candidate_path.display(),
            err
        )
    });

    assert_eq!(base_frames.len(), 10_000, "unexpected frame count for {}", base_path.display());
    assert_eq!(
        candidate_frames.len(),
        10_000,
        "unexpected frame count for {}",
        candidate_path.display()
    );
    assert_eq!(diff_artifacts0(&base_bytes, &base_bytes), Ok(None));
    assert_eq!(diff_artifacts0(&base_bytes, &candidate_bytes), Ok(Some(0)));
}

#[test]
fn trailing_bytes_preserve_canonical_prefix_but_do_not_broaden_replay() {
    let base_path = &real_v1_fixture_paths()[0];
    let mut bytes = fs::read(base_path).expect("fixture must be readable");
    let canonical_len = bytes.len();
    bytes.extend_from_slice(b"\xAA\xBB");

    let allow = parse_artifact_allow_trailing(&bytes).expect("allow-trailing parse must succeed");
    match allow {
        ParsedArtifact::V1(parsed) => assert_eq!(parsed.canonical_len, canonical_len),
        ParsedArtifact::V0(_) => panic!("expected v1 fixture at {}", base_path.display()),
    }

    let strict = parse_artifact(&bytes).expect_err("strict parse must reject trailing bytes");
    assert!(matches!(strict, ParseError::LengthMismatch { .. }));

    let replay = parse_replay_frames_legacy0(&bytes)
        .expect_err("legacy replay decode must stay strict on trailing bytes");
    assert!(matches!(replay, ParseError::LengthMismatch { .. }));
}
