#!/usr/bin/env python3
"""Regression tests for scripts/check_replay_witness.py."""

from __future__ import annotations

import os
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
OTHER_VERSION = "1.9.1"
REPORT_NAME = "rpl0_witness_fw_capture.txt"
ARTIFACT_NAME = "fw_capture.bin"


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


def run_make_replay_witness_check_without_version() -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env.pop("VERSION", None)
    return subprocess.run(
        ["make", "replay-witness-check"],
        cwd=REPO_ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def make_release(root: Path, version: str = VERSION, data: bytes | None = None) -> Path:
    release_dir = root / version
    release_dir.mkdir(parents=True)
    if data is None:
        data = _build_v1_artifact(frame_count=3, schema_len=0)
    artifact_path = release_dir / ARTIFACT_NAME
    artifact_path.write_bytes(data)
    report = rpl0_witness.witness(data, str(artifact_path))
    (release_dir / REPORT_NAME).write_text(report, encoding="utf-8")
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


def report_path(release_dir: Path) -> Path:
    return release_dir / REPORT_NAME


def artifact_path(release_dir: Path) -> Path:
    return release_dir / ARTIFACT_NAME


def report_text(release_dir: Path) -> str:
    return report_path(release_dir).read_text(encoding="utf-8")


def replace_report(release_dir: Path, text: str) -> None:
    report_path(release_dir).write_text(text, encoding="utf-8")


def replace_report_bytes(release_dir: Path, data: bytes) -> None:
    report_path(release_dir).write_bytes(data)


def field_value(report: str, field: str) -> str:
    prefix = f"{field}:"
    for line in report.splitlines():
        if line.startswith(prefix):
            return line.split(":", 1)[1].strip()
    raise AssertionError(f"missing {field} in report:\n{report}")


def valid_digest(release_dir: Path) -> str:
    return field_value(report_text(release_dir), "WITNESS_DIGEST")


def report_with_result(release_dir: Path, result: str) -> str:
    return report_text(release_dir).replace("RESULT: PASS", f"RESULT: {result}", 1)


def mutate_artifact(release_dir: Path) -> None:
    path = artifact_path(release_dir)
    data = bytearray(path.read_bytes())
    data[-1] ^= 0x01
    path.write_bytes(bytes(data))


def release_tree(release_dir: Path) -> list[tuple[str, str]]:
    entries: list[tuple[str, str]] = []
    for path in sorted(release_dir.rglob("*")):
        kind = "dir" if path.is_dir() else "file"
        entries.append((path.relative_to(release_dir).as_posix(), kind))
    return entries


def release_file_bytes(release_dir: Path) -> dict[str, bytes]:
    return {
        path.relative_to(release_dir).as_posix(): path.read_bytes()
        for path in sorted(release_dir.rglob("*"))
        if path.is_file()
    }


def assert_release_unchanged(
    name: str,
    release_dir: Path,
    before_tree: list[tuple[str, str]],
    before_bytes: dict[str, bytes],
) -> None:
    after_tree = release_tree(release_dir)
    after_bytes = release_file_bytes(release_dir)
    if after_tree != before_tree:
        raise AssertionError(f"{name}: release directory listing changed: {before_tree!r} -> {after_tree!r}")
    if after_bytes != before_bytes:
        raise AssertionError(f"{name}: retained release file bytes changed")


def with_mutated_byte(data: bytes, offset: int) -> bytes:
    mutated = bytearray(data)
    mutated[offset] ^= 0x01
    return bytes(mutated)


def test_missing_version_fails() -> None:
    assert_fail("missing_version", run_check_without_version(), "--version")


def test_make_replay_witness_check_without_version_fails() -> None:
    proc = run_make_replay_witness_check_without_version()
    assert_fail("make_missing_version", proc, "VERSION is required")
    assert_fail("make_missing_version_usage", proc, "make replay-witness-check VERSION=<version>")


def test_missing_release_directory_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        assert_fail("missing_release_dir", run_check(Path(tmp)), "missing retained release directory")


def test_missing_fw_capture_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        artifact_path(release_dir).unlink()
        assert_fail("missing_fw_capture", run_check(Path(tmp)), "missing retained artifact")


def test_missing_witness_report_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        report_path(release_dir).unlink()
        assert_fail("missing_witness_report", run_check(Path(tmp)), "missing retained witness report")


def test_requested_version_does_not_fallback_to_existing_version() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        root = Path(tmp)
        make_release(root, version=OTHER_VERSION)
        (root / VERSION).mkdir()
        assert_fail("wrong_version", run_check(root), "missing retained artifact")


def test_requested_version_does_not_search_sibling_release() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        root = Path(tmp)
        make_release(root, version="sibling")
        assert_fail("sibling_ignored", run_check(root), "missing retained release directory")


def test_requested_version_does_not_search_parent_release_directory() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        root = Path(tmp)
        release_dir = make_release(root)
        for path in (artifact_path(release_dir), report_path(release_dir)):
            path.replace(root / path.name)
        release_dir.rmdir()
        assert_fail("parent_ignored", run_check(root), "missing retained release directory")


def test_explicit_release_root_does_not_fallback_to_other_root() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        root = Path(tmp)
        root_a = root / "tmp-a"
        root_b = root / "tmp-b"
        make_release(root_a)
        (root_b / VERSION).mkdir(parents=True)
        assert_fail("explicit_root_missing", run_check(root_b), "missing retained artifact")


def test_malformed_retained_report_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, "RESULT PASS\nWITNESS_DIGEST: 0123456789abcdef\n")
        assert_fail("malformed_retained_report", run_check(Path(tmp)), "malformed line")


def test_malformed_retained_report_empty_field_name_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, ": PASS\nWITNESS_DIGEST: 0123456789abcdef\n")
        assert_fail("empty_field_name", run_check(Path(tmp)), "empty field name")


def test_malformed_unknown_line_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, "RESULT: PASS\nUNKNOWN MALFORMED\nWITNESS_DIGEST: 0123456789abcdef\n")
        assert_fail("unknown_malformed_line", run_check(Path(tmp)), "malformed line")


def test_retained_report_invalid_utf8_fails_cleanly() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report_bytes(release_dir, b"RESULT: PASS\nWITNESS_DIGEST: 0123456789abcdef\n\xff")
        assert_fail("invalid_utf8", run_check(Path(tmp)), "cannot decode retained witness report")


def test_retained_fail_result_with_matching_digest_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, report_with_result(release_dir, "FAIL"))
        assert_fail("retained_fail_result", run_check(Path(tmp)), "RESULT is not PASS")


def test_retained_lowercase_result_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, f"RESULT: pass\nWITNESS_DIGEST: {valid_digest(release_dir)}\n")
        assert_fail("lowercase_result", run_check(Path(tmp)), "RESULT is not PASS")


def test_retained_lowercase_result_field_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, f"result: PASS\nWITNESS_DIGEST: {valid_digest(release_dir)}\n")
        assert_fail("lowercase_result_field", run_check(Path(tmp)), "missing RESULT")


def test_retained_lowercase_digest_field_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, f"RESULT: PASS\nwitness_digest: {valid_digest(release_dir)}\n")
        assert_fail("lowercase_digest_field", run_check(Path(tmp)), "missing WITNESS_DIGEST")


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


def test_retained_blank_result_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, f"RESULT:   \nWITNESS_DIGEST: {valid_digest(release_dir)}\n")
        assert_fail("blank_result", run_check(Path(tmp)), "empty RESULT")


def test_retained_blank_digest_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, "RESULT: PASS\nWITNESS_DIGEST:   \n")
        assert_fail("blank_digest", run_check(Path(tmp)), "empty WITNESS_DIGEST")


def test_duplicate_retained_result_identical_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, report_text(release_dir) + "RESULT: PASS\n")
        assert_fail("duplicate_result_identical", run_check(Path(tmp)), "duplicate RESULT")


def test_duplicate_retained_result_conflicting_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, report_text(release_dir) + "RESULT: FAIL\n")
        assert_fail("duplicate_result_conflicting", run_check(Path(tmp)), "duplicate RESULT")


def test_duplicate_retained_digest_identical_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, report_text(release_dir) + f"WITNESS_DIGEST: {valid_digest(release_dir)}\n")
        assert_fail("duplicate_digest_identical", run_check(Path(tmp)), "duplicate WITNESS_DIGEST")


def test_duplicate_retained_digest_conflicting_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, report_text(release_dir) + "WITNESS_DIGEST: 0000000000000000\n")
        assert_fail("duplicate_digest_conflicting", run_check(Path(tmp)), "duplicate WITNESS_DIGEST")


def test_malformed_digest_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, "RESULT: PASS\nWITNESS_DIGEST: not-a-digest\n")
        assert_fail("malformed_digest", run_check(Path(tmp)), "malformed WITNESS_DIGEST")


def test_uppercase_digest_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, f"RESULT: PASS\nWITNESS_DIGEST: {valid_digest(release_dir).upper()}\n")
        assert_fail("uppercase_digest", run_check(Path(tmp)), "malformed WITNESS_DIGEST")


def test_whitespace_around_required_fields_is_accepted() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, f"  RESULT  :  PASS  \n  WITNESS_DIGEST  :  {valid_digest(release_dir)}  \n")
        assert_ok("required_field_whitespace", run_check(Path(tmp)))


def test_digest_mismatch_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, "RESULT: PASS\nWITNESS_DIGEST: 0000000000000000\n")
        assert_fail("digest_mismatch", run_check(Path(tmp)), "witness digest mismatch")


def test_digest_from_different_artifact_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        root = Path(tmp)
        release_dir = make_release(root)
        other_dir = make_release(root, version="other", data=_build_v1_artifact(frame_count=4, schema_len=0))
        replace_report(release_dir, f"RESULT: PASS\nWITNESS_DIGEST: {valid_digest(other_dir)}\n")
        assert_fail("different_artifact_digest", run_check(root), "witness digest mismatch")


def test_digest_from_sibling_version_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        root = Path(tmp)
        release_dir = make_release(root)
        sibling_dir = make_release(root, version="sibling", data=_build_v1_artifact(frame_count=5, schema_len=0))
        replace_report(release_dir, f"RESULT: PASS\nWITNESS_DIGEST: {valid_digest(sibling_dir)}\n")
        assert_fail("sibling_version_digest", run_check(root), "witness digest mismatch")


def test_stale_digest_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, "RESULT: PASS\nWITNESS_DIGEST: 1111111111111111\n")
        assert_fail("stale_digest", run_check(Path(tmp)), "witness digest mismatch")


def test_modified_capture_digest_mismatch_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        mutate_artifact(release_dir)
        assert_fail("modified_capture", run_check(Path(tmp)), "witness digest mismatch")


def test_empty_capture_recomputed_witness_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        artifact_path(release_dir).write_bytes(b"")
        assert_fail("empty_capture", run_check(Path(tmp)), "RESULT is not PASS")


def test_short_header_recomputed_witness_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        artifact_path(release_dir).write_bytes(_build_v1_artifact()[:10])
        assert_fail("short_header", run_check(Path(tmp)), "RESULT is not PASS")


def test_truncated_schema_recomputed_witness_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        artifact_path(release_dir).write_bytes(_build_v1_artifact(schema_len=64)[:162])
        assert_fail("truncated_schema", run_check(Path(tmp)), "RESULT is not PASS")


def test_truncated_frame_region_recomputed_witness_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        artifact_path(release_dir).write_bytes(_build_v1_artifact(frame_count=10)[:240])
        assert_fail("truncated_frame_region", run_check(Path(tmp)), "RESULT is not PASS")


def test_invalid_header_length_recomputed_witness_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        artifact_path(release_dir).write_bytes(b"RPL0\x01\x00\x04\x00")
        assert_fail("invalid_header_length", run_check(Path(tmp)), "RESULT is not PASS")


def test_schema_hash_mismatch_recomputed_witness_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        artifact_path(release_dir).write_bytes(with_mutated_byte(_build_v1_artifact(schema=b"schema"), 0x14))
        assert_fail("schema_hash_mismatch", run_check(Path(tmp)), "RESULT is not PASS")


def test_frame_count_payload_mismatch_recomputed_witness_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        artifact_path(release_dir).write_bytes(_build_v1_artifact(frame_count=3) + b"extra")
        assert_fail("frame_count_payload_mismatch", run_check(Path(tmp)), "RESULT is not PASS")


def test_unsupported_version_recomputed_witness_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        artifact_path(release_dir).write_bytes(_build_v1_artifact(version=2))
        assert_fail("unsupported_version", run_check(Path(tmp)), "RESULT is not PASS")


def test_bad_magic_recomputed_witness_fails() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        artifact_path(release_dir).write_bytes(_build_v1_artifact(magic=b"XXXX"))
        assert_fail("bad_magic", run_check(Path(tmp)), "RESULT is not PASS")


def test_valid_retained_style_fixture_passes() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        report = report_text(release_dir)
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


def test_release_directory_is_not_mutated_on_success() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        before_tree = release_tree(release_dir)
        before_bytes = release_file_bytes(release_dir)
        assert_ok("no_mutation_success", run_check(Path(tmp)))
        assert_release_unchanged("no_mutation_success", release_dir, before_tree, before_bytes)


def test_release_directory_is_not_mutated_on_failure() -> None:
    with tempfile.TemporaryDirectory(prefix="precision_replay_witness_") as tmp:
        release_dir = make_release(Path(tmp))
        replace_report(release_dir, "RESULT: PASS\nWITNESS_DIGEST: 0000000000000000\n")
        before_tree = release_tree(release_dir)
        before_bytes = release_file_bytes(release_dir)
        assert_fail("no_mutation_failure", run_check(Path(tmp)), "witness digest mismatch")
        assert_release_unchanged("no_mutation_failure", release_dir, before_tree, before_bytes)


_TESTS = [
    test_missing_version_fails,
    test_make_replay_witness_check_without_version_fails,
    test_missing_release_directory_fails,
    test_missing_fw_capture_fails,
    test_missing_witness_report_fails,
    test_requested_version_does_not_fallback_to_existing_version,
    test_requested_version_does_not_search_sibling_release,
    test_requested_version_does_not_search_parent_release_directory,
    test_explicit_release_root_does_not_fallback_to_other_root,
    test_malformed_retained_report_fails,
    test_malformed_retained_report_empty_field_name_fails,
    test_malformed_unknown_line_fails,
    test_retained_report_invalid_utf8_fails_cleanly,
    test_retained_fail_result_with_matching_digest_fails,
    test_retained_lowercase_result_fails,
    test_retained_lowercase_result_field_fails,
    test_retained_lowercase_digest_field_fails,
    test_retained_result_missing_fails,
    test_retained_digest_missing_fails,
    test_retained_blank_result_fails,
    test_retained_blank_digest_fails,
    test_duplicate_retained_result_identical_fails,
    test_duplicate_retained_result_conflicting_fails,
    test_duplicate_retained_digest_identical_fails,
    test_duplicate_retained_digest_conflicting_fails,
    test_malformed_digest_fails,
    test_uppercase_digest_fails,
    test_whitespace_around_required_fields_is_accepted,
    test_digest_mismatch_fails,
    test_digest_from_different_artifact_fails,
    test_digest_from_sibling_version_fails,
    test_stale_digest_fails,
    test_modified_capture_digest_mismatch_fails,
    test_empty_capture_recomputed_witness_fails,
    test_short_header_recomputed_witness_fails,
    test_truncated_schema_recomputed_witness_fails,
    test_truncated_frame_region_recomputed_witness_fails,
    test_invalid_header_length_recomputed_witness_fails,
    test_schema_hash_mismatch_recomputed_witness_fails,
    test_frame_count_payload_mismatch_recomputed_witness_fails,
    test_unsupported_version_recomputed_witness_fails,
    test_bad_magic_recomputed_witness_fails,
    test_valid_retained_style_fixture_passes,
    test_release_directory_is_not_mutated_on_success,
    test_release_directory_is_not_mutated_on_failure,
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
