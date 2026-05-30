#!/usr/bin/env python3
"""Run the no-button dual-board HIL observer capture sequence."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Callable

from hil_timing_capture import (
    ALLOWED_CONTEXT_FILES,
    GENERATED_ARTIFACT_FILES,
    PROFILE_DEFINITIONS,
    parse_report,
    validate_dual_board_run_context,
    validate_output_directory,
)


REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_PROFILE = "dual_edge_timing_observer_v1"
DEFAULT_RUN_ROOT = Path("artifacts/hil_timing_dual")
SCRATCH_ROOT = Path("/tmp").resolve()
SERIAL_BY_ID_ROOT = Path("/dev/serial/by-id")
DEVICE_SETTLE_SECONDS = 2.0
DEVICE_READY_TIMEOUT_SECONDS = 15.0
FLASH_RETRY_COUNT = 1
RETRYABLE_STLINK_TRANSPORT_PATTERNS = (
    "LIBUSB_ERROR_NO_DEVICE",
    "LIBUSB_ERROR_TIMEOUT",
    "GETLASTRWSTATUS2 read reply failed",
    "READMEM_32BIT read reply failed",
    "device reports readiness to read but returned no data",
    "write_buffer_to_sram() == -1",
    "stlink_flash_loader_run",
    "stlink_fwrite_flash() == -1",
)


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run the dual-board HIL observer capture sequence."
    )
    identity = parser.add_mutually_exclusive_group(required=True)
    identity.add_argument("--run-id", help="retained run id under artifacts/hil_timing_dual")
    identity.add_argument("--context", help="explicit run_context.json to use")
    parser.add_argument(
        "--out",
        help=(
            "output directory; defaults to artifacts/hil_timing_dual/<run-id> "
            "when --run-id is used"
        ),
    )
    parser.add_argument("--scratch", action="store_true")
    parser.add_argument("--overwrite-generated", action="store_true")
    parser.add_argument("--baud", type=int, default=115200)
    parser.add_argument("--timeout", type=float, default=20.0)
    parser.add_argument("--make", default="make")
    args = parser.parse_args(argv)

    if args.context and not args.out:
        parser.error("--context requires explicit --out")
    if args.scratch and not args.out:
        parser.error("--scratch requires explicit --out")
    if args.overwrite_generated and not args.scratch:
        parser.error("--overwrite-generated requires --scratch")
    if args.context and not args.scratch:
        parser.error("--context requires --scratch")
    return args


def run_dir_for_id(run_id: str) -> Path:
    return DEFAULT_RUN_ROOT / run_id


def resolve_context_and_out(args: argparse.Namespace) -> tuple[Path, Path]:
    if args.run_id:
        retained_out = run_dir_for_id(args.run_id)
        context_path = retained_out / "run_context.json"
        out_dir = Path(args.out) if args.out else retained_out
    else:
        context_path = Path(args.context)
        out_dir = Path(args.out)
    return context_path, out_dir


def load_context(context_path: Path) -> dict[str, object]:
    try:
        return json.loads(context_path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise ValueError(f"missing run context: {context_path}") from exc
    except json.JSONDecodeError as exc:
        raise ValueError(f"malformed run context: {context_path}: {exc}") from exc


def alias(context: dict[str, object], name: str) -> dict[str, str]:
    aliases = context.get("board_aliases")
    if not isinstance(aliases, dict):
        raise ValueError("dual-board run context missing board_aliases")
    entry = aliases.get(name)
    if not isinstance(entry, dict):
        raise ValueError(f"dual-board run context missing board_aliases.{name}")
    return {key: value for key, value in entry.items() if isinstance(value, str)}


def is_relative_to(path: Path, root: Path) -> bool:
    try:
        path.relative_to(root)
    except ValueError:
        return False
    return True


def validate_scratch_out(out_dir: Path) -> None:
    resolved = out_dir.resolve()
    if not is_relative_to(resolved, SCRATCH_ROOT):
        raise ValueError(f"--scratch output must be under /tmp: {out_dir}")


def validate_scratch_existing(out_dir: Path) -> None:
    if not out_dir.exists():
        return
    existing = {path.name for path in out_dir.iterdir()}
    unexpected = sorted(existing - GENERATED_ARTIFACT_FILES - ALLOWED_CONTEXT_FILES)
    if unexpected:
        raise ValueError(
            "scratch output directory has unexpected existing files: "
            + ", ".join(unexpected)
            + f" in {out_dir}"
        )


def preflight_context(context_path: Path, observer_vcp: str) -> None:
    with tempfile.TemporaryDirectory(prefix="dpw_hil_dual_preflight_") as tmp:
        preflight_dir = Path(tmp)
        shutil.copyfile(context_path, preflight_dir / "run_context.json")
        validate_dual_board_run_context(preflight_dir, observer_vcp, DEFAULT_PROFILE)


def prepare_output_context(context_path: Path, out_dir: Path) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    target = out_dir / "run_context.json"
    if context_path.resolve() != target.resolve():
        shutil.copyfile(context_path, target)


def make_flash_command(make_cmd: str) -> list[str]:
    return [make_cmd, "flash-ur"]


def make_flash_env(alias_entry: dict[str, str], firmware_features: str) -> dict[str, str]:
    env = os.environ.copy()
    env["STFLASH_SERIAL"] = alias_entry["stlink_serial"]
    env["FW_FEATURES"] = firmware_features
    return env


def observe_flash_identity(alias_entry: dict[str, str]) -> str | None:
    serial = alias_entry.get("stlink_serial", "")
    if not serial or not SERIAL_BY_ID_ROOT.is_dir():
        return None
    for path in sorted(SERIAL_BY_ID_ROOT.iterdir()):
        if serial in path.name:
            return str(path)
    return None


def observe_vcp_identity(vcp_path: str) -> str | None:
    if vcp_path and Path(vcp_path).exists():
        return vcp_path
    return None


def wait_for_stable_identity(
    observe: Callable[[], str | None],
    failure_message: str,
    timeout_seconds: float = DEVICE_READY_TIMEOUT_SECONDS,
    settle_seconds: float = DEVICE_SETTLE_SECONDS,
) -> str:
    deadline = time.monotonic() + timeout_seconds
    poll_seconds = min(0.25, settle_seconds) if settle_seconds > 0 else 0
    while time.monotonic() <= deadline:
        first = observe()
        if first is not None:
            if time.monotonic() + settle_seconds > deadline:
                break
            if settle_seconds > 0:
                time.sleep(settle_seconds)
            second = observe()
            if second == first and second is not None:
                return second
        if poll_seconds > 0:
            time.sleep(min(poll_seconds, max(0.0, deadline - time.monotonic())))
    raise RuntimeError(failure_message)


def require_flash_identity(role: str, phase: str, alias_entry: dict[str, str]) -> str:
    return wait_for_stable_identity(
        lambda: observe_flash_identity(alias_entry),
        f"{role} flash identity not ready for {phase}",
    )


def require_observer_vcp_for_capture(observer_vcp: str) -> str:
    return wait_for_stable_identity(
        lambda: observe_vcp_identity(observer_vcp),
        "observer VCP not ready for capture start",
    )


def terminate_capture(proc: subprocess.Popen[str]) -> None:
    if proc.poll() is not None:
        return
    proc.terminate()
    try:
        proc.wait(timeout=2)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait(timeout=2)


def retryable_flash_output(output: str) -> bool:
    return any(pattern in output for pattern in RETRYABLE_STLINK_TRANSPORT_PATTERNS)


def run_flash_attempt(
    make_cmd: str,
    alias_entry: dict[str, str],
    firmware_features: str,
) -> tuple[int, str]:
    proc = subprocess.Popen(
        make_flash_command(make_cmd),
        cwd=REPO_ROOT,
        env=make_flash_env(alias_entry, firmware_features),
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )
    output: list[str] = []
    if proc.stdout is not None:
        for line in proc.stdout:
            print(line, end="", flush=True)
            output.append(line)
    return proc.wait(), "".join(output)


def run_flash(
    make_cmd: str,
    phase: str,
    role: str,
    alias_entry: dict[str, str],
    firmware_features: str,
) -> None:
    attempts = FLASH_RETRY_COUNT + 1
    recovery_attempted = False
    for attempt in range(1, attempts + 1):
        print(f"flashing {phase} with make flash-ur", flush=True)
        returncode, output = run_flash_attempt(make_cmd, alias_entry, firmware_features)
        if returncode == 0:
            return
        retryable = retryable_flash_output(output)
        if attempt < attempts and retryable:
            recovery_attempted = True
            print(
                f"WARN: {phase} flash failed with retryable ST-LINK transport "
                f"error; waiting for {role} flash identity recovery",
                flush=True,
            )
            require_flash_identity(role, phase, alias_entry)
            print(
                f"WARN: retrying {phase} flash attempt {attempt + 1}/{attempts}",
                flush=True,
            )
            continue
        if recovery_attempted or retryable:
            raise RuntimeError(
                f"{phase} flash failed after retryable ST-LINK transport recovery attempt"
            )
        raise RuntimeError(
            f"{phase} flash failed with non-retryable exit code {returncode}"
        )


def capture_command(
    out_dir: Path, observer_vcp: str, baud: int, timeout: float, overwrite: bool
) -> list[str]:
    command = [
        sys.executable,
        "scripts/hil_timing_capture.py",
        "--profile",
        DEFAULT_PROFILE,
        "--serial",
        observer_vcp,
        "--out",
        str(out_dir),
        "--baud",
        str(baud),
        "--timeout",
        str(timeout),
    ]
    if overwrite:
        command.append("--overwrite")
    return command


def start_capture(
    out_dir: Path, observer_vcp: str, baud: int, timeout: float, overwrite: bool
) -> subprocess.Popen[str]:
    print(f"starting capture listener on {observer_vcp}", flush=True)
    return subprocess.Popen(
        capture_command(out_dir, observer_vcp, baud, timeout, overwrite),
        cwd=REPO_ROOT,
        text=True,
    )


def timing_summary(out_dir: Path) -> str:
    report = (out_dir / "timing_report.txt").read_text(encoding="utf-8")
    fields = parse_report(report, PROFILE_DEFINITIONS[DEFAULT_PROFILE])
    return (
        "timing_report summary: "
        f"result={fields['result']} "
        f"evidence_window_result={fields['evidence_window_result']} "
        f"trigger_count={fields['trigger_count']} "
        f"ack_count={fields['ack_count']} "
        f"unexpected_ack_count={fields['unexpected_ack_count']} "
        f"pre_first_trigger_ack_count={fields['pre_first_trigger_ack_count']} "
        f"in_window_unexpected_ack_count={fields['in_window_unexpected_ack_count']} "
        f"first_in_window_unexpected_ack_trigger_count={fields['first_in_window_unexpected_ack_trigger_count']} "
        f"last_in_window_unexpected_ack_trigger_count={fields['last_in_window_unexpected_ack_trigger_count']} "
        f"post_final_trigger_ack_count={fields['post_final_trigger_ack_count']} "
        f"evidence_window_trigger_count={fields['evidence_window_trigger_count']} "
        f"evidence_window_ack_count={fields['evidence_window_ack_count']} "
        f"evidence_window_unexpected_ack_count={fields['evidence_window_unexpected_ack_count']} "
        f"evidence_window_max_delta_ticks={fields['evidence_window_max_delta_ticks']} "
        f"evidence_window_max_delta_ns={fields['evidence_window_max_delta_ns']} "
        f"threshold_ticks={fields['threshold_ticks']} "
        f"out={out_dir}"
    )


def run(args: argparse.Namespace) -> int:
    context_path, out_dir = resolve_context_and_out(args)
    context = load_context(context_path)
    actor = alias(context, "actor")
    observer = alias(context, "observer")
    observer_vcp = observer.get("vcp_by_id", "")

    preflight_context(context_path, observer_vcp)
    if args.scratch:
        validate_scratch_out(out_dir)
        validate_scratch_existing(out_dir)
    validate_output_directory(out_dir, args.overwrite_generated)
    prepare_output_context(context_path, out_dir)

    capture_proc: subprocess.Popen[str] | None = None
    try:
        require_flash_identity("actor", "actor quiesce", actor)
        run_flash(args.make, "actor quiesce", "actor", actor, "")
        require_flash_identity("actor", "actor quiesce", actor)

        require_flash_identity("observer", "observer", observer)
        run_flash(args.make, "observer", "observer", observer, observer["firmware_features"])
        require_flash_identity("observer", "observer", observer)
        require_observer_vcp_for_capture(observer_vcp)

        capture_proc = start_capture(
            out_dir,
            observer_vcp,
            args.baud,
            args.timeout,
            args.overwrite_generated,
        )
        try:
            require_flash_identity("actor", "actor active", actor)
            run_flash(args.make, "actor active", "actor", actor, actor["firmware_features"])
        except RuntimeError:
            terminate_capture(capture_proc)
            raise

        capture_rc = capture_proc.wait()
        if capture_rc != 0:
            raise RuntimeError(f"capture failed with exit code {capture_rc}")

        print(timing_summary(out_dir), flush=True)
        return 0
    finally:
        if capture_proc is not None:
            terminate_capture(capture_proc)


def main(argv: list[str] | None = None) -> int:
    try:
        return run(parse_args(argv))
    except (OSError, RuntimeError, ValueError) as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
