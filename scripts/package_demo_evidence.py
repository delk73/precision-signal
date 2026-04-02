#!/usr/bin/env python3
import argparse
import hashlib
import os
import shutil
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
BUNDLE_ROOT = REPO_ROOT / "artifacts" / "demo_evidence"
INPUTS_DIR = BUNDLE_ROOT / "inputs"
RETAINED_DIR = BUNDLE_ROOT / "retained"
GENERATED_DIR = BUNDLE_ROOT / "_generated"
GENERATED_OUTPUTS_DIR = GENERATED_DIR / "outputs"
RETAINED_OUTPUTS_DIR = RETAINED_DIR / "outputs"

BASELINE_CSV = INPUTS_DIR / "baseline_capture.csv.fixture"
PERTURBED_CSV = INPUTS_DIR / "perturbed_frame017_plus1.csv.fixture"
GENERATED_BASELINE_RPL = GENERATED_OUTPUTS_DIR / "baseline.rpl"
GENERATED_PERTURBED_RPL = GENERATED_OUTPUTS_DIR / "perturbed_frame017_plus1.rpl"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Rebuild the canonical replay evidence bundle and verify it matches retained outputs."
    )
    parser.add_argument(
        "--refresh-retained",
        action="store_true",
        help="Overwrite the retained bundle with freshly generated outputs.",
    )
    return parser.parse_args()


def repo_rel(path: Path) -> str:
    return path.relative_to(REPO_ROOT).as_posix()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def run_step(
    transcript_lines: list[str],
    commands: list[str],
    output_name: str,
    display_cmd: str,
    argv: list[str],
    env_extra: dict[str, str] | None = None,
) -> str:
    env = os.environ.copy()
    if env_extra:
        env.update(env_extra)

    proc = subprocess.run(
        argv,
        cwd=REPO_ROOT,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )

    output_text = (proc.stdout or "") + (proc.stderr or "")
    transcript_lines.append(f"$ {display_cmd}\n")
    if output_text:
        transcript_lines.append(output_text)
        if not output_text.endswith("\n"):
            transcript_lines.append("\n")
    commands.append(display_cmd)

    output_path = GENERATED_OUTPUTS_DIR / output_name
    output_path.write_text(output_text, encoding="utf-8")

    if proc.returncode != 0:
        raise SystemExit(
            f"FAIL: command exited with {proc.returncode}: {display_cmd}\n"
            f"see {repo_rel(output_path)}"
        )

    return output_text


def ensure_inputs_exist() -> None:
    missing = [path for path in (BASELINE_CSV, PERTURBED_CSV) if not path.is_file()]
    if missing:
        formatted = ", ".join(repo_rel(path) for path in missing)
        raise SystemExit(f"FAIL: missing evidence input fixture(s): {formatted}")


def write_manifest() -> None:
    manifest = "\n".join(
        [
            "bundle_version=phase5_demo_evidence_v1",
            "operator_command=make demo-evidence-package",
            f"bundle_root={repo_rel(BUNDLE_ROOT)}",
            f"retained_dir={repo_rel(RETAINED_DIR)}",
            f"generated_dir={repo_rel(GENERATED_DIR)}",
            f"input_baseline={repo_rel(BASELINE_CSV)}",
            f"input_baseline_sha256={sha256_file(BASELINE_CSV)}",
            f"input_perturbed={repo_rel(PERTURBED_CSV)}",
            f"input_perturbed_sha256={sha256_file(PERTURBED_CSV)}",
            f"generated_baseline_artifact={repo_rel(GENERATED_BASELINE_RPL)}",
            f"generated_baseline_sha256={sha256_file(GENERATED_BASELINE_RPL)}",
            f"generated_perturbed_artifact={repo_rel(GENERATED_PERTURBED_RPL)}",
            f"generated_perturbed_sha256={sha256_file(GENERATED_PERTURBED_RPL)}",
            "expected_replay_diff_identical=no divergence",
            "expected_replay_diff_perturbed=first divergence at frame 17",
            f"commands_record={repo_rel(RETAINED_DIR / 'commands.txt')}",
            f"transcript_record={repo_rel(RETAINED_DIR / 'transcript.txt')}",
            "",
        ]
    )
    (GENERATED_DIR / "manifest.txt").write_text(manifest, encoding="utf-8")


def copy_generated_to_retained() -> None:
    RETAINED_OUTPUTS_DIR.mkdir(parents=True, exist_ok=True)
    for src in GENERATED_OUTPUTS_DIR.iterdir():
        if src.is_file():
            shutil.copyfile(src, RETAINED_OUTPUTS_DIR / src.name)
    shutil.copyfile(GENERATED_DIR / "commands.txt", RETAINED_DIR / "commands.txt")
    shutil.copyfile(GENERATED_DIR / "transcript.txt", RETAINED_DIR / "transcript.txt")
    shutil.copyfile(GENERATED_DIR / "manifest.txt", RETAINED_DIR / "manifest.txt")


def verify_retained_bundle() -> None:
    expected_files = [
        "artifact_diff_identical.txt",
        "artifact_diff_perturbed.txt",
        "baseline.rpl",
        "hash_baseline.txt",
        "hash_perturbed.txt",
        "import_baseline.txt",
        "import_perturbed.txt",
        "perturbed_frame017_plus1.rpl",
        "replay_diff_identical.txt",
        "replay_diff_perturbed.txt",
        "validate_baseline.txt",
        "validate_perturbed.txt",
    ]

    for name in expected_files:
        generated = GENERATED_OUTPUTS_DIR / name
        retained = RETAINED_OUTPUTS_DIR / name
        if not retained.is_file():
            raise SystemExit(f"FAIL: retained file missing: {repo_rel(retained)}")
        if generated.read_bytes() != retained.read_bytes():
            raise SystemExit(f"FAIL: retained output drift: {repo_rel(retained)}")

    for name in ("commands.txt", "transcript.txt", "manifest.txt"):
        generated = GENERATED_DIR / name
        retained = RETAINED_DIR / name
        if not retained.is_file():
            raise SystemExit(f"FAIL: retained file missing: {repo_rel(retained)}")
        if generated.read_bytes() != retained.read_bytes():
            raise SystemExit(f"FAIL: retained output drift: {repo_rel(retained)}")


def main() -> int:
    args = parse_args()
    ensure_inputs_exist()

    if GENERATED_DIR.exists():
        shutil.rmtree(GENERATED_DIR)
    GENERATED_OUTPUTS_DIR.mkdir(parents=True, exist_ok=True)

    transcript_lines: list[str] = []
    commands: list[str] = []

    run_step(
        transcript_lines,
        commands,
        "validate_baseline.txt",
        f"cargo run -q -p replay-host -- validate-interval-csv {repo_rel(BASELINE_CSV)}",
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "replay-host",
            "--",
            "validate-interval-csv",
            repo_rel(BASELINE_CSV),
        ],
    )
    run_step(
        transcript_lines,
        commands,
        "validate_perturbed.txt",
        f"cargo run -q -p replay-host -- validate-interval-csv {repo_rel(PERTURBED_CSV)}",
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "replay-host",
            "--",
            "validate-interval-csv",
            repo_rel(PERTURBED_CSV),
        ],
    )
    run_step(
        transcript_lines,
        commands,
        "import_baseline.txt",
        "cargo run -q -p replay-host -- import-interval-csv "
        f"{repo_rel(BASELINE_CSV)} {repo_rel(GENERATED_BASELINE_RPL)}",
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "replay-host",
            "--",
            "import-interval-csv",
            repo_rel(BASELINE_CSV),
            repo_rel(GENERATED_BASELINE_RPL),
        ],
    )
    run_step(
        transcript_lines,
        commands,
        "import_perturbed.txt",
        "cargo run -q -p replay-host -- import-interval-csv "
        f"{repo_rel(PERTURBED_CSV)} {repo_rel(GENERATED_PERTURBED_RPL)}",
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "replay-host",
            "--",
            "import-interval-csv",
            repo_rel(PERTURBED_CSV),
            repo_rel(GENERATED_PERTURBED_RPL),
        ],
    )
    run_step(
        transcript_lines,
        commands,
        "hash_baseline.txt",
        f"PYTHONPATH=. python3 scripts/artifact_tool.py hash {repo_rel(GENERATED_BASELINE_RPL)}",
        [
            "python3",
            "scripts/artifact_tool.py",
            "hash",
            repo_rel(GENERATED_BASELINE_RPL),
        ],
        env_extra={"PYTHONPATH": "."},
    )
    run_step(
        transcript_lines,
        commands,
        "hash_perturbed.txt",
        f"PYTHONPATH=. python3 scripts/artifact_tool.py hash {repo_rel(GENERATED_PERTURBED_RPL)}",
        [
            "python3",
            "scripts/artifact_tool.py",
            "hash",
            repo_rel(GENERATED_PERTURBED_RPL),
        ],
        env_extra={"PYTHONPATH": "."},
    )
    replay_identical = run_step(
        transcript_lines,
        commands,
        "replay_diff_identical.txt",
        "cargo run -q -p replay-host -- diff "
        f"{repo_rel(GENERATED_BASELINE_RPL)} {repo_rel(GENERATED_BASELINE_RPL)}",
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "replay-host",
            "--",
            "diff",
            repo_rel(GENERATED_BASELINE_RPL),
            repo_rel(GENERATED_BASELINE_RPL),
        ],
    )
    replay_perturbed = run_step(
        transcript_lines,
        commands,
        "replay_diff_perturbed.txt",
        "cargo run -q -p replay-host -- diff "
        f"{repo_rel(GENERATED_BASELINE_RPL)} {repo_rel(GENERATED_PERTURBED_RPL)}",
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "replay-host",
            "--",
            "diff",
            repo_rel(GENERATED_BASELINE_RPL),
            repo_rel(GENERATED_PERTURBED_RPL),
        ],
    )
    artifact_identical = run_step(
        transcript_lines,
        commands,
        "artifact_diff_identical.txt",
        "PYTHONPATH=. python3 scripts/artifact_diff.py "
        f"{repo_rel(GENERATED_BASELINE_RPL)} {repo_rel(GENERATED_BASELINE_RPL)}",
        [
            "python3",
            "scripts/artifact_diff.py",
            repo_rel(GENERATED_BASELINE_RPL),
            repo_rel(GENERATED_BASELINE_RPL),
        ],
        env_extra={"PYTHONPATH": "."},
    )
    artifact_perturbed = run_step(
        transcript_lines,
        commands,
        "artifact_diff_perturbed.txt",
        "PYTHONPATH=. python3 scripts/artifact_diff.py "
        f"{repo_rel(GENERATED_BASELINE_RPL)} {repo_rel(GENERATED_PERTURBED_RPL)}",
        [
            "python3",
            "scripts/artifact_diff.py",
            repo_rel(GENERATED_BASELINE_RPL),
            repo_rel(GENERATED_PERTURBED_RPL),
        ],
        env_extra={"PYTHONPATH": "."},
    )

    if replay_identical != "no divergence\n":
        raise SystemExit("FAIL: identical replay diff did not report 'no divergence'")
    if replay_perturbed != "first divergence at frame 17\n":
        raise SystemExit(
            "FAIL: perturbed replay diff did not report 'first divergence at frame 17'"
        )
    if "first_divergence_frame: none\n" not in artifact_identical:
        raise SystemExit("FAIL: identical artifact diff lost no-divergence summary")
    if "first_divergence_frame: 17\n" not in artifact_perturbed:
        raise SystemExit("FAIL: perturbed artifact diff lost frame-17 summary")

    (GENERATED_DIR / "commands.txt").write_text("\n".join(commands) + "\n", encoding="utf-8")
    (GENERATED_DIR / "transcript.txt").write_text("".join(transcript_lines), encoding="utf-8")
    write_manifest()

    if args.refresh_retained:
        copy_generated_to_retained()
        print(f"retained bundle refreshed: {repo_rel(RETAINED_DIR)}")
        return 0

    verify_retained_bundle()
    print(f"generated bundle: {repo_rel(GENERATED_DIR)}")
    print(f"retained bundle matches: {repo_rel(RETAINED_DIR)}")
    print("replay diff identical: no divergence")
    print("replay diff perturbed: first divergence at frame 17")
    return 0


if __name__ == "__main__":
    sys.exit(main())
