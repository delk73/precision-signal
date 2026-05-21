#!/usr/bin/env python3
"""Run the hardware-backed firmware gate."""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
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
    parser.add_argument("--make", default="make")
    return parser.parse_args()


def run(command: list[str], env: dict[str, str] | None = None) -> None:
    merged_env = os.environ.copy()
    if env:
        merged_env.update(env)
    result = subprocess.run(command, env=merged_env, check=False)
    if result.returncode != 0:
        raise SystemExit(result.returncode)


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
    run(
        [
            "timeout",
            args.capture_timeout,
            "env",
            f"SERIAL={args.serial}",
            sys.executable,
            "scripts/artifact_tool.py",
            "capture",
            "--quick",
            "--reset-context",
            args.reset_mode,
            "--out",
            args.replay_run,
        ],
        env={"PYTHONPATH": pythonpath},
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
