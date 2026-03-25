#!/usr/bin/env python3
"""Adversarial structure tests for replay artifact parsing."""

import hashlib
import struct
import sys

import inspect_artifact

FRAME_FMT = "<IBBHIi"
FRAME_SIZE = struct.calcsize(FRAME_FMT)
MAGIC = inspect_artifact.MAGIC
SCHEMA_HASH_LEN = 32
V1_HEADER_MIN = inspect_artifact.V1_MIN_HEADER_SIZE
V1_OFF_VERSION = inspect_artifact.V1_OFF_VERSION
V1_OFF_HEADER_LEN = inspect_artifact.V1_OFF_HEADER_LEN
V1_OFF_FRAME_COUNT = inspect_artifact.V1_OFF_FRAME_COUNT
V1_OFF_FRAME_SIZE = inspect_artifact.V1_OFF_FRAME_SIZE
V1_OFF_FLAGS = inspect_artifact.V1_OFF_FLAGS
V1_OFF_SCHEMA_LEN = inspect_artifact.V1_OFF_SCHEMA_LEN
V1_OFF_SCHEMA_HASH = inspect_artifact.V1_OFF_SCHEMA_HASH
V1_OFF_CAPTURE_BOUNDARY = inspect_artifact.V1_OFF_CAPTURE_BOUNDARY
V1_OFF_RESERVED = inspect_artifact.V1_OFF_RESERVED


def frame_bytes(frame_idx: int = 0) -> bytes:
    return struct.pack(FRAME_FMT, frame_idx, 0x02, 0x00, 0x0000, 1000, frame_idx & 0xFF)


def build_v0(frame_count: int = 1, reserved: int = 0) -> bytes:
    header = struct.pack("<4sIII", b"RPL0", 0, frame_count, reserved)
    frames = b"".join(frame_bytes(i) for i in range(frame_count))
    return header + frames


def build_v1(
    frame_count: int = 1,
    schema: bytes = b"",
    version: int = 1,
    header_len: int = V1_HEADER_MIN,
    frame_size: int = FRAME_SIZE,
    flags: int = 0,
    reserved: int = 0,
) -> bytes:
    schema_hash = hashlib.sha256(schema).digest()
    header = bytearray(header_len)
    header[0:4] = MAGIC
    struct.pack_into("<H", header, V1_OFF_VERSION, version)
    struct.pack_into("<H", header, V1_OFF_HEADER_LEN, header_len)
    struct.pack_into("<I", header, V1_OFF_FRAME_COUNT, frame_count)
    struct.pack_into("<H", header, V1_OFF_FRAME_SIZE, frame_size)
    struct.pack_into("<H", header, V1_OFF_FLAGS, flags)
    struct.pack_into("<I", header, V1_OFF_SCHEMA_LEN, len(schema))
    header[V1_OFF_SCHEMA_HASH : V1_OFF_SCHEMA_HASH + SCHEMA_HASH_LEN] = schema_hash
    struct.pack_into("<H", header, V1_OFF_CAPTURE_BOUNDARY, 0)  # capture_boundary
    struct.pack_into("<H", header, V1_OFF_RESERVED, reserved)
    frames = b"".join(frame_bytes(i) for i in range(frame_count))
    return bytes(header) + schema + frames


def expect_fail(name: str, payload: bytes) -> None:
    try:
        inspect_artifact.parse_artifact_bytes(payload, allow_trailing=False)
    except ValueError:
        print(f"PASS: {name}")
        return
    raise AssertionError(f"{name}: expected parse failure")


def expect_pass(name: str, payload: bytes, allow_trailing: bool = False) -> None:
    try:
        inspect_artifact.parse_artifact_bytes(payload, allow_trailing=allow_trailing)
    except ValueError as exc:
        raise AssertionError(f"{name}: expected success, got {exc}") from exc
    print(f"PASS: {name}")


def parse_or_fail(name: str, payload: bytes, allow_trailing: bool = False) -> dict:
    try:
        return inspect_artifact.parse_artifact_bytes(payload, allow_trailing=allow_trailing)
    except ValueError as exc:
        raise AssertionError(f"{name}: expected success, got {exc}") from exc


def main() -> int:
    # Known-good baselines
    expect_pass("magic_exact_rpl0_accepts", build_v1(frame_count=1, schema=b""))
    expect_pass("valid_v0", build_v0(frame_count=2))
    expect_pass("valid_v1_empty_schema", build_v1(frame_count=2, schema=b""))
    expect_pass("valid_v1_nonempty_schema", build_v1(frame_count=2, schema=b"schema-v1"))

    # Exact schema/frame boundary: frame region must begin immediately after schema.
    artifact = parse_or_fail(
        "exact_schema_frame_boundary_valid",
        build_v1(frame_count=1, schema=b"schema-boundary"),
    )
    if artifact["frames_offset"] != artifact["header_len"] + artifact["schema_len"]:
        raise AssertionError("exact_schema_frame_boundary_valid: frames_offset mismatch")
    print("PASS: exact_schema_frame_boundary_valid")

    # Required adversarial failures
    expect_fail("bad_magic", b"XPL0" + build_v0(frame_count=1)[4:])
    expect_fail("truncated_header", b"RPL0\x00")

    v1 = bytearray(build_v1(frame_count=1, version=2))
    expect_fail("nonzero_u32_invalid_v1_u16_version", bytes(v1))

    v0 = build_v0(frame_count=1, reserved=7)
    expect_fail("v0_nonzero_reserved", v0)

    v1 = build_v1(frame_count=1, flags=1)
    expect_fail("v1_nonzero_flags", v1)

    v1 = build_v1(frame_count=1, reserved=1)
    expect_fail("v1_nonzero_reserved", v1)

    # Ambiguous under naive parsing: v1 signature at 0x04 but structurally invalid v1 header.
    # Dispatch MUST enter v1 path (u32 at 0x04 is non-zero) and reject deterministically.
    v1 = bytearray(build_v1(frame_count=1, version=1))
    struct.pack_into("<H", v1, V1_OFF_HEADER_LEN, 0x0008)
    expect_fail("ambiguous_header_v1_like_but_invalid_header_len", bytes(v1))

    v1 = bytearray(build_v1(frame_count=1))
    struct.pack_into("<H", v1, V1_OFF_HEADER_LEN, 0x0090)  # header_len < V1_MIN_HEADER_SIZE
    expect_fail("header_len_too_small", bytes(v1))

    v1 = bytearray(build_v1(frame_count=1))
    struct.pack_into("<H", v1, V1_OFF_HEADER_LEN, 0x0200)  # header_len > file_len
    expect_fail("oversized_header_len", bytes(v1))

    v1 = bytearray(build_v1(frame_count=1))
    struct.pack_into("<H", v1, V1_OFF_FRAME_SIZE, 8)  # invalid frame_size
    expect_fail("invalid_frame_size", bytes(v1))

    v1 = bytearray(build_v1(frame_count=1))
    v1[V1_OFF_SCHEMA_HASH] ^= 0x01  # corrupt schema_hash
    expect_fail("schema_hash_mismatch", bytes(v1))

    # Mutate a schema byte without updating schema_hash.
    v1 = bytearray(build_v1(frame_count=1, schema=b"abcd"))
    schema_offset = V1_HEADER_MIN
    v1[schema_offset + 2] ^= 0x01
    expect_fail("schema_byte_mutated_without_hash_update", bytes(v1))

    v1 = bytearray(build_v1(frame_count=1))
    struct.pack_into("<I", v1, V1_OFF_SCHEMA_LEN, 0xFFFF_FFF0)  # schema_len overflow/range violation
    expect_fail("corrupted_schema_len", bytes(v1))

    # schema_len = 0 but schema_hash intentionally not SHA256(empty).
    v1 = bytearray(build_v1(frame_count=1, schema=b""))
    v1[V1_OFF_SCHEMA_HASH] ^= 0x80
    expect_fail("empty_schema_with_corrupted_empty_hash", bytes(v1))

    # Enlarge declared schema so it consumes a frame while keeping total length internally consistent.
    # Original: header + 3 schema + 2 frames. Mutated: header + 19 schema + 1 frame.
    # Contract point: redeclaring schema to overlap frame bytes must not yield a valid artifact.
    # Rejection may occur via schema_hash mismatch (expected for this mutation).
    v1 = bytearray(build_v1(frame_count=2, schema=b"abc"))
    struct.pack_into("<I", v1, V1_OFF_SCHEMA_LEN, 19)  # 3 + 16; consumes first frame into schema region
    struct.pack_into("<I", v1, V1_OFF_FRAME_COUNT, 1)   # keep expected total length equal to file length
    expect_fail("schema_len_consumes_frame_region", bytes(v1))

    v0 = build_v0(frame_count=2)
    expect_fail("frame_count_mismatch", v0[:-FRAME_SIZE])  # declared 2, only 1 present

    v1 = build_v1(frame_count=2, schema=b"abc")
    expect_fail("partial_frame_region", v1[:-5])  # trailing partial frame bytes removed

    # Remove exactly one byte at schema/frame boundary.
    v1 = build_v1(frame_count=1, schema=b"abcd")
    boundary = V1_HEADER_MIN + 4
    v1 = v1[:boundary] + v1[boundary + 1 :]
    expect_fail("truncate_one_byte_at_schema_frame_boundary", v1)

    # Truncated schema region (declared schema_len cannot be fully read).
    v1 = build_v1(frame_count=1, schema=b"abcdef")
    expect_fail("truncated_schema_region", v1[: V1_HEADER_MIN + 5])

    v1 = build_v1(frame_count=1, schema=b"")
    expect_fail("trailing_bytes_rejected_strict_mode", v1 + b"\xAA\xBB")

    # Canonical hash parsing should ignore trailing bytes when enabled.
    v1 = build_v1(frame_count=1, schema=b"")
    expect_pass("trailing_bytes_ignored_for_hash_mode", v1 + b"\xAA\xBB", allow_trailing=True)

    print("PASS: adversarial parser suite")
    return 0


if __name__ == "__main__":
    sys.exit(main())
