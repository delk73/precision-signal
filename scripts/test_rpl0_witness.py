#!/usr/bin/env python3
"""Tests for scripts/rpl0_witness.py.

Uses synthetic minimal RPL0 v1 fixtures built in-process.
Does not depend on retained release artifacts.
"""

import hashlib
import struct
import sys
import tempfile
from pathlib import Path

SCRIPTS_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPTS_DIR))

import rpl0_witness

# ---------------------------------------------------------------------------
# Fixture builder (independent of inspect_artifact / artifact_tool)
# ---------------------------------------------------------------------------

_MAGIC = b"RPL0"
_V1_MIN_HEADER_LEN = 0x98
_FRAME_FMT = "<IBBHIi"
_FRAME_SIZE = struct.calcsize(_FRAME_FMT)

assert _FRAME_SIZE == 16, f"frame size should be 16, got {_FRAME_SIZE}"


def _frame_bytes(frame_idx: int, input_sample: int = 0) -> bytes:
    return struct.pack(_FRAME_FMT, frame_idx, 0x02, 0x00, 0x0000, 1000, input_sample)


def _build_v1_artifact(
    frame_count: int = 3,
    schema_len: int = 0,
    header_len: int | None = None,
    frame_size: int = 16,
    version: int = 1,
    magic: bytes = _MAGIC,
    samples: list[int] | None = None,
    flags: int = 0,
    reserved: int = 0,
    schema: bytes | None = None,
) -> bytes:
    """Build a minimal valid RPL0 v1 artifact."""
    if header_len is None:
        header_len = _V1_MIN_HEADER_LEN

    if schema is None:
        schema = b"\x00" * schema_len
    else:
        schema_len = len(schema)
    schema_hash = hashlib.sha256(schema).digest()

    header = bytearray(header_len)
    header[0:4] = magic
    struct.pack_into("<H", header, 0x04, version)
    struct.pack_into("<H", header, 0x06, header_len)
    struct.pack_into("<I", header, 0x08, frame_count)
    struct.pack_into("<H", header, 0x0C, frame_size)
    struct.pack_into("<H", header, 0x0E, flags)
    struct.pack_into("<I", header, 0x10, schema_len)
    header[0x14:0x14 + 32] = schema_hash         # schema_hash
    # build_hash, config_hash, board_id, clock_profile remain zero
    struct.pack_into("<H", header, 0x96, reserved)

    if samples is None:
        samples = [i & 0xFF for i in range(frame_count)]

    frames = b"".join(_frame_bytes(i, samples[i]) for i in range(frame_count))
    return bytes(header) + schema + frames


def _result(report: str) -> str:
    for line in report.splitlines():
        if line.startswith("RESULT:"):
            return line.split(":", 1)[1].strip()
    return ""


def _field(report: str, key: str) -> str:
    for line in report.splitlines():
        if line.startswith(f"{key}:"):
            return line.split(":", 1)[1].strip()
    return ""


def _field_names(report: str) -> list[str]:
    return [line.split(":", 1)[0] for line in report.splitlines() if ":" in line]


def _assert_common_shape(report: str, result: str) -> None:
    assert _field(report, "RESULT") == result, f"unexpected result:\n{report}"
    assert _field(report, "ARTIFACT_COUNT") == "1", f"missing artifact count:\n{report}"
    assert _field(report, "WITNESS_DIGEST") != "", f"missing digest field:\n{report}"
    assert _field(report, "FIRST_INVALID_FRAME") != "", f"missing invalid-frame field:\n{report}"


def _assert_core_field_order(report: str, expected: list[str]) -> None:
    names = _field_names(report)
    assert names[:len(expected)] == expected, f"unexpected field order {names}:\n{report}"


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------

def test_valid_minimal_v1_passes() -> None:
    data = _build_v1_artifact(frame_count=3, schema_len=0)
    report = rpl0_witness.witness(data, "synthetic")
    assert _result(report) == "PASS", f"expected PASS:\n{report}"
    assert _field(report, "FRAME_COUNT") == "3"
    assert _field(report, "FRAME_SIZE") == "16"
    assert _field(report, "FIRST_INVALID_FRAME") == "none"


def test_valid_with_schema_passes() -> None:
    data = _build_v1_artifact(frame_count=5, schema_len=32)
    report = rpl0_witness.witness(data, "synthetic")
    assert _result(report) == "PASS", f"expected PASS:\n{report}"
    assert _field(report, "SCHEMA_LEN") == "32"


def test_zero_frames_passes() -> None:
    data = _build_v1_artifact(frame_count=0, schema_len=0, samples=[])
    report = rpl0_witness.witness(data, "synthetic")
    assert _result(report) == "PASS", f"expected PASS:\n{report}"
    assert _field(report, "FRAME_COUNT") == "0"


def test_pass_report_shape_order() -> None:
    data = _build_v1_artifact(frame_count=3, schema_len=0)
    report = rpl0_witness.witness(data, "synthetic")
    _assert_common_shape(report, "PASS")
    _assert_core_field_order(
        report,
        [
            "RESULT",
            "ARTIFACT_COUNT",
            "FRAME_SIZE",
            "FRAME_COUNT",
            "WITNESS_DIGEST",
            "FIRST_INVALID_FRAME",
        ],
    )
    assert _field(report, "FRAME_SIZE") == "16"
    assert _field(report, "FRAME_COUNT") == "3"
    assert _field(report, "FIRST_INVALID_FRAME") == "none"
    assert _field(report, "WITNESS_DIGEST") != "none"


def test_fail_report_shape_order_for_pre_frame_error() -> None:
    report = rpl0_witness.witness(b"", "synthetic")
    _assert_common_shape(report, "FAIL")
    _assert_core_field_order(
        report,
        [
            "RESULT",
            "ARTIFACT_COUNT",
            "FRAME_SIZE",
            "FRAME_COUNT",
            "WITNESS_DIGEST",
            "FIRST_INVALID_FRAME",
            "ERROR",
        ],
    )
    assert _field(report, "FRAME_SIZE") == "none"
    assert _field(report, "FRAME_COUNT") == "none"
    assert _field(report, "WITNESS_DIGEST") == "none"
    assert _field(report, "FIRST_INVALID_FRAME") == "none"


def test_bad_magic_fails() -> None:
    data = _build_v1_artifact(magic=b"XXXX")
    report = rpl0_witness.witness(data, "synthetic")
    assert _result(report) == "FAIL", f"expected FAIL:\n{report}"
    assert "magic" in _field(report, "ERROR").lower()


def test_unsupported_version_fails() -> None:
    data = _build_v1_artifact(version=0)
    report = rpl0_witness.witness(data, "synthetic")
    assert _result(report) == "FAIL", f"expected FAIL:\n{report}"
    assert "version" in _field(report, "ERROR").lower()


def test_version_2_fails() -> None:
    data = _build_v1_artifact(version=2)
    report = rpl0_witness.witness(data, "synthetic")
    assert _result(report) == "FAIL", f"expected FAIL:\n{report}"


def test_bad_frame_size_fails() -> None:
    data = _build_v1_artifact(frame_size=8)
    report = rpl0_witness.witness(data, "synthetic")
    assert _result(report) == "FAIL", f"expected FAIL:\n{report}"
    assert "frame_size" in _field(report, "ERROR").lower()


def test_truncated_header_fails() -> None:
    data = _build_v1_artifact()[:10]
    report = rpl0_witness.witness(data, "synthetic")
    assert _result(report) == "FAIL", f"expected FAIL:\n{report}"
    assert "truncated" in _field(report, "ERROR").lower()


def test_truncated_schema_fails() -> None:
    data = _build_v1_artifact(schema_len=64)
    # cut off mid-schema
    data = data[:_V1_MIN_HEADER_LEN + 10]
    report = rpl0_witness.witness(data, "synthetic")
    assert _result(report) == "FAIL", f"expected FAIL:\n{report}"
    assert "truncated" in _field(report, "ERROR").lower()


def test_truncated_frame_region_fails() -> None:
    data = _build_v1_artifact(frame_count=10, schema_len=0)
    # cut off mid-frame-region
    data = data[: _V1_MIN_HEADER_LEN + 5 * 16 + 8]
    report = rpl0_witness.witness(data, "synthetic")
    assert _result(report) == "FAIL", f"expected FAIL:\n{report}"
    assert "truncated" in _field(report, "ERROR").lower()


def test_non_monotonic_frame_idx_fails() -> None:
    """Manually patch frame 2 to have the wrong frame_idx."""
    data = bytearray(_build_v1_artifact(frame_count=4, schema_len=0))
    # frame 2 is at offset _V1_MIN_HEADER_LEN + 2 * 16
    frame2_off = _V1_MIN_HEADER_LEN + 2 * 16
    # overwrite frame_idx (u32 LE) to 99
    struct.pack_into("<I", data, frame2_off, 99)
    report = rpl0_witness.witness(bytes(data), "synthetic")
    assert _result(report) == "FAIL", f"expected FAIL:\n{report}"
    assert "non-monotonic" in _field(report, "ERROR").lower()
    assert _field(report, "FIRST_INVALID_FRAME") == "2"


def test_trailing_bytes_fail() -> None:
    data = _build_v1_artifact(frame_count=3, schema_len=0) + b"\xAA\xBB"
    report = rpl0_witness.witness(data, "synthetic")
    assert _result(report) == "FAIL", f"expected FAIL:\n{report}"
    assert "size mismatch" in _field(report, "ERROR").lower()
    assert _field(report, "FRAME_SIZE") == "16"
    assert _field(report, "FRAME_COUNT") == "3"
    assert _field(report, "FIRST_INVALID_FRAME") == "none"


def test_nonzero_flags_fail() -> None:
    data = _build_v1_artifact(flags=1)
    report = rpl0_witness.witness(data, "synthetic")
    assert _result(report) == "FAIL", f"expected FAIL:\n{report}"
    assert "flags" in _field(report, "ERROR").lower()
    assert _field(report, "FIRST_INVALID_FRAME") == "none"


def test_nonzero_reserved_fail() -> None:
    data = _build_v1_artifact(reserved=1)
    report = rpl0_witness.witness(data, "synthetic")
    assert _result(report) == "FAIL", f"expected FAIL:\n{report}"
    assert "reserved" in _field(report, "ERROR").lower()
    assert _field(report, "FIRST_INVALID_FRAME") == "none"


def test_mutated_header_schema_hash_fails() -> None:
    data = bytearray(_build_v1_artifact(frame_count=1, schema=b"schema-v1"))
    data[0x14] ^= 0x01
    report = rpl0_witness.witness(bytes(data), "synthetic")
    assert _result(report) == "FAIL", f"expected FAIL:\n{report}"
    assert "schema_hash" in _field(report, "ERROR").lower()
    assert _field(report, "FIRST_INVALID_FRAME") == "none"


def test_mutated_schema_bytes_with_old_header_hash_fails() -> None:
    data = bytearray(_build_v1_artifact(frame_count=1, schema=b"schema-v1"))
    data[_V1_MIN_HEADER_LEN + 2] ^= 0x01
    report = rpl0_witness.witness(bytes(data), "synthetic")
    assert _result(report) == "FAIL", f"expected FAIL:\n{report}"
    assert "schema_hash" in _field(report, "ERROR").lower()
    assert _field(report, "FIRST_INVALID_FRAME") == "none"


def test_declared_frame_count_leaves_extra_bytes_fails() -> None:
    data = bytearray(_build_v1_artifact(frame_count=2, schema_len=0))
    struct.pack_into("<I", data, 0x08, 1)
    report = rpl0_witness.witness(bytes(data), "synthetic")
    assert _result(report) == "FAIL", f"expected FAIL:\n{report}"
    assert "size mismatch" in _field(report, "ERROR").lower()
    assert _field(report, "FRAME_COUNT") == "1"
    assert _field(report, "FIRST_INVALID_FRAME") == "none"


def test_declared_schema_len_consumes_frame_bytes_fails() -> None:
    data = bytearray(_build_v1_artifact(frame_count=2, schema=b"abc"))
    struct.pack_into("<I", data, 0x10, 19)
    struct.pack_into("<I", data, 0x08, 1)
    report = rpl0_witness.witness(bytes(data), "synthetic")
    assert _result(report) == "FAIL", f"expected FAIL:\n{report}"
    assert "schema_hash" in _field(report, "ERROR").lower()
    assert _field(report, "FIRST_INVALID_FRAME") == "none"


def test_empty_file_fails() -> None:
    report = rpl0_witness.witness(b"", "synthetic")
    assert _result(report) == "FAIL", f"expected FAIL:\n{report}"


def test_digest_is_stable() -> None:
    data = _build_v1_artifact(frame_count=10, schema_len=0)
    r1 = rpl0_witness.witness(data, "a")
    r2 = rpl0_witness.witness(data, "a")
    assert _field(r1, "WITNESS_DIGEST") == _field(r2, "WITNESS_DIGEST"), \
        "digest must be deterministic"


def test_digest_changes_when_input_sample_changes() -> None:
    samples_a = [i & 0xFF for i in range(5)]
    samples_b = list(samples_a)
    samples_b[2] = (samples_b[2] + 1) & 0xFF

    data_a = _build_v1_artifact(frame_count=5, schema_len=0, samples=samples_a)
    data_b = _build_v1_artifact(frame_count=5, schema_len=0, samples=samples_b)

    r_a = rpl0_witness.witness(data_a, "a")
    r_b = rpl0_witness.witness(data_b, "b")

    d_a = _field(r_a, "WITNESS_DIGEST")
    d_b = _field(r_b, "WITNESS_DIGEST")
    assert d_a != d_b, f"digest must change when input_sample changes (both: {d_a})"


def test_cli_pass_on_valid_artifact() -> None:
    """Smoke test the CLI entry point via a temp file."""
    import subprocess

    data = _build_v1_artifact(frame_count=3, schema_len=0)
    with tempfile.NamedTemporaryFile(suffix=".rpl0", delete=False) as f:
        f.write(data)
        tmp = Path(f.name)
    try:
        result = subprocess.run(
            [sys.executable, str(SCRIPTS_DIR / "rpl0_witness.py"), str(tmp)],
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0, f"expected exit 0:\n{result.stdout}{result.stderr}"
        assert "RESULT: PASS" in result.stdout
    finally:
        tmp.unlink(missing_ok=True)


def test_cli_fail_on_bad_magic() -> None:
    import subprocess

    data = _build_v1_artifact(magic=b"BAAD")
    with tempfile.NamedTemporaryFile(suffix=".rpl0", delete=False) as f:
        f.write(data)
        tmp = Path(f.name)
    try:
        result = subprocess.run(
            [sys.executable, str(SCRIPTS_DIR / "rpl0_witness.py"), str(tmp)],
            capture_output=True,
            text=True,
        )
        assert result.returncode == 1, f"expected exit 1:\n{result.stdout}{result.stderr}"
        assert "RESULT: FAIL" in result.stdout
    finally:
        tmp.unlink(missing_ok=True)


# ---------------------------------------------------------------------------
# Runner
# ---------------------------------------------------------------------------

_TESTS = [
    test_valid_minimal_v1_passes,
    test_valid_with_schema_passes,
    test_zero_frames_passes,
    test_pass_report_shape_order,
    test_fail_report_shape_order_for_pre_frame_error,
    test_bad_magic_fails,
    test_unsupported_version_fails,
    test_version_2_fails,
    test_bad_frame_size_fails,
    test_truncated_header_fails,
    test_truncated_schema_fails,
    test_truncated_frame_region_fails,
    test_non_monotonic_frame_idx_fails,
    test_trailing_bytes_fail,
    test_nonzero_flags_fail,
    test_nonzero_reserved_fail,
    test_mutated_header_schema_hash_fails,
    test_mutated_schema_bytes_with_old_header_hash_fails,
    test_declared_frame_count_leaves_extra_bytes_fails,
    test_declared_schema_len_consumes_frame_bytes_fails,
    test_empty_file_fails,
    test_digest_is_stable,
    test_digest_changes_when_input_sample_changes,
    test_cli_pass_on_valid_artifact,
    test_cli_fail_on_bad_magic,
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
        print(f"FAIL: rpl0_witness tests ({passed} passed, {failed} failed)", file=sys.stderr)
        return 1

    print(f"PASS: rpl0_witness tests ({passed} passed)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
