#!/usr/bin/env python3
"""Deterministic CLI regression tests for `artifact_tool.py hash`."""

import re
import subprocess
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
BASELINE = REPO_ROOT / "artifacts" / "baseline.bin"
SHA_LINE = re.compile(r"^sha256:\s*([0-9a-f]{64})$", re.MULTILINE)
TRAIL_LINE = re.compile(r"^trailing_len:\s*(\d+)$", re.MULTILINE)


def run_cmd(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )


def extract_sha(stdout: str) -> str:
    match = SHA_LINE.search(stdout)
    if match is None:
        raise AssertionError(f"missing sha256 line in output:\n{stdout}")
    return match.group(1)


def extract_trailing_len(stdout: str) -> int:
    match = TRAIL_LINE.search(stdout)
    if match is None:
        raise AssertionError(f"missing trailing_len line in output:\n{stdout}")
    return int(match.group(1))


def main() -> int:
    proc = run_cmd(
        [
            "python3",
            "scripts/artifact_tool.py",
            "hash",
            str(BASELINE),
        ]
    )
    if proc.returncode != 0:
        raise AssertionError(f"hash_valid_artifact: expected success rc=0, got {proc.returncode}")
    base_sha = extract_sha(proc.stdout)
    if extract_trailing_len(proc.stdout) != 0:
        raise AssertionError("hash_valid_artifact: expected trailing_len=0")

    with tempfile.TemporaryDirectory(prefix="dpw_hash_") as tmp:
        tmp_dir = Path(tmp)
        trailing = tmp_dir / "trailing.bin"
        trailing.write_bytes(BASELINE.read_bytes() + b"\x01\x02\x03")
        proc = run_cmd(
            [
                "python3",
                "scripts/artifact_tool.py",
                "hash",
                str(trailing),
            ]
        )
        if proc.returncode != 0:
            raise AssertionError(
                f"hash_trailing_artifact: expected success rc=0, got {proc.returncode}\n{proc.stdout}"
            )
        trailing_sha = extract_sha(proc.stdout)
        if trailing_sha != base_sha:
            raise AssertionError("hash_trailing_artifact: canonical hash changed with trailing bytes")
        if extract_trailing_len(proc.stdout) == 0:
            raise AssertionError("hash_trailing_artifact: expected non-zero trailing_len")

        malformed = tmp_dir / "malformed.bin"
        raw = bytearray(BASELINE.read_bytes())
        raw[0] ^= 0x01
        malformed.write_bytes(bytes(raw))
        proc = run_cmd(
            [
                "python3",
                "scripts/artifact_tool.py",
                "hash",
                str(malformed),
            ]
        )
        if proc.returncode == 0:
            raise AssertionError("hash_malformed_artifact: expected failure")
        if "FAIL: invalid artifact structure" not in proc.stdout:
            raise AssertionError("hash_malformed_artifact: expected invalid-structure failure")

    proc = run_cmd(
        [
            "python3",
            "scripts/artifact_tool.py",
            "hash",
            str(BASELINE),
            "--expect",
            "0" * 64,
        ]
    )
    if proc.returncode == 0:
        raise AssertionError("hash_expect_mismatch: expected failure")
    if "FAIL: hash mismatch" not in proc.stdout:
        raise AssertionError("hash_expect_mismatch: expected hash mismatch failure")

    print("PASS: artifact_tool hash CLI regression suite")
    return 0


if __name__ == "__main__":
    sys.exit(main())
