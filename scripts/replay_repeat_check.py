#!/usr/bin/env python3
import argparse
import hashlib
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

import inspect_artifact

DEFAULT_RUNS = 5
DEFAULT_BASELINE = Path("artifacts/baseline.bin")
DEFAULT_RUNS_PARENT = Path("artifacts/replay_runs")
DEFAULT_RESET_MODE = "manual"
DEFAULT_STFLASH = "st-flash"
INTERNAL_CAPTURE_MANIFEST = "repeat_capture_internal.txt"
MANIFEST_NAME = "replay_manifest_v1.txt"
SHA_SUMMARY_NAME = "sha256_summary.txt"
CONTRACT_VERSION = "rpl0_capture_v1"
REPO_ROOT = Path(__file__).resolve().parent.parent


class InterruptedCommand(RuntimeError):
    pass


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run repeat replay capture acceptance checks against pinned baseline."
    )
    parser.add_argument("--runs", type=int, default=DEFAULT_RUNS)
    parser.add_argument("--signal-model", required=True)
    parser.add_argument(
        "--reset-mode",
        choices=("manual", "stlink"),
        default=DEFAULT_RESET_MODE,
        help="Reset trigger mode per run: manual (default) or stlink.",
    )
    parser.add_argument(
        "--stflash",
        default=DEFAULT_STFLASH,
        help="st-flash executable path/name for --reset-mode stlink (default: st-flash).",
    )
    parser.add_argument("--baseline", type=Path, default=DEFAULT_BASELINE)
    parser.add_argument(
        "--artifacts-dir",
        type=Path,
        default=DEFAULT_RUNS_PARENT,
        help="Parent directory for run_<timestamp> output directories.",
    )
    parser.add_argument(
        "--serial",
        type=str,
        default=None,
        help="Serial port override (otherwise SERIAL env/defaults apply).",
    )
    return parser.parse_args()


def utc_iso8601_now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def run_stamp_from_now() -> str:
    return datetime.now(timezone.utc).strftime("run_%Y%m%dT%H%M%SZ")


def allocate_run_dir(parent_dir: Path) -> Path:
    base = parent_dir / run_stamp_from_now()
    candidate = base
    suffix = 0
    while candidate.exists():
        suffix += 1
        candidate = parent_dir / f"{base.name}_{suffix:02d}"
    candidate.mkdir(parents=True, exist_ok=False)
    return candidate


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def load_baseline_metadata(path: Path) -> tuple[str, str]:
    artifact = inspect_artifact.parse_artifact(path, allow_trailing=False)
    version = str(artifact["version"])
    schema_hash = "-"
    if artifact["version"] == 1 and artifact["schema_hash"] is not None:
        schema_hash = artifact["schema_hash"].hex()
    return version, schema_hash


def run_cmd(
    cmd: list[str], env: dict[str, str] | None = None, *, stream_output: bool = False
) -> tuple[int, str]:
    if stream_output:
        proc = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            env=env,
        )
        collected: list[str] = []
        try:
            assert proc.stdout is not None
            for line in proc.stdout:
                print(line, end="")
                collected.append(line)
            proc.wait()
        except KeyboardInterrupt as exc:
            proc.terminate()
            try:
                proc.wait(timeout=2)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait(timeout=2)
            raise InterruptedCommand() from exc
        return proc.returncode, "".join(collected)

    proc = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env,
    )
    try:
        out, err = proc.communicate()
    except KeyboardInterrupt as exc:
        proc.terminate()
        try:
            proc.wait(timeout=2)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait(timeout=2)
        raise InterruptedCommand() from exc
    return proc.returncode, (out or "") + (err or "")


def classify_capture_failure(output: str) -> str:
    text = output.lower()
    if "magic not found" in text or "serial open/read error" in text or "short read" in text:
        return "uart_sync_timeout"
    if "hash mismatch" in text:
        return "repeat_hash_mismatch"
    return "verify_failed"


def repo_relative_path(path: Path) -> Path:
    resolved = path.resolve()
    try:
        return resolved.relative_to(REPO_ROOT)
    except ValueError:
        return resolved


def write_manifest(
    path: Path,
    *,
    contract_version: str,
    artifact_version: str,
    schema_hash: str,
    signal_model: str,
    baseline_path: Path,
    baseline_sha: str,
    requested_runs: int,
    completed_runs: int,
    final_status: str,
    failure_class: str,
    baseline_hash_match: bool,
    timestamp_utc: str,
    run_dir: Path,
) -> None:
    path.write_text(
        "\n".join(
            [
                f"contract_version={contract_version}",
                f"artifact_version={artifact_version}",
                f"schema_hash={schema_hash}",
                f"signal_model={signal_model}",
                f"baseline_path={baseline_path}",
                f"baseline_sha256={baseline_sha}",
                f"requested_runs={requested_runs}",
                f"completed_runs={completed_runs}",
                f"final_status={final_status}",
                f"failure_class={failure_class if failure_class else '-'}",
                f"baseline_hash_match={'true' if baseline_hash_match else 'false'}",
                f"timestamp_utc={timestamp_utc}",
                f"run_dir={repo_relative_path(run_dir)}",
                "",
            ]
        ),
        encoding="utf-8",
    )


def write_sha_summary(path: Path, baseline_sha: str, run_hashes: list[tuple[str, Path]]) -> None:
    lines = [f"baseline_sha256 {baseline_sha} baseline"]
    for digest, run_path in run_hashes:
        lines.append(f"{digest} {run_path.name}")
    lines.append("")
    path.write_text("\n".join(lines), encoding="utf-8")


def main() -> int:
    args = parse_args()
    if args.runs <= 0:
        print("FAIL: --runs must be > 0")
        return 2

    parent_dir = args.artifacts_dir
    run_dir = allocate_run_dir(parent_dir)

    manifest_path = run_dir / MANIFEST_NAME
    sha_summary_path = run_dir / SHA_SUMMARY_NAME
    timestamp_utc = utc_iso8601_now()

    baseline_path = args.baseline
    artifact_version = "-"
    schema_hash = "-"
    if not baseline_path.is_file():
        print(f"FAIL: baseline artifact not found: {baseline_path}")
        write_manifest(
            manifest_path,
            contract_version=CONTRACT_VERSION,
            artifact_version=artifact_version,
            schema_hash=schema_hash,
            signal_model=args.signal_model,
            baseline_path=baseline_path,
            baseline_sha="-",
            requested_runs=args.runs,
            completed_runs=0,
            final_status="FAIL",
            failure_class="verify_failed",
            baseline_hash_match=False,
            timestamp_utc=timestamp_utc,
            run_dir=run_dir.resolve(),
        )
        return 1

    try:
        artifact_version, schema_hash = load_baseline_metadata(baseline_path)
    except ValueError as exc:
        print(f"FAIL: invalid baseline artifact ({exc})")
        write_manifest(
            manifest_path,
            contract_version=CONTRACT_VERSION,
            artifact_version=artifact_version,
            schema_hash=schema_hash,
            signal_model=args.signal_model,
            baseline_path=baseline_path,
            baseline_sha="-",
            requested_runs=args.runs,
            completed_runs=0,
            final_status="FAIL",
            failure_class="verify_failed",
            baseline_hash_match=False,
            timestamp_utc=timestamp_utc,
            run_dir=run_dir.resolve(),
        )
        return 1

    baseline_sha = sha256_file(baseline_path)
    completed_runs = 0
    failure_class = ""
    baseline_hash_match = False

    env = os.environ.copy()
    env["PYTHONUNBUFFERED"] = "1"
    if args.serial:
        env["SERIAL"] = args.serial

    capture_cmd = [
        sys.executable,
        "-u",
        "scripts/repeat_capture.py",
        "--runs",
        str(args.runs),
        "--signal-model",
        args.signal_model,
        "--reset-mode",
        args.reset_mode,
        "--stflash",
        args.stflash,
        "--artifacts-dir",
        str(run_dir),
        "--manifest-name",
        INTERNAL_CAPTURE_MANIFEST,
    ]

    print(
        f"Repeat capture: runs={args.runs}, signal_model={args.signal_model}, "
        f"reset_mode={args.reset_mode}"
    )
    if args.reset_mode == "manual":
        print("Press reset once per run after listener readiness line.")
    else:
        print("Using stlink auto-reset per run.")
    try:
        capture_rc, capture_output = run_cmd(capture_cmd, env=env, stream_output=True)
    except InterruptedCommand:
        failure_class = "verify_failed"
        run_files = sorted(run_dir.glob("run_*.bin"))
        completed_runs = len(run_files)
        write_manifest(
            manifest_path,
            contract_version=CONTRACT_VERSION,
            artifact_version=artifact_version,
            schema_hash=schema_hash,
            signal_model=args.signal_model,
            baseline_path=baseline_path,
            baseline_sha=baseline_sha,
            requested_runs=args.runs,
            completed_runs=completed_runs,
            final_status="FAIL",
            failure_class=failure_class,
            baseline_hash_match=False,
            timestamp_utc=timestamp_utc,
            run_dir=run_dir.resolve(),
        )
        print("FAIL: interrupted by operator (KeyboardInterrupt)")
        return 1

    if capture_rc != 0:
        failure_class = classify_capture_failure(capture_output)
        run_files = sorted(run_dir.glob("run_*.bin"))
        completed_runs = len(run_files)
        run_hashes = [(sha256_file(path), path) for path in run_files]
        write_sha_summary(sha_summary_path, baseline_sha, run_hashes)
        write_manifest(
            manifest_path,
            contract_version=CONTRACT_VERSION,
            artifact_version=artifact_version,
            schema_hash=schema_hash,
            signal_model=args.signal_model,
            baseline_path=baseline_path,
            baseline_sha=baseline_sha,
            requested_runs=args.runs,
            completed_runs=completed_runs,
            final_status="FAIL",
            failure_class=failure_class,
            baseline_hash_match=False,
            timestamp_utc=timestamp_utc,
            run_dir=run_dir.resolve(),
        )
        sys.stdout.write(capture_output)
        print(f"FAIL: {failure_class}")
        return 1

    run_files = sorted(run_dir.glob("run_*.bin"))
    completed_runs = len(run_files)
    print(f"INFO: capture phase complete, validating {completed_runs} artifacts")
    if completed_runs == 0:
        failure_class = "verify_failed"
        write_manifest(
            manifest_path,
            contract_version=CONTRACT_VERSION,
            artifact_version=artifact_version,
            schema_hash=schema_hash,
            signal_model=args.signal_model,
            baseline_path=baseline_path,
            baseline_sha=baseline_sha,
            requested_runs=args.runs,
            completed_runs=completed_runs,
            final_status="FAIL",
            failure_class=failure_class,
            baseline_hash_match=False,
            timestamp_utc=timestamp_utc,
            run_dir=run_dir.resolve(),
        )
        print("FAIL: verify_failed (no run artifacts produced)")
        return 1

    run_hashes: list[tuple[str, Path]] = []
    for run_path in run_files:
        verify_cmd = [
            sys.executable,
            "scripts/artifact_tool.py",
            "verify",
            str(run_path),
            "--signal-model",
            args.signal_model,
        ]
        try:
            verify_rc, verify_out = run_cmd(verify_cmd, env=env)
        except InterruptedCommand:
            failure_class = "verify_failed"
            break
        if verify_rc != 0:
            failure_class = "verify_failed"
            sys.stdout.write(verify_out)
            break

        compare_cmd = [
            sys.executable,
            "scripts/artifact_tool.py",
            "compare",
            str(baseline_path),
            str(run_path),
        ]
        try:
            compare_rc, compare_out = run_cmd(compare_cmd, env=env)
        except InterruptedCommand:
            failure_class = "baseline_compare_failed"
            break
        if compare_rc != 0:
            failure_class = "baseline_compare_failed"
            sys.stdout.write(compare_out)
            break

        run_hashes.append((sha256_file(run_path), run_path))

    if not run_hashes:
        if not failure_class:
            failure_class = "verify_failed"
        write_manifest(
            manifest_path,
            contract_version=CONTRACT_VERSION,
            artifact_version=artifact_version,
            schema_hash=schema_hash,
            signal_model=args.signal_model,
            baseline_path=baseline_path,
            baseline_sha=baseline_sha,
            requested_runs=args.runs,
            completed_runs=completed_runs,
            final_status="FAIL",
            failure_class=failure_class,
            baseline_hash_match=False,
            timestamp_utc=timestamp_utc,
            run_dir=run_dir.resolve(),
        )
        print(f"FAIL: {failure_class}")
        return 1

    write_sha_summary(sha_summary_path, baseline_sha, run_hashes)

    if failure_class:
        write_manifest(
            manifest_path,
            contract_version=CONTRACT_VERSION,
            artifact_version=artifact_version,
            schema_hash=schema_hash,
            signal_model=args.signal_model,
            baseline_path=baseline_path,
            baseline_sha=baseline_sha,
            requested_runs=args.runs,
            completed_runs=completed_runs,
            final_status="FAIL",
            failure_class=failure_class,
            baseline_hash_match=False,
            timestamp_utc=timestamp_utc,
            run_dir=run_dir.resolve(),
        )
        print(f"FAIL: {failure_class}")
        return 1

    unique_hashes = {digest for digest, _ in run_hashes}
    baseline_hash_match = all(digest == baseline_sha for digest, _ in run_hashes)
    if len(unique_hashes) != 1 or not baseline_hash_match:
        failure_class = "repeat_hash_mismatch"
        write_manifest(
            manifest_path,
            contract_version=CONTRACT_VERSION,
            artifact_version=artifact_version,
            schema_hash=schema_hash,
            signal_model=args.signal_model,
            baseline_path=baseline_path,
            baseline_sha=baseline_sha,
            requested_runs=args.runs,
            completed_runs=completed_runs,
            final_status="FAIL",
            failure_class=failure_class,
            baseline_hash_match=baseline_hash_match,
            timestamp_utc=timestamp_utc,
            run_dir=run_dir.resolve(),
        )
        print("FAIL: repeat_hash_mismatch")
        return 1

    write_manifest(
        manifest_path,
        contract_version=CONTRACT_VERSION,
        artifact_version=artifact_version,
        schema_hash=schema_hash,
        signal_model=args.signal_model,
        baseline_path=baseline_path,
        baseline_sha=baseline_sha,
        requested_runs=args.runs,
        completed_runs=completed_runs,
        final_status="PASS",
        failure_class="",
        baseline_hash_match=True,
        timestamp_utc=timestamp_utc,
        run_dir=run_dir.resolve(),
    )

    print(f"PASS: replay repeat check ({completed_runs} runs)")
    print(f"run_dir: {run_dir}")
    print(f"manifest: {manifest_path}")
    print(f"sha_summary: {sha_summary_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
