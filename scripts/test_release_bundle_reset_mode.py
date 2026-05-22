#!/usr/bin/env python3
"""Regression tests for retained firmware reset-mode policy."""

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
SOURCE_BUNDLE = REPO_ROOT / "docs" / "verification" / "releases" / "1.8.0"


def run_check(bundle_dir: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            sys.executable,
            "scripts/check_release_bundle.py",
            "--bundle-dir",
            str(bundle_dir),
        ],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )


def copy_bundle(root: Path, version: str) -> Path:
    dst = root / version
    shutil.copytree(SOURCE_BUNDLE, dst)
    return dst


def set_reset_mode(bundle_dir: Path, mode: str) -> None:
    for rel_path in (
        "fw_repeat/replay_manifest_v1.txt",
        "firmware_release_evidence.md",
    ):
        path = bundle_dir / rel_path
        text = path.read_text(encoding="utf-8")
        text = text.replace("reset_mode=manual", f"reset_mode={mode}")
        text = text.replace("reset_mode=stlink", f"reset_mode={mode}")
        path.write_text(text, encoding="utf-8")


def assert_ok(name: str, proc: subprocess.CompletedProcess[str]) -> None:
    if proc.returncode != 0:
        raise AssertionError(
            f"{name}: expected success, rc={proc.returncode}\n"
            f"stdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )


def assert_fail(name: str, proc: subprocess.CompletedProcess[str], needle: str) -> None:
    if proc.returncode == 0:
        raise AssertionError(f"{name}: expected failure")
    if needle not in proc.stdout:
        raise AssertionError(
            f"{name}: missing failure marker {needle!r}\nstdout:\n{proc.stdout}"
        )


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="precision_release_reset_") as tmp:
        root = Path(tmp)

        legacy = copy_bundle(root, "1.8.0")
        assert_ok("legacy_manual_reset_passes", run_check(legacy))

        future_manual = copy_bundle(root, "1.8.1")
        assert_fail(
            "future_manual_reset_fails",
            run_check(future_manual),
            "reset_mode must be stlink for release 1.8.1",
        )

        future_stlink = copy_bundle(root, "1.8.2")
        set_reset_mode(future_stlink, "stlink")
        assert_ok("future_stlink_reset_passes", run_check(future_stlink))

    print("PASS: release bundle reset-mode regression suite")
    return 0


if __name__ == "__main__":
    sys.exit(main())
