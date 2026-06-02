#!/usr/bin/env python3
"""Run the hardware-backed firmware gate."""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import time
import select
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--serial", required=True)
    parser.add_argument("--reset-mode", choices=("stlink", "manual"), default="stlink")
    parser.add_argument(
        "--allow-manual-reset",
        action="store_true",
        help="Legacy/debug escape hatch only; active release and board bring-up use stlink.",
    )
    parser.add_argument("--capture-timeout", required=True)
    parser.add_argument("--repeat-runs", required=True)
    parser.add_argument("--signal-model", required=True)
    parser.add_argument("--replay-run", required=True)
    parser.add_argument("--replay-baseline", required=True)
    parser.add_argument("--repeat-dir", required=True)
    parser.add_argument("--stflash", required=True)
    parser.add_argument("--stflash-freq", default="200")
    parser.add_argument("--make", default="make")
    return parser.parse_args()


def run(command: list[str], env: dict[str, str] | None = None) -> None:
    merged_env = os.environ.copy()
    if env:
        merged_env.update(env)
    result = subprocess.run(command, env=merged_env, check=False)
    if result.returncode != 0:
        raise SystemExit(result.returncode)


def trigger_stlink_reset(stflash: str, stflash_freq: str) -> int:
    result = subprocess.run(
        [stflash, "--connect-under-reset", f"--freq={stflash_freq}", "reset"],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
    )
    if result.stdout:
        print(result.stdout, end="" if result.stdout.endswith("\n") else "\n")
    return result.returncode


def stop_process(proc: subprocess.Popen[str]) -> None:
    proc.terminate()
    try:
        proc.wait(timeout=2)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait(timeout=2)


def run_capture(
    command: list[str],
    *,
    env: dict[str, str],
    reset_mode: str,
    stflash: str,
    stflash_freq: str,
    timeout_s: float,
) -> None:
    proc = subprocess.Popen(
        command,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        env=env,
    )
    assert proc.stdout is not None
    deadline = time.monotonic() + timeout_s
    reset_fired = False

    while proc.poll() is None:
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            stop_process(proc)
            raise SystemExit(124)

        readable, _, _ = select.select([proc.stdout], [], [], min(0.25, remaining))
        if not readable:
            continue

        line = proc.stdout.readline()
        if not line:
            continue
        print(line, end="")
        if reset_mode == "stlink" and not reset_fired and "Listener active;" in line:
            reset_rc = trigger_stlink_reset(stflash, stflash_freq)
            if reset_rc != 0:
                stop_process(proc)
                raise SystemExit(reset_rc)
            reset_fired = True

    for line in proc.stdout:
        print(line, end="")
    if proc.returncode != 0:
        raise SystemExit(proc.returncode)


def main() -> int:
    args = parse_args()
    if args.reset_mode == "manual" and not args.allow_manual_reset:
        print(
            "manual reset is legacy/debug only for fw-gate; "
            "use --allow-manual-reset to opt in explicitly",
            file=sys.stderr,
        )
        return 1

    make = [args.make, "--no-print-directory"]
    run([*make, "check-workspace"])
    run([*make, "test"])
    run([*make, "gate"])
    run([*make, "fw"])
    run([*make, "fw-bin"])
    run([*make, "flash-ur", f"SERIAL={args.serial}", f"STFLASH={args.stflash}"])
    run([*make, "flash-verify-ur", f"STFLASH={args.stflash}"])
    run([*make, "flash-compare-ur", f"STFLASH={args.stflash}"])

    pythonpath = f"{Path.cwd()}{os.pathsep}{os.environ['PYTHONPATH']}" if "PYTHONPATH" in os.environ else str(Path.cwd())
    run_capture(
        [
            sys.executable,
            "scripts/artifact_tool.py",
            "capture",
            "--quick",
            "--reset-context",
            args.reset_mode,
            "--out",
            args.replay_run,
        ],
        env={"PYTHONPATH": pythonpath, "SERIAL": args.serial},
        reset_mode=args.reset_mode,
        stflash=args.stflash,
        stflash_freq=args.stflash_freq,
        timeout_s=float(args.capture_timeout),
    )
    run(
        [
            sys.executable,
            "scripts/artifact_tool.py",
            "verify",
            args.replay_run,
            "--signal-model",
            args.signal_model,
        ],
        env={"PYTHONPATH": pythonpath},
    )
    run(
        [
            sys.executable,
            "scripts/artifact_tool.py",
            "compare",
            args.replay_baseline,
            args.replay_run,
        ],
        env={"PYTHONPATH": pythonpath},
    )

    repeat_dir = Path(args.repeat_dir)
    if str(repeat_dir) in {"", ".", "/"}:
        print(f"refusing unsafe repeat-dir cleanup: {repeat_dir}", file=sys.stderr)
        return 1
    if repeat_dir.exists():
        shutil.rmtree(repeat_dir)
    run(
        [
            sys.executable,
            "scripts/repeat_capture.py",
            "--contract",
            "rpl0",
            "--runs",
            args.repeat_runs,
            "--signal-model",
            args.signal_model,
            "--manifest-name",
            "replay_manifest_v1.txt",
            "--artifacts-dir",
            args.repeat_dir,
            "--reset-mode",
            args.reset_mode,
            "--stflash",
            args.stflash,
        ],
        env={"PYTHONPATH": pythonpath, "SERIAL": args.serial},
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
