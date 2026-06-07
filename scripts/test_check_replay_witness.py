#!/usr/bin/env python3
"""Regression tests for scripts/check_replay_witness.py."""

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

SCRIPTS_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPTS_DIR.parent
sys.path.insert(0, str(SCRIPTS_DIR))

import rpl0_witness
from test_rpl0_witness import _build_v1_artifact


VERSION = "9.9.9"


def run_check(release_root: Path, version: str = VERSION) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            sys.executable,
            str(SCRIPTS_DIR / "check_replay_witness.py"),
            "--version",
            version,
            "--release-root",
            str(release_root),
        ],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )


def run_check_without_version() -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(SCRIPTS_DIR / "check_replay_witness.py")],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )


def make_release(root: Path, version: str = VERSION, data: bytes | None = None) -> Path:
    release_dir = root / version
    release_dir.mkdir(parents=True)
    if data is None:
        data = _build_v1_artifact(frame_count=3, schema_len=0)
    artifact_path = release_dir / "fw_capture.bin"
    artifact_path.write_bytes(data)
    report = rpl0_witness.witness(data, str(artifact_path))
    (release_dir / "rpl0_witness_fw_capture.txt").write_text(report, encoding="utf-8")
    return release_dir


def assert_ok(name: str, proc: subprocess.CompletedProcess[str]) -> None:
    if proc.returncode != 0:
        raise AssertionError(
            f"{name}: expected success, rc={proc.returncode}\n"
            f"stdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )
    if "replay witness check: PASS" not in proc.stdout:
        raise AssertionError(f"{name}: missing PASS output\nstdout:\n{proc.stdout}")


def assert_fail(name: str, proc: subprocess.CompletedProcess[str], needle: str) -> None:
    if proc.returncode == 0:
        raise AssertionError(f"{name}: expected failure\nstdout:\n{proc.stdout}")
    output = proc.stdout + proc.stderr
    if needle not in output:
        raise AssertionError(
            f"{name}: missing failure marker {needle!r}\n"
            f"stdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )


def replace_report(release_dir: Path, text: str) -> None:
    (release_dir / "rpl0_witness_fw_capture.txt").write_text(text, encoding="utf-8")


def mutate_artifact(release_dir: Path) -> None:
    path = release_dir / "fw_capture.bin"
    data = bytearray(path.read_bytes())
    data[-1] ^= 0x01
    path.write_bytes(bytes(data))


def test_missing_version_fails() -> None:
    assert_fail("missing_version", run_check_without_version(), "--version")


def test_missing_release_directory_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        assert_fail("missing_release_dir", run_check(Path(tmp)), "missing retained release directory")


def test_missing_fw_capture_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        (release_dir / "fw_capture.bin").unlink()
        assert_fail("missing_fw_capture", run_check(Path(tmp)), "missing retained artifact")


def test_missing_witness_report_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        (release_dir / "rpl0_witness_fw_capture.txt").unlink()
        assert_fail("missing_witness_report", run_check(Path(tmp)), "missing retained witness report")


def test_malformed_retained_report_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, "RESULT PASS\nWITNESS_DIGEST: 0123456789abcdef\n")
        assert_fail("malformed_retained_report", run_check(Path(tmp)), "malformed line")


def test_retained_fail_result_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, "RESULT: FAIL\nWITNESS_DIGEST: 0123456789abcdef\n")
        assert_fail("retained_fail_result", run_check(Path(tmp)), "RESULT is not PASS")


def test_retained_result_missing_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, "WITNESS_DIGEST: 0123456789abcdef\n")
        assert_fail("retained_result_missing", run_check(Path(tmp)), "missing RESULT")


def test_retained_digest_missing_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, "RESULT: PASS\n")
        assert_fail("retained_digest_missing", run_check(Path(tmp)), "missing WITNESS_DIGEST")


def test_duplicate_retained_fields_fail() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        report = (release_dir / "rpl0_witness_fw_capture.txt").read_text(encoding="utf-8")
        replace_report(release_dir, report + "RESULT: PASS\n")
        assert_fail("duplicate_retained_fields", run_check(Path(tmp)), "duplicate RESULT")


def test_malformed_digest_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, "RESULT: PASS\nWITNESS_DIGEST: not-a-digest\n")
        assert_fail("malformed_digest", run_check(Path(tmp)), "malformed WITNESS_DIGEST")


def test_digest_mismatch_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, "RESULT: PASS\nWITNESS_DIGEST: 0000000000000000\n")
        assert_fail("digest_mismatch", run_check(Path(tmp)), "witness digest mismatch")


def test_modified_capture_digest_mismatch_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        mutate_artifact(release_dir)
        assert_fail("modified_capture", run_check(Path(tmp)), "witness digest mismatch")


def test_truncated_capture_recomputed_witness_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        path = release_dir / "fw_capture.bin"
        path.write_bytes(path.read_bytes()[:-1])
        assert_fail("truncated_capture", run_check(Path(tmp)), "RESULT is not PASS")


def test_valid_retained_style_fixture_passes() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        report = (release_dir / "rpl0_witness_fw_capture.txt").read_text(encoding="utf-8")
        lines = report.splitlines()
        reordered: list[str] = []
        for key in ("RESULT:", "WITNESS:", "INPUT:", "FORMAT:", "HEADER_LEN:", "SCHEMA_LEN:"):
            reordered.extend(line for line in lines if line.startswith(key))
        reordered.extend(
            line
            for line in lines
            if line.startswith(("FRAME_COUNT:", "FRAME_SIZE:", "FIRST_INVALID_FRAME:", "WITNESS_DIGEST:", "CLAIM:"))
        )
        replace_report(release_dir, "\n".join(reordered) + "\n")
        assert_ok("valid_retained_style_fixture", run_check(Path(tmp)))


def test_release_root_isolated_from_retained_evidence() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        retained_copy = release_dir / "retained_copy.txt"
        shutil.copyfile(release_dir / "rpl0_witness_fw_capture.txt", retained_copy)
        assert_ok("isolated_release_root", run_check(Path(tmp)))
        if retained_copy.read_text(encoding="utf-8") != (
            release_dir / "rpl0_witness_fw_capture.txt"
        ).read_text(encoding="utf-8"):
            raise AssertionError("checker mutated retained witness fixture")


_TESTS = [
    test_missing_version_fails,
    test_missing_release_directory_fails,
    test_missing_fw_capture_fails,
    test_missing_witness_report_fails,
    test_malformed_retained_report_fails,
    test_retained_fail_result_fails,
    test_retained_result_missing_fails,
    test_retained_digest_missing_fails,
    test_duplicate_retained_fields_fail,
    test_malformed_digest_fails,
    test_digest_mismatch_fails,
    test_modified_capture_digest_mismatch_fails,
    test_truncated_capture_recomputed_witness_fails,
    test_valid_retained_style_fixture_passes,
    test_release_root_isolated_from_retained_evidence,
]


def main() -> int:
    passed = 0
    failed = 0
    for test in _TESTS:
        try:
            test()
            passed += 1
        except AssertionError as exc:
            print(f"FAIL: {test.__name__}: {exc}", file=sys.stderr)
            failed += 1
        except Exception as exc:  # noqa: BLE001
            print(f"ERROR: {test.__name__}: {type(exc).__name__}: {exc}", file=sys.stderr)
            failed += 1

    if failed:
        print(f"FAIL: check_replay_witness tests ({passed} passed, {failed} failed)", file=sys.stderr)
        return 1

    print(f"PASS: check_replay_witness tests ({passed} passed)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
