#!/usr/bin/env python3
"""Regression tests for release-bundle inventory validation."""

from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent

NON_FIRMWARE_FILES = (
    "index.md",
    "cargo_check_dpw4_thumb_locked.txt",
    "kani_evidence.txt",
    "make_demo_evidence_package.txt",
    "make_doc_link_check.txt",
    "make_gate.txt",
    "make_release_bundle_check.txt",
    "make_replay_tests.txt",
    "release_reproducibility.txt",
)


def run_check(bundle_dir: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            "python3",
            "scripts/check_release_bundle.py",
            "--bundle-dir",
            str(bundle_dir),
        ],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )


def write_bundle_file(bundle_dir: Path, name: str) -> None:
    path = bundle_dir / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(f"{name}\n", encoding="utf-8")


def write_non_firmware_bundle(bundle_dir: Path) -> None:
    for name in NON_FIRMWARE_FILES:
        write_bundle_file(bundle_dir, name)


def assert_ok(name: str, proc: subprocess.CompletedProcess[str]) -> None:
    if proc.returncode != 0:
        raise AssertionError(
            f"{name}: expected success, rc={proc.returncode}\nstdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )


def assert_fail(name: str, proc: subprocess.CompletedProcess[str], needle: str) -> None:
    if proc.returncode == 0:
        raise AssertionError(f"{name}: expected failure")
    if needle not in proc.stdout:
        raise AssertionError(f"{name}: missing failure marker {needle!r}\nstdout:\n{proc.stdout}")


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="dpw_release_bundle_") as tmp:
        bundle_dir = Path(tmp) / "index_only"
        write_non_firmware_bundle(bundle_dir)
        proc = run_check(bundle_dir)
        assert_ok("index_only_non_firmware_bundle_passes", proc)

    with tempfile.TemporaryDirectory(prefix="dpw_release_bundle_") as tmp:
        bundle_dir = Path(tmp) / "duplicate_landing"
        write_non_firmware_bundle(bundle_dir)
        write_bundle_file(bundle_dir, "README.md")
        proc = run_check(bundle_dir)
        assert_fail(
            "duplicate_landing_files_fail",
            proc,
            "cannot contain both README.md and index.md",
        )

    print("PASS: release-bundle inventory regression suite")
    return 0


if __name__ == "__main__":
    sys.exit(main())
