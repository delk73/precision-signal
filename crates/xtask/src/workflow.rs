use crate::firmware::run_firmware_build_check;
use std::path::Path;
use std::process::Command;

const EXIT_OK: i32 = 0;
const EXIT_FAIL: i32 = 1;
const EXIT_USAGE: i32 = 2;

pub(crate) fn run(args: Vec<String>, repo_root: &Path) -> i32 {
    if args.is_empty() {
        eprintln!("{}", usage());
        return EXIT_USAGE;
    }

    match args[0].as_str() {
        "doc-link-check" => run_doc_link_check(repo_root),
        "fixture-drift-check" => run_fixture_drift_check(repo_root),
        "parser-tests" => run_parser_tests(repo_root),
        "replay-tool-tests" => run_replay_tool_tests(repo_root),
        "replay-tests" => run_replay_tests(repo_root),
        "demo-evidence-package" => run_demo_evidence_package(repo_root),
        "ci-local" => run_ci_local(repo_root),
        _ => {
            eprintln!("{}", usage());
            EXIT_USAGE
        }
    }
}

fn run_ci_local(repo_root: &Path) -> i32 {
    for step in [
        run_doc_link_check as fn(&Path) -> i32,
        run_firmware_build_check,
        run_workspace_tests,
        run_replay_tests,
        run_gate,
        run_fixture_drift_check,
    ] {
        let exit = step(repo_root);
        if exit != EXIT_OK {
            return exit;
        }
    }
    EXIT_OK
}

fn run_doc_link_check(repo_root: &Path) -> i32 {
    run_python(repo_root, &["scripts/check_doc_links.py"])
}

fn run_fixture_drift_check(repo_root: &Path) -> i32 {
    for args in [
        [
            "scripts/generate_demo_v3_fixtures.py",
            "--out-dir",
            "artifacts/demo_v3",
        ],
        [
            "scripts/generate_demo_v4_fixtures.py",
            "--out-dir",
            "artifacts/demo_v4",
        ],
        [
            "scripts/generate_demo_v5_fixtures.py",
            "--out-dir",
            "artifacts/demo_v5",
        ],
    ] {
        let exit = run_python(repo_root, &args);
        if exit != EXIT_OK {
            return exit;
        }
    }

    run_command(
        Command::new("git")
            .args([
                "diff",
                "--exit-code",
                "artifacts/demo_v3",
                "artifacts/demo_v4",
                "artifacts/demo_v5",
            ])
            .current_dir(repo_root),
    )
}

fn run_parser_tests(repo_root: &Path) -> i32 {
    for args in [
        ["scripts/test_artifact_parser_adversarial.py"],
        ["scripts/test_artifact_parser_valid_v1.py"],
        ["scripts/test_artifact_parser_mutation_corpus.py"],
    ] {
        let exit = run_python(repo_root, &args);
        if exit != EXIT_OK {
            return exit;
        }
    }
    EXIT_OK
}

fn run_replay_tool_tests(repo_root: &Path) -> i32 {
    for args in [
        ["scripts/test_artifact_tool_verify.py"],
        ["scripts/test_artifact_tool_hash.py"],
        ["scripts/test_doc_link_check.py"],
        ["scripts/test_artifact_diff.py"],
        ["scripts/test_demo_v3_fixtures.py"],
        ["tests/test_demo_v4_region_attribution.py"],
        ["tests/test_demo_v5_evolution.py"],
        ["scripts/test_compare_artifact.py"],
    ] {
        let exit = run_python(repo_root, &args);
        if exit != EXIT_OK {
            return exit;
        }
    }
    EXIT_OK
}

fn run_replay_tests(repo_root: &Path) -> i32 {
    let exit = run_parser_tests(repo_root);
    if exit != EXIT_OK {
        return exit;
    }
    run_replay_tool_tests(repo_root)
}

fn run_demo_evidence_package(repo_root: &Path) -> i32 {
    run_python(repo_root, &["scripts/package_demo_evidence.py"])
}

fn run_workspace_tests(repo_root: &Path) -> i32 {
    run_command(
        Command::new("cargo")
            .args(["test", "--workspace", "--locked"])
            .current_dir(repo_root),
    )
}

fn run_gate(repo_root: &Path) -> i32 {
    run_command(
        Command::new("make")
            .arg("gate")
            .current_dir(repo_root),
    )
}

fn run_python(repo_root: &Path, args: &[&str]) -> i32 {
    let mut command = Command::new("python3");
    command.env("PYTHONPATH", ".");
    command.args(args).current_dir(repo_root);
    run_command(&mut command)
}

fn run_command(command: &mut Command) -> i32 {
    match command.status() {
        Ok(status) => status.code().unwrap_or(EXIT_FAIL),
        Err(err) => {
            eprintln!("{err}");
            EXIT_FAIL
        }
    }
}

fn usage() -> String {
    "usage: cargo xtask workflow doc-link-check\n       cargo xtask workflow fixture-drift-check\n       cargo xtask workflow parser-tests\n       cargo xtask workflow replay-tool-tests\n       cargo xtask workflow replay-tests\n       cargo xtask workflow demo-evidence-package\n       cargo xtask workflow ci-local"
        .to_string()
}
