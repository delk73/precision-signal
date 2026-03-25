#!/usr/bin/env python3
import argparse
import hashlib
import os
import subprocess
import sys
from pathlib import Path

DEFAULT_RUNS = 5
DEFAULT_SERIAL = "/dev/ttyACM0"
DEFAULT_RESET_MODE = "manual"
DEFAULT_STFLASH = "st-flash"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Repeat UART artifact capture and verify byte-for-byte repeatability."
    )
    parser.add_argument(
        "--runs",
        type=int,
        default=DEFAULT_RUNS,
        help="Number of capture runs (default: 5).",
    )
    parser.add_argument(
        "--artifacts-dir",
        type=Path,
        default=Path("artifacts"),
        help="Directory for run_N.bin and manifest.txt (default: artifacts).",
    )
    parser.add_argument(
        "--signal-model",
        required=True,
        help="Signal model for capture validation (e.g. phase8, ramp). Required.",
    )
    parser.add_argument(
        "--manifest-name",
        default="manifest.txt",
        help="Manifest filename within artifacts-dir (default: manifest.txt).",
    )
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
    return parser.parse_args()


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def append_manifest(
    manifest_path: Path,
    run_no: int,
    file_name: str,
    byte_count: int,
    sha256_hex: str,
    status: str,
) -> None:
    with manifest_path.open("a", encoding="utf-8") as mf:
        mf.write(
            f"run={run_no:02d} file={file_name} bytes={byte_count} "
            f"sha256={sha256_hex} status={status}\n"
        )


def trigger_stlink_reset(stflash: str) -> tuple[int, str]:
    proc = subprocess.run(
        [stflash, "--connect-under-reset", "--freq=200K", "reset"],
        capture_output=True,
        text=True,
    )
    return proc.returncode, (proc.stdout or "") + (proc.stderr or "")


def run_capture(
    serial_port: str, out_path: Path, reset_mode: str, stflash: str
) -> tuple[int, str]:
    env = os.environ.copy()
    env["SERIAL"] = serial_port
    cmd = [
        sys.executable,
        "scripts/artifact_tool.py",
        "capture",
        "--quick",
        "--out",
        str(out_path),
    ]
    proc = subprocess.Popen(
        cmd,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    lines: list[str] = []
    assert proc.stdout is not None
    auto_reset_fired = False
    for line in proc.stdout:
        print(line, end="")
        lines.append(line)
        if (
            reset_mode == "stlink"
            and not auto_reset_fired
            and "Listener active; press reset now" in line
        ):
            rc, out = trigger_stlink_reset(stflash)
            lines.append(out)
            if out:
                print(out, end="" if out.endswith("\n") else "\n")
            if rc != 0:
                proc.terminate()
                try:
                    proc.wait(timeout=2)
                except subprocess.TimeoutExpired:
                    proc.kill()
                    proc.wait(timeout=2)
                lines.append("Failed: stlink reset command failed\n")
                return 31, "".join(lines)
            auto_reset_fired = True
    proc.wait()
    return proc.returncode, "".join(lines)


def extract_failure_reason(output: str) -> str:
    for line in output.splitlines():
        if line.startswith("Failed:"):
            text = line.split(":", 1)[1].strip().lower()
            if "magic not found within" in text:
                return "UART capture failed (no replay header observed after reset)"
            if "serial" in text:
                return "UART capture failed (serial open/read error)"
            return f"UART capture failed ({text})"
    return "UART capture failed (unknown cause)"


def main() -> int:
    args = parse_args()
    if args.runs <= 0:
        print("FAIL run 00: invalid runs value")
        return 2

    serial_port = os.environ.get("SERIAL", DEFAULT_SERIAL)
    artifacts_dir = args.artifacts_dir
    artifacts_dir.mkdir(parents=True, exist_ok=True)
    manifest_path = artifacts_dir / args.manifest_name

    with manifest_path.open("w", encoding="utf-8") as mf:
        mf.write(f"# {args.signal_model} repeat_capture manifest\n")
        mf.write(f"signal_model={args.signal_model}\n")
        mf.write(f"serial={serial_port} runs={args.runs}\n")
        mf.write(f"reset_mode={args.reset_mode}\n")
        mf.write("# fields: run file bytes sha256 status\n")

    reference_hash = None
    name_width = max(2, len(str(args.runs)))

    for run_no in range(1, args.runs + 1):
        out_name = f"run_{run_no:0{name_width}d}.bin"
        out_path = artifacts_dir / out_name

        if args.reset_mode == "manual":
            prompt = "wait for 'Listener active; press reset now', then press reset once."
        else:
            prompt = "waiting for listener readiness; reset will be triggered via stlink."
        print(f"Run {run_no:0{name_width}d}/{args.runs:0{name_width}d}: {prompt}")
        rc, output = run_capture(
            serial_port, out_path, args.reset_mode, args.stflash
        )
        if rc != 0:
            reason = extract_failure_reason(output)
            append_manifest(
                manifest_path,
                run_no,
                out_name,
                out_path.stat().st_size if out_path.exists() else 0,
                "-",
                f"FAIL({reason})",
            )
            print(f"FAIL run {run_no:02d}: {reason}")
            return 1

        if not out_path.exists():
            reason = "UART capture failed"
            append_manifest(manifest_path, run_no, out_name, 0, "-", f"FAIL({reason})")
            print(f"FAIL run {run_no:02d}: {reason}")
            return 1

        byte_count = out_path.stat().st_size
        digest = sha256_file(out_path)

        if reference_hash is None:
            reference_hash = digest
            status = "PASS"
        elif digest != reference_hash:
            status = "FAIL(hash mismatch)"
            append_manifest(manifest_path, run_no, out_name, byte_count, digest, status)
            print(f"FAIL run {run_no:02d}: hash mismatch")
            return 1
        else:
            status = "PASS"

        append_manifest(manifest_path, run_no, out_name, byte_count, digest, status)
        print(
            f"PASS run {run_no:02d}: bytes={byte_count} sha256={digest}"
        )

    print(f"PASS all {args.runs} runs: hashes identical")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
