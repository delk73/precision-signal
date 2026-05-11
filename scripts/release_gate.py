#!/usr/bin/env python3
"""Run retained release verification steps for compatibility release targets."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--version", required=True)
    parser.add_argument("--release-root", default="docs/verification/releases")
    parser.add_argument("--serial", default="")
    parser.add_argument("--reset-mode", default="manual")
    parser.add_argument("--fw-target", default="thumbv7em-none-eabihf")
    parser.add_argument("--cargo", default="cargo")
    parser.add_argument("--dpw4-pkg", default="dpw4")
    parser.add_argument("--make", default="make")
    parser.add_argument("--require-serial", action="store_true")
    parser.add_argument("--require-manual-reset", action="store_true")
    parser.add_argument("--thumb-check", action="store_true")
    parser.add_argument("--functional", action="store_true")
    parser.add_argument("--demo-evidence", action="store_true")
    parser.add_argument("--replay-tests", action="store_true")
    parser.add_argument("--doc-link", action="store_true")
    parser.add_argument("--repro", action="store_true")
    parser.add_argument("--firmware", action="store_true")
    parser.add_argument("--bundle-check", action="store_true")
    return parser.parse_args()


def run_transcript(label: str, command: list[str], output: Path, env: dict[str, str] | None = None) -> None:
    print(label)
    output.parent.mkdir(parents=True, exist_ok=True)
    merged_env = os.environ.copy()
    if env:
        merged_env.update(env)
    with output.open("wb") as handle:
        result = subprocess.run(
            command,
            stdout=handle,
            stderr=subprocess.STDOUT,
            env=merged_env,
            check=False,
        )
    if result.returncode != 0:
        print(f"[release-gate] FAIL file={output}", file=sys.stderr)
        raise SystemExit(result.returncode)


def run(label: str, command: list[str]) -> None:
    print(label)
    result = subprocess.run(command, check=False)
    if result.returncode != 0:
        raise SystemExit(result.returncode)


def generate_summary(version: str, release_dir: Path) -> None:
    run(
        "--- [SUMMARY] Release Bundle Summary ---",
        [
            sys.executable,
            "scripts/release_summary.py",
            "--version",
            version,
            "--bundle-dir",
            str(release_dir),
        ],
    )


def main() -> int:
    args = parse_args()
    if args.require_serial and not args.serial:
        print(f"[release-{args.version}] SERIAL is required for firmware-including release", file=sys.stderr)
        return 1
    if args.require_manual_reset and args.reset_mode != "manual":
        print(f"[release-{args.version}] FW_GATE_RESET_MODE must be manual", file=sys.stderr)
        return 1

    release_dir = Path(args.release_root) / args.version
    kani_evidence = release_dir / "kani_evidence.txt"
    if not kani_evidence.is_file() or kani_evidence.stat().st_size == 0:
        print(
            f"[release-{args.version}] FAIL missing or empty {kani_evidence}",
            file=sys.stderr,
        )
        return 1

    make = [args.make, "--no-print-directory"]
    steps: list[tuple[str, list[str], Path, dict[str, str] | None]] = []
    if args.thumb_check:
        steps.append(
            (
                "Thumb Locked Check",
                [args.cargo, "check", "--locked", "-p", args.dpw4_pkg, "--target", args.fw_target],
                release_dir / "cargo_check_dpw4_thumb_locked.txt",
                None,
            )
        )
    if args.functional:
        steps.append(("Functional Validation", [*make, "gate"], release_dir / "make_gate.txt", None))
    if args.demo_evidence:
        steps.append(
            ("Evidence Packaging", [*make, "demo-evidence-package"], release_dir / "make_demo_evidence_package.txt", None)
        )
    if args.replay_tests:
        steps.append(("Replay Tests", [*make, "replay-tests"], release_dir / "make_replay_tests.txt", None))
    if args.doc_link:
        steps.append(
            ("Documentation Integrity", [*make, "doc-link-check"], release_dir / "make_doc_link_check.txt", None)
        )
    if args.repro:
        steps.append(
            (
                "Reproducibility Record",
                ["bash", "scripts/verify_release_repro.sh"],
                release_dir / "release_reproducibility.txt",
                {"RELEASE_EVIDENCE_DIR": str(release_dir)},
            )
        )

    total = len(steps) + (1 if args.bundle_check else 0)
    for index, (name, command, output, env) in enumerate(steps, start=1):
        run_transcript(f"--- [GATE {index}/{total}] {name} ---", command, output, env)

    if args.firmware:
        run("--- [FIRMWARE] RPL0 Capture Gate ---", [*make, "fw-gate", f"SERIAL={args.serial}", "FW_GATE_RESET_MODE=manual"])
        run(
            "--- [FIRMWARE] Archive Evidence ---",
            [*make, "fw-release-archive-current", f"VERSION={args.version}", f"SERIAL={args.serial}"],
        )

    if args.bundle_check:
        bundle_label = "--- [GATE {}/{}] Final Bundle Coherence Check ---".format(total, total)
        if not args.firmware:
            bundle_label = "--- [AUDIT] Bundle Coherence Check ---"
        bundle_check_tmp = release_dir / "make_release_bundle_check.next"
        run_transcript(
            bundle_label,
            [*make, "release-bundle-check", f"VERSION={args.version}"],
            bundle_check_tmp,
        )
        bundle_check_tmp.replace(release_dir / "make_release_bundle_check.txt")
        generate_summary(args.version, release_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
