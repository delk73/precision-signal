#!/usr/bin/env python3
"""Deterministic CLI regression tests for `artifact_tool.py verify`."""

import struct
import subprocess
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
SCRIPTS_DIR = REPO_ROOT / "scripts"
sys.path.insert(0, str(REPO_ROOT))
sys.path.insert(0, str(SCRIPTS_DIR))

import artifact_tool  # noqa: E402
import inspect_artifact  # noqa: E402

BASELINE = REPO_ROOT / "artifacts" / "baseline.bin"
FRAME_INPUT_SAMPLE_OFFSET = 12


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


def write_model_artifact(source: Path, dest: Path, model: str) -> None:
    artifact = inspect_artifact.parse_artifact(source, allow_trailing=False)
    raw = bytearray(source.read_bytes())
    frames_offset = artifact["frames_offset"]
    frame_size = artifact["frame_size"]
    for frame_idx in range(artifact["frame_count"]):
        sample = artifact_tool.expected_sample_for_model(frame_idx, model)
        sample_offset = frames_offset + frame_idx * frame_size + FRAME_INPUT_SAMPLE_OFFSET
        raw[sample_offset : sample_offset + 4] = struct.pack("<i", sample)
    dest.write_bytes(raw)


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
        burst8 = tmp_dir / "burst8.bin"
        write_model_artifact(BASELINE, burst8, "burst8")
        proc = run_cmd(
            [
                "python3",
                "scripts/artifact_tool.py",
                "verify",
                str(burst8),
                "--signal-model",
                "burst8",
            ]
        )
        assert_ok("verify_burst8_artifact", proc)

        proc = run_cmd(
            [
                "python3",
                "scripts/artifact_tool.py",
                "verify",
                str(burst8),
                "--signal-model",
                "phase8",
            ]
        )
        assert_fail("verify_burst8_as_phase8_fail", proc)
        if "FAIL: sample mismatch" not in proc.stdout:
            raise AssertionError("verify_burst8_as_phase8_fail: expected sample mismatch")

        seeded_lfsr8 = tmp_dir / "seeded_lfsr8.bin"
        write_model_artifact(BASELINE, seeded_lfsr8, "seeded_lfsr8")
        proc = run_cmd(
            [
                "python3",
                "scripts/artifact_tool.py",
                "verify",
                str(seeded_lfsr8),
                "--signal-model",
                "seeded_lfsr8",
            ]
        )
        assert_ok("verify_seeded_lfsr8_artifact", proc)

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
