#!/usr/bin/env python3
import argparse
import hashlib
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

DEFAULT_RUNS = 3
DEFAULT_RUNS_PARENT = Path("artifacts/fw_repeat_runs")
DEFAULT_RESET_MODE = "manual"
DEFAULT_STFLASH = "st-flash"
DEFAULT_TIMEOUT = 10.0
INTERNAL_CAPTURE_MANIFEST = "repeat_capture_transport.txt"
MANIFEST_NAME = "interval_capture_manifest_v1.txt"
CSV_SHA_SUMMARY_NAME = "csv_sha256_summary.txt"
IMPORTED_SHA_SUMMARY_NAME = "imported_artifact_sha256_summary.txt"
CONTRACT_VERSION = "interval_capture_v1"
REPO_ROOT = Path(__file__).resolve().parent.parent


class InterruptedCommand(RuntimeError):
    pass


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run repeat STM32 interval CSV capture acceptance checks."
    )
    parser.add_argument("--runs", type=int, default=DEFAULT_RUNS)
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
    parser.add_argument(
        "--timeout",
        type=float,
        default=DEFAULT_TIMEOUT,
        help="UART emission/read timeout in seconds per run (default: 10.0).",
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


def csv_row_count(path: Path) -> int:
    with path.open("r", encoding="utf-8", newline="") as handle:
        line_count = sum(1 for _ in handle)
    return max(0, line_count - 1)


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
    if "no uart emission within timeout" in text or "serial read timed out before capture completed" in text:
        return "uart_emission_timeout"
    if "no state preamble observed" in text or "capture produced no state line" in text:
        return "missing_state_preamble"
    if "explicit incomplete or malformed state preamble" in text or "capture did not complete:" in text:
        return "malformed_state_preamble"
    if "serial open/read error" in text or "could not open port" in text:
        return "serial_open_read_error"
    return "transport_failed"


def repo_relative_path(path: Path) -> Path:
    resolved = path.resolve()
    try:
        return resolved.relative_to(REPO_ROOT)
    except ValueError:
        return resolved


def write_sha_summary(path: Path, label: str, hashes: list[tuple[str, Path]]) -> None:
    lines = [f"# {label}"]
    for digest, file_path in hashes:
        lines.append(f"{digest} {file_path.name}")
    lines.append("")
    path.write_text("\n".join(lines), encoding="utf-8")


def write_manifest(
    path: Path,
    *,
    reset_mode: str,
    requested_runs: int,
    completed_runs: int,
    final_status: str,
    failure_class: str,
    csv_hash_stable: bool,
    imported_hash_stable: bool,
    timestamp_utc: str,
    run_dir: Path,
    run_records: list[dict[str, str | int]],
) -> None:
    lines = [
        f"contract_version={CONTRACT_VERSION}",
        "transport_contract=state_then_interval_csv",
        "csv_header=index,interval_us",
        "expected_rows=138",
        f"reset_mode={reset_mode}",
        "imported_artifacts=true",
        f"requested_runs={requested_runs}",
        f"completed_runs={completed_runs}",
        f"final_status={final_status}",
        f"failure_class={failure_class if failure_class else '-'}",
        f"csv_hash_stable={'true' if csv_hash_stable else 'false'}",
        f"imported_hash_stable={'true' if imported_hash_stable else 'false'}",
        f"timestamp_utc={timestamp_utc}",
        f"run_dir={repo_relative_path(run_dir)}",
        "",
        "# fields: run csv rows csv_sha256 imported imported_sha256 status",
    ]
    for record in run_records:
        lines.append(
            "run={run} csv={csv} rows={rows} csv_sha256={csv_sha256} "
            "imported={imported} imported_sha256={imported_sha256} status={status}".format(
                **record
            )
        )
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
    csv_sha_summary_path = run_dir / CSV_SHA_SUMMARY_NAME
    imported_sha_summary_path = run_dir / IMPORTED_SHA_SUMMARY_NAME
    timestamp_utc = utc_iso8601_now()

    env = os.environ.copy()
    env["PYTHONUNBUFFERED"] = "1"
    if args.serial:
        env["SERIAL"] = args.serial

    capture_cmd = [
        sys.executable,
        "-u",
        "scripts/repeat_capture.py",
        "--contract",
        "csv",
        "--runs",
        str(args.runs),
        "--reset-mode",
        args.reset_mode,
        "--stflash",
        args.stflash,
        "--timeout",
        str(args.timeout),
        "--artifacts-dir",
        str(run_dir),
        "--manifest-name",
        INTERNAL_CAPTURE_MANIFEST,
    ]

    print(
        f"Interval capture: runs={args.runs}, reset_mode={args.reset_mode}, "
        f"timeout={args.timeout}s"
    )
    if args.reset_mode == "manual":
        print("Canonical gate reset mode: manual hardware reset, once per run.")
    else:
        print("Non-canonical reset mode selected: stlink auto-reset.")

    run_records: list[dict[str, str | int]] = []
    try:
        capture_rc, capture_output = run_cmd(capture_cmd, env=env, stream_output=True)
    except InterruptedCommand:
        write_manifest(
            manifest_path,
            reset_mode=args.reset_mode,
            requested_runs=args.runs,
            completed_runs=0,
            final_status="FAIL",
            failure_class="interrupted",
            csv_hash_stable=False,
            imported_hash_stable=False,
            timestamp_utc=timestamp_utc,
            run_dir=run_dir.resolve(),
            run_records=run_records,
        )
        print("FAIL: interrupted by operator (KeyboardInterrupt)")
        return 1

    csv_files = sorted(run_dir.glob("run_*.csv"))
    completed_runs = len(csv_files)

    if capture_rc != 0:
        write_manifest(
            manifest_path,
            reset_mode=args.reset_mode,
            requested_runs=args.runs,
            completed_runs=completed_runs,
            final_status="FAIL",
            failure_class=classify_capture_failure(capture_output),
            csv_hash_stable=False,
            imported_hash_stable=False,
            timestamp_utc=timestamp_utc,
            run_dir=run_dir.resolve(),
            run_records=run_records,
        )
        print(f"FAIL: {classify_capture_failure(capture_output)}")
        return 1

    if completed_runs == 0:
        write_manifest(
            manifest_path,
            reset_mode=args.reset_mode,
            requested_runs=args.runs,
            completed_runs=0,
            final_status="FAIL",
            failure_class="transport_failed",
            csv_hash_stable=False,
            imported_hash_stable=False,
            timestamp_utc=timestamp_utc,
            run_dir=run_dir.resolve(),
            run_records=run_records,
        )
        print("FAIL: transport_failed (no CSV artifacts produced)")
        return 1

    csv_hashes: list[tuple[str, Path]] = []
    imported_hashes: list[tuple[str, Path]] = []
    failure_class = ""

    for csv_path in csv_files:
        validate_cmd = [
            "cargo",
            "run",
            "-q",
            "-p",
            "replay-host",
            "--",
            "validate-interval-csv",
            str(csv_path),
        ]
        try:
            validate_rc, validate_out = run_cmd(validate_cmd, env=env)
        except InterruptedCommand:
            failure_class = "interrupted"
            break
        if validate_rc != 0:
            failure_class = "validator_failed"
            sys.stdout.write(validate_out)
            break

        imported_path = csv_path.with_suffix(".imported.rpl")
        import_cmd = [
            "cargo",
            "run",
            "-q",
            "-p",
            "replay-host",
            "--",
            "import-interval-csv",
            str(csv_path),
            str(imported_path),
        ]
        try:
            import_rc, import_out = run_cmd(import_cmd, env=env)
        except InterruptedCommand:
            failure_class = "interrupted"
            break
        if import_rc != 0:
            failure_class = "import_failed"
            sys.stdout.write(import_out)
            break

        csv_digest = sha256_file(csv_path)
        imported_digest = sha256_file(imported_path)
        csv_hashes.append((csv_digest, csv_path))
        imported_hashes.append((imported_digest, imported_path))
        run_records.append(
            {
                "run": len(run_records) + 1,
                "csv": csv_path.name,
                "rows": csv_row_count(csv_path),
                "csv_sha256": csv_digest,
                "imported": imported_path.name,
                "imported_sha256": imported_digest,
                "status": "PASS",
            }
        )

    csv_hash_stable = bool(csv_hashes) and len({digest for digest, _ in csv_hashes}) == 1
    imported_hash_stable = bool(imported_hashes) and len({digest for digest, _ in imported_hashes}) == 1

    if not failure_class and not csv_hash_stable:
        failure_class = "repeat_csv_hash_mismatch"
    if not failure_class and not imported_hash_stable:
        failure_class = "repeat_import_hash_mismatch"

    write_sha_summary(csv_sha_summary_path, "csv_sha256", csv_hashes)
    write_sha_summary(imported_sha_summary_path, "imported_artifact_sha256", imported_hashes)

    final_status = "PASS" if not failure_class else "FAIL"
    write_manifest(
        manifest_path,
        reset_mode=args.reset_mode,
        requested_runs=args.runs,
        completed_runs=len(run_records),
        final_status=final_status,
        failure_class=failure_class,
        csv_hash_stable=csv_hash_stable,
        imported_hash_stable=imported_hash_stable,
        timestamp_utc=timestamp_utc,
        run_dir=run_dir.resolve(),
        run_records=run_records,
    )

    if failure_class:
        print(f"FAIL: {failure_class}")
        return 1

    print(f"PASS: validated {len(run_records)} captures; CSV and imported artifact hashes are stable")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
