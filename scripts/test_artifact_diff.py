#!/usr/bin/env python3
"""Deterministic CLI regression tests for `artifact_diff.py`."""

import struct
import subprocess
import sys
import tempfile
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import inspect_artifact


REPO_ROOT = Path(__file__).resolve().parent.parent
FIXTURE_GEN = ["python3", "scripts/generate_demo_v3_fixtures.py"]
DIFF = ["python3", "scripts/artifact_diff.py"]
SAMPLE_OFFSET = 12


def run(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=REPO_ROOT, text=True, capture_output=True, check=False)


def require(proc: subprocess.CompletedProcess[str], rc: int, contains: str, name: str) -> None:
    if proc.returncode != rc:
        raise AssertionError(
            f"{name}: expected rc={rc}, got rc={proc.returncode}\nstdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )
    if contains not in proc.stdout:
        raise AssertionError(
            f"{name}: missing output '{contains}'\nstdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )


def set_sample(path: Path, frame_idx: int, new_sample: int) -> None:
    parsed = inspect_artifact.parse_artifact(path, allow_trailing=False)
    data = bytearray(path.read_bytes())
    frame_base = parsed["frames_offset"] + (frame_idx * parsed["frame_size"])
    sample_offset = frame_base + SAMPLE_OFFSET
    struct.pack_into("<i", data, sample_offset, new_sample)
    path.write_bytes(data)


def main() -> int:
    require(run(FIXTURE_GEN), 0, "PASS: demo-v3 fixtures generated", "fixture_gen")

    require(
        run(DIFF + ["artifacts/demo_v3/transient_A.rpl", "artifacts/demo_v3/transient_B.rpl"]),
        0,
        "Classification: transient",
        "transient",
    )
    require(
        run(DIFF + ["artifacts/demo_v3/offset_A.rpl", "artifacts/demo_v3/offset_B.rpl"]),
        0,
        "Classification: persistent_offset",
        "persistent_offset",
    )
    require(
        run(DIFF + ["artifacts/demo_v3/rate_A.rpl", "artifacts/demo_v3/rate_B.rpl"]),
        0,
        "Classification: rate_divergence",
        "rate_divergence",
    )
    require(
        run(DIFF + ["artifacts/demo_v3/rate_A.rpl", "artifacts/demo_v3/rate_A.rpl"]),
        0,
        "NO DIVERGENCE: artifacts identical",
        "no_divergence",
    )
    with tempfile.TemporaryDirectory(prefix="dpw_artifact_diff_") as tmp:
        tmp_dir = Path(tmp)
        a_path = tmp_dir / "nonclass_a.rpl"
        b_path = tmp_dir / "nonclass_b.rpl"
        baseline = REPO_ROOT / "artifacts" / "demo_v3" / "rate_A.rpl"
        a_path.write_bytes(baseline.read_bytes())
        b_path.write_bytes(baseline.read_bytes())

        # Nonclassifiable by design:
        # - no sustained reconvergence in transient window
        # - non-constant offset
        # - abs(diff) is not nondecreasing
        base = inspect_artifact.parse_artifact(b_path, allow_trailing=False)
        start = 4096
        offsets = [1, 2, 1, 2, 1, 2, 1, 2, 1]
        for rel, offset in enumerate(offsets):
            frame_idx = start + rel
            sample = base["frames"][frame_idx]["input_sample"]
            set_sample(b_path, frame_idx, sample + offset)

        require(
            run(DIFF + [str(a_path), str(b_path)]),
            1,
            "does not satisfy rate-divergence monotonic-growth rule",
            "nonclassifiable_shape",
        )

    print("PASS: artifact_diff CLI regression suite")
    return 0


if __name__ == "__main__":
    sys.exit(main())
