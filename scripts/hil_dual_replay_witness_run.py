#!/usr/bin/env python3
"""Run the dual-board replay witness sequence."""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from hil_dual_observer_run import (
    REPO_ROOT,
    alias,
    load_context,
    require_flash_identity,
    require_observer_vcp_for_capture,
    run_flash,
    terminate_capture,
    validate_scratch_out,
)
from hil_replay_witness_capture import (
    ALLOWED_CONTEXT_FILES,
    GENERATED_ARTIFACT_FILES,
    parse_report,
    validate_output_directory,
)


DEFAULT_RUN_ROOT = Path("artifacts/hil_replay_witness_dual")
ACTOR_ACTIVE_FEATURES = "sync_trigger_out sync_trigger_in sync_timing_capture"
WITNESS_FEATURES = "replay_witness_observer"
EXPECTED_ACTOR_ROLE = "external_actor"
EXPECTED_WITNESS_ROLE = "external_observer"


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run the dual-board replay witness sequence."
    )
    identity = parser.add_mutually_exclusive_group(required=True)
    identity.add_argument("--run-id", help="run id under artifacts/hil_replay_witness_dual")
    identity.add_argument("--context", help="explicit run_context.json to use")
    parser.add_argument("--out")
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
        default_out = run_dir_for_id(args.run_id)
        context_path = default_out / "run_context.json"
        out_dir = Path(args.out) if args.out else default_out
    else:
        context_path = Path(args.context)
        out_dir = Path(args.out)
    return context_path, out_dir


def validate_witness_context(context_path: Path, witness_vcp: str) -> None:
    with tempfile.TemporaryDirectory(prefix="dpw_hil_witness_preflight_") as tmp:
        preflight_dir = Path(tmp)
        shutil.copyfile(context_path, preflight_dir / "run_context.json")
        context = load_context(preflight_dir / "run_context.json")
        aliases = context.get("board_aliases")
        if not isinstance(aliases, dict):
            raise ValueError("dual-board witness context missing board_aliases")
        for name, role in (("actor", EXPECTED_ACTOR_ROLE), ("observer", EXPECTED_WITNESS_ROLE)):
            entry = aliases.get(name)
            if not isinstance(entry, dict):
                raise ValueError(f"dual-board witness context missing board_aliases.{name}")
            missing = [
                field
                for field in ("stlink_serial", "vcp_by_id", "firmware_features", "role")
                if not isinstance(entry.get(field), str) or not entry[field]
            ]
            if missing:
                raise ValueError(
                    f"dual-board witness context missing {name} fields: {', '.join(missing)}"
                )
            if entry["role"] != role:
                raise ValueError(
                    f"dual-board witness context invalid {name}.role: "
                    f"expected {role!r}, got {entry['role']!r}"
                )
        if aliases["observer"]["vcp_by_id"] != witness_vcp:
            raise ValueError("witness capture serial must match board_aliases.observer.vcp_by_id")
        confirmation = context.get("board_alias_confirmation")
        if not isinstance(confirmation, dict) or not confirmation.get(
            "confirmed_from_dev_serial_by_id"
        ):
            raise ValueError(
                "dual-board aliases must be confirmed from /dev/serial/by-id/ before capture"
            )


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


def prepare_output_context(context_path: Path, out_dir: Path) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    target = out_dir / "run_context.json"
    if context_path.resolve() != target.resolve():
        shutil.copyfile(context_path, target)


def capture_command(
    out_dir: Path, witness_vcp: str, baud: int, timeout: float, overwrite: bool
) -> list[str]:
    command = [
        sys.executable,
        "scripts/hil_replay_witness_capture.py",
        "--serial",
        witness_vcp,
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
    out_dir: Path, witness_vcp: str, baud: int, timeout: float, overwrite: bool
) -> subprocess.Popen[str]:
    print(f"starting replay witness listener on {witness_vcp}", flush=True)
    return subprocess.Popen(
        capture_command(out_dir, witness_vcp, baud, timeout, overwrite),
        cwd=REPO_ROOT,
        text=True,
    )


def witness_summary(out_dir: Path) -> str:
    report = (out_dir / "witness_report.txt").read_text(encoding="utf-8")
    fields = parse_report(report)
    return (
        "witness_report summary: "
        f"RESULT={fields['RESULT']} "
        f"RULE_ID={fields['RULE_ID']} "
        f"ACTOR_EVENT_COUNT={fields['ACTOR_EVENT_COUNT']} "
        f"FIRST_INVALID_EVENT={fields['FIRST_INVALID_EVENT']} "
        f"WITNESS_DIGEST={fields['WITNESS_DIGEST']} "
        f"out={out_dir}"
    )


def validate_generated_sets() -> None:
    if GENERATED_ARTIFACT_FILES != {"witness_report.txt", "meta.json", "wiring.txt"}:
        raise RuntimeError("unexpected replay witness generated artifact set")
    if "run_context.json" not in ALLOWED_CONTEXT_FILES:
        raise RuntimeError("run_context.json must be allowed in witness output")


def run(args: argparse.Namespace) -> int:
    validate_generated_sets()
    context_path, out_dir = resolve_context_and_out(args)
    context = load_context(context_path)
    actor = alias(context, "actor")
    witness = alias(context, "observer")
    witness_vcp = witness.get("vcp_by_id", "")

    validate_witness_context(context_path, witness_vcp)
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

        require_flash_identity("witness", "witness", witness)
        run_flash(args.make, "witness", "witness", witness, WITNESS_FEATURES)
        require_flash_identity("witness", "witness", witness)
        require_observer_vcp_for_capture(witness_vcp)

        capture_proc = start_capture(
            out_dir,
            witness_vcp,
            args.baud,
            args.timeout,
            args.overwrite_generated,
        )
        try:
            require_flash_identity("actor", "actor active", actor)
            run_flash(args.make, "actor active", "actor", actor, ACTOR_ACTIVE_FEATURES)
        except RuntimeError:
            terminate_capture(capture_proc)
            raise

        capture_rc = capture_proc.wait()
        if capture_rc != 0:
            raise RuntimeError(f"witness capture failed with exit code {capture_rc}")

        fields = parse_report((out_dir / "witness_report.txt").read_text(encoding="utf-8"))
        if fields["RESULT"] != "PASS":
            raise RuntimeError(f"witness RESULT was {fields['RESULT']}")
        print(witness_summary(out_dir), flush=True)
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
