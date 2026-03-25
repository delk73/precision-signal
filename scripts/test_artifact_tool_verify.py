#!/usr/bin/env python3
"""Deterministic CLI regression tests for `artifact_tool.py verify`."""

import subprocess
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
BASELINE = REPO_ROOT / "artifacts" / "baseline.bin"


def run_cmd(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )


def assert_ok(name: str, proc: subprocess.CompletedProcess[str]) -> None:
    if proc.returncode != 0:
        raise AssertionError(
            f"{name}: expected success, rc={proc.returncode}\nstdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )


def assert_fail(name: str, proc: subprocess.CompletedProcess[str]) -> None:
    if proc.returncode == 0:
        raise AssertionError(
            f"{name}: expected failure\nstdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )


def main() -> int:
    proc = run_cmd(
        [
            "python3",
            "scripts/artifact_tool.py",
            "verify",
            str(BASELINE),
            "--signal-model",
            "phase8",
        ]
    )
    assert_ok("verify_valid_artifact", proc)
    if "PASS: artifact structure is valid" not in proc.stdout:
        raise AssertionError("verify_valid_artifact: missing PASS banner")

    proc = run_cmd(
        [
            "python3",
            "scripts/artifact_tool.py",
            "verify",
            str(BASELINE),
        ]
    )
    assert_ok("verify_valid_artifact_default_mode", proc)

    with tempfile.TemporaryDirectory(prefix="dpw_verify_") as tmp:
        tmp_dir = Path(tmp)
        trailing = tmp_dir / "trailing.bin"
        trailing.write_bytes(BASELINE.read_bytes() + b"\xAA\xBB")
        proc = run_cmd(
            [
                "python3",
                "scripts/artifact_tool.py",
                "verify",
                str(trailing),
            ]
        )
        assert_fail("verify_trailing_bytes_fail", proc)
        if "FAIL: invalid artifact structure" not in proc.stdout:
            raise AssertionError("verify_trailing_bytes_fail: expected invalid-structure failure")

        malformed = tmp_dir / "malformed.bin"
        malformed.write_bytes(BASELINE.read_bytes()[:-1])
        proc = run_cmd(
            [
                "python3",
                "scripts/artifact_tool.py",
                "verify",
                str(malformed),
            ]
        )
        assert_fail("verify_malformed_fail", proc)
        if "FAIL: invalid artifact structure" not in proc.stdout:
            raise AssertionError("verify_malformed_fail: expected invalid-structure failure")

    print("PASS: artifact_tool verify CLI regression suite")
    return 0


if __name__ == "__main__":
    sys.exit(main())
