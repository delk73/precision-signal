#!/usr/bin/env python3
"""Deterministic CLI regression tests for `compare_artifact.py`."""

import random
import subprocess
import sys
import tempfile
from pathlib import Path

import inspect_artifact
from test_artifact_parser_valid_v1 import build_valid_v1_artifact


REPO_ROOT = Path(__file__).resolve().parent.parent
BASELINE = REPO_ROOT / "artifacts" / "baseline.bin"


def run_compare(baseline: Path, candidate: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["python3", "scripts/compare_artifact.py", str(baseline), str(candidate)],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="dpw_compare_") as tmp:
        tmp_dir = Path(tmp)

        identical_a = tmp_dir / "identical_a.bin"
        identical_b = tmp_dir / "identical_b.bin"
        data = BASELINE.read_bytes()
        identical_a.write_bytes(data)
        identical_b.write_bytes(data)
        proc = run_compare(identical_a, identical_b)
        if proc.returncode != 0:
            raise AssertionError(f"identical_artifacts: expected success\n{proc.stdout}\n{proc.stderr}")
        if "PASS: artifacts are identical" not in proc.stdout:
            raise AssertionError("identical_artifacts: missing PASS banner")

        frame_diff = tmp_dir / "frame_diff.bin"
        raw = bytearray(data)
        parsed = inspect_artifact.parse_artifact_bytes(bytes(raw), allow_trailing=False)
        raw[parsed["frames_offset"] + 12] ^= 0x01
        frame_diff.write_bytes(bytes(raw))
        proc = run_compare(identical_a, frame_diff)
        if proc.returncode != 5:
            raise AssertionError(
                f"frame_mismatch: expected rc=5, got {proc.returncode}\n{proc.stdout}\n{proc.stderr}"
            )
        if "FAIL: first divergence at frame" not in proc.stdout:
            raise AssertionError("frame_mismatch: expected frame divergence message")

        rng = random.Random(0)
        header_a = tmp_dir / "header_a.bin"
        header_b = tmp_dir / "header_b.bin"
        header_a.write_bytes(build_valid_v1_artifact(rng, frame_count=2, schema_len=0)[0])
        header_b.write_bytes(build_valid_v1_artifact(rng, frame_count=3, schema_len=0)[0])
        proc = run_compare(header_a, header_b)
        if proc.returncode != 4:
            raise AssertionError(
                f"header_mismatch: expected rc=4, got {proc.returncode}\n{proc.stdout}\n{proc.stderr}"
            )
        if "FAIL: header mismatch" not in proc.stdout:
            raise AssertionError("header_mismatch: expected header mismatch message")

        malformed_candidate = tmp_dir / "malformed_candidate.bin"
        malformed_candidate.write_bytes(data[:-1])
        proc = run_compare(identical_a, malformed_candidate)
        if proc.returncode != 3:
            raise AssertionError(
                "malformed_candidate: expected rc=3 "
                f"got {proc.returncode}\n{proc.stdout}\n{proc.stderr}"
            )
        if "FAIL: candidate parse failed" not in proc.stdout:
            raise AssertionError("malformed_candidate: expected candidate parse failure")

        malformed_baseline = tmp_dir / "malformed_baseline.bin"
        malformed_baseline.write_bytes(data[:-1])
        proc = run_compare(malformed_baseline, identical_a)
        if proc.returncode != 3:
            raise AssertionError(
                "malformed_baseline: expected rc=3 "
                f"got {proc.returncode}\n{proc.stdout}\n{proc.stderr}"
            )
        if "FAIL: baseline parse failed" not in proc.stdout:
            raise AssertionError("malformed_baseline: expected baseline parse failure")

    print("PASS: compare_artifact CLI regression suite")
    return 0


if __name__ == "__main__":
    sys.exit(main())
