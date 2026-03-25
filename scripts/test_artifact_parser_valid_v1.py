#!/usr/bin/env python3
"""Deterministic acceptance-space tests for valid replay artifact v1 parsing."""

import hashlib
import random
import struct
import sys

import inspect_artifact

FRAME_FMT = "<IBBHIi"
FRAME_SIZE = struct.calcsize(FRAME_FMT)
MAGIC = inspect_artifact.MAGIC
V1_HEADER_MIN = inspect_artifact.V1_MIN_HEADER_SIZE
V1_OFF_VERSION = inspect_artifact.V1_OFF_VERSION
V1_OFF_HEADER_LEN = inspect_artifact.V1_OFF_HEADER_LEN
V1_OFF_FRAME_COUNT = inspect_artifact.V1_OFF_FRAME_COUNT
V1_OFF_FRAME_SIZE = inspect_artifact.V1_OFF_FRAME_SIZE
V1_OFF_FLAGS = inspect_artifact.V1_OFF_FLAGS
V1_OFF_SCHEMA_LEN = inspect_artifact.V1_OFF_SCHEMA_LEN
V1_OFF_SCHEMA_HASH = inspect_artifact.V1_OFF_SCHEMA_HASH
V1_OFF_BUILD_HASH = inspect_artifact.V1_OFF_BUILD_HASH
V1_OFF_CONFIG_HASH = inspect_artifact.V1_OFF_CONFIG_HASH
V1_OFF_BOARD_ID = inspect_artifact.V1_OFF_BOARD_ID
V1_OFF_CLOCK_PROFILE = inspect_artifact.V1_OFF_CLOCK_PROFILE
V1_OFF_CAPTURE_BOUNDARY = inspect_artifact.V1_OFF_CAPTURE_BOUNDARY
V1_OFF_RESERVED = inspect_artifact.V1_OFF_RESERVED

SCHEMA_HASH_LEN = 32
BUILD_HASH_LEN = 32
CONFIG_HASH_LEN = 32
BOARD_ID_LEN = 16
CLOCK_PROFILE_LEN = 16
CORPUS_SIZE = 200


def frame_bytes(frame_idx: int = 0) -> bytes:
    return struct.pack(FRAME_FMT, frame_idx, 0x02, 0x00, 0x0000, 1000, frame_idx & 0xFF)


def random_bytes(rng: random.Random, length: int) -> bytes:
    return bytes(rng.getrandbits(8) for _ in range(length))


def build_valid_v1_artifact(
    rng: random.Random,
    frame_count: int | None = None,
    schema_len: int | None = None,
    header_ext: bytes = b"",
) -> tuple[bytes, int, bytes]:
    if frame_count is None:
        frame_count = rng.randint(1, 32)
    if schema_len is None:
        schema_len = rng.randint(0, 64)
    schema = random_bytes(rng, schema_len)
    schema_hash = hashlib.sha256(schema).digest()
    board_id = random_bytes(rng, BOARD_ID_LEN)
    clock_profile = random_bytes(rng, CLOCK_PROFILE_LEN)
    build_hash = random_bytes(rng, BUILD_HASH_LEN)
    config_hash = random_bytes(rng, CONFIG_HASH_LEN)
    capture_boundary = rng.getrandbits(16)

    header_len = V1_HEADER_MIN + len(header_ext)
    header = bytearray(header_len)
    header[0:4] = MAGIC
    struct.pack_into("<H", header, V1_OFF_VERSION, 1)
    struct.pack_into("<H", header, V1_OFF_HEADER_LEN, header_len)
    struct.pack_into("<I", header, V1_OFF_FRAME_COUNT, frame_count)
    struct.pack_into("<H", header, V1_OFF_FRAME_SIZE, FRAME_SIZE)
    struct.pack_into("<H", header, V1_OFF_FLAGS, 0)
    struct.pack_into("<I", header, V1_OFF_SCHEMA_LEN, schema_len)
    header[V1_OFF_SCHEMA_HASH : V1_OFF_SCHEMA_HASH + SCHEMA_HASH_LEN] = schema_hash
    header[V1_OFF_BUILD_HASH : V1_OFF_BUILD_HASH + BUILD_HASH_LEN] = build_hash
    header[V1_OFF_CONFIG_HASH : V1_OFF_CONFIG_HASH + CONFIG_HASH_LEN] = config_hash
    header[V1_OFF_BOARD_ID : V1_OFF_BOARD_ID + BOARD_ID_LEN] = board_id
    header[V1_OFF_CLOCK_PROFILE : V1_OFF_CLOCK_PROFILE + CLOCK_PROFILE_LEN] = clock_profile
    struct.pack_into("<H", header, V1_OFF_CAPTURE_BOUNDARY, capture_boundary)
    struct.pack_into("<H", header, V1_OFF_RESERVED, 0)
    if header_ext:
        header[V1_HEADER_MIN:header_len] = header_ext

    frames = b"".join(frame_bytes(i) for i in range(frame_count))
    artifact = bytes(header) + schema + frames
    return artifact, frame_count, schema


def assert_parse_invariants(
    idx_label: str,
    artifact: bytes,
    expected_frame_count: int,
    schema: bytes,
    expected_header_len: int = V1_HEADER_MIN,
) -> None:
    parsed = inspect_artifact.parse_artifact_bytes(artifact, allow_trailing=False)

    expected_schema_len = len(schema)
    expected_frames_offset = expected_header_len + expected_schema_len
    expected_canonical_len = (
        expected_header_len + expected_schema_len + expected_frame_count * FRAME_SIZE
    )

    if parsed["version"] != 1:
        raise AssertionError(f"{idx_label}: version dispatch failed")
    if parsed["header_len"] != expected_header_len:
        raise AssertionError(f"{idx_label}: header_len invariant failed")
    if parsed["schema_len"] != expected_schema_len:
        raise AssertionError(f"{idx_label}: schema boundary failed")
    if parsed["frame_count"] != expected_frame_count:
        raise AssertionError(f"{idx_label}: frame count integrity failed")
    if parsed["frames_offset"] != expected_frames_offset:
        raise AssertionError(f"{idx_label}: layout invariant failed")
    if parsed["canonical_len"] != expected_canonical_len:
        raise AssertionError(f"{idx_label}: canonical length correctness failed")


def main() -> int:
    rng = random.Random(0)

    explicit_cases = 0
    artifact, expected_frame_count, schema = build_valid_v1_artifact(
        rng,
        frame_count=0,
        schema_len=0,
    )
    assert_parse_invariants(
        "valid_v1_zero_frames_empty_schema",
        artifact,
        expected_frame_count,
        schema,
    )
    explicit_cases += 1

    artifact, expected_frame_count, schema = build_valid_v1_artifact(
        rng,
        frame_count=0,
        schema_len=7,
    )
    assert_parse_invariants(
        "valid_v1_zero_frames_nonempty_schema",
        artifact,
        expected_frame_count,
        schema,
    )
    explicit_cases += 1

    header_ext = random_bytes(rng, 16)
    artifact, expected_frame_count, schema = build_valid_v1_artifact(
        rng,
        frame_count=3,
        schema_len=5,
        header_ext=header_ext,
    )
    assert_parse_invariants(
        "valid_v1_extended_header",
        artifact,
        expected_frame_count,
        schema,
        expected_header_len=V1_HEADER_MIN + len(header_ext),
    )
    explicit_cases += 1

    stratified_pairs = [(1, 0), (1, 64), (32, 0), (32, 64)]
    for idx, (forced_frame_count, forced_schema_len) in enumerate(stratified_pairs):
        artifact, expected_frame_count, schema = build_valid_v1_artifact(
            rng,
            frame_count=forced_frame_count,
            schema_len=forced_schema_len,
        )
        assert_parse_invariants(
            f"stratified[{idx}]",
            artifact,
            expected_frame_count,
            schema,
        )

    for idx in range(CORPUS_SIZE):
        artifact, expected_frame_count, schema = build_valid_v1_artifact(rng)
        assert_parse_invariants(
            f"random[{idx}]",
            artifact,
            expected_frame_count,
            schema,
        )

    total = explicit_cases + len(stratified_pairs) + CORPUS_SIZE
    print(
        "PASS: v1 valid artifact corpus "
        f"(explicit={explicit_cases}, stratified={len(stratified_pairs)}, "
        f"random={CORPUS_SIZE}, total={total})"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
