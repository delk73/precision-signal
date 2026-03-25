#!/usr/bin/env python3
"""Deterministic near-valid mutation corpus for replay artifact v1 parsing."""

import random
import struct
import sys
from collections.abc import Callable
from dataclasses import dataclass

import inspect_artifact
from test_artifact_parser_valid_v1 import build_valid_v1_artifact

FRAME_SIZE = inspect_artifact.FRAME_SIZE
V1_HEADER_MIN = inspect_artifact.V1_MIN_HEADER_SIZE
V1_OFF_VERSION = inspect_artifact.V1_OFF_VERSION
V1_OFF_HEADER_LEN = inspect_artifact.V1_OFF_HEADER_LEN
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

BASE_CORPUS_SIZE = 8


@dataclass(frozen=True)
class MutationCase:
    name: str
    fn: Callable[[bytes], bytes]
    expect_success: bool


def _mutated(data: bytes) -> bytearray:
    return bytearray(data)


def mutate_flip_magic_byte(data: bytes) -> bytes:
    out = _mutated(data)
    out[0] ^= 0x01
    return bytes(out)


def mutate_set_version_2(data: bytes) -> bytes:
    out = _mutated(data)
    struct.pack_into("<H", out, V1_OFF_VERSION, 2)
    return bytes(out)


def mutate_set_header_len_below_min(data: bytes) -> bytes:
    out = _mutated(data)
    struct.pack_into("<H", out, V1_OFF_HEADER_LEN, V1_HEADER_MIN - 1)
    return bytes(out)


def mutate_set_header_len_above_file(data: bytes) -> bytes:
    out = _mutated(data)
    struct.pack_into("<H", out, V1_OFF_HEADER_LEN, len(data) + 1)
    return bytes(out)


def mutate_set_invalid_frame_size(data: bytes) -> bytes:
    out = _mutated(data)
    struct.pack_into("<H", out, V1_OFF_FRAME_SIZE, FRAME_SIZE - 1)
    return bytes(out)


def mutate_set_flags_nonzero(data: bytes) -> bytes:
    out = _mutated(data)
    struct.pack_into("<H", out, V1_OFF_FLAGS, 1)
    return bytes(out)


def mutate_set_reserved_nonzero(data: bytes) -> bytes:
    out = _mutated(data)
    struct.pack_into("<H", out, V1_OFF_RESERVED, 1)
    return bytes(out)


def mutate_corrupt_schema_hash(data: bytes) -> bytes:
    out = _mutated(data)
    out[V1_OFF_SCHEMA_HASH] ^= 0x01
    return bytes(out)


def mutate_increase_schema_len_without_extending_file(data: bytes) -> bytes:
    out = _mutated(data)
    schema_len = int.from_bytes(
        out[V1_OFF_SCHEMA_LEN : V1_OFF_SCHEMA_LEN + 4], "little", signed=False
    )
    struct.pack_into("<I", out, V1_OFF_SCHEMA_LEN, schema_len + 1)
    return bytes(out)


def mutate_truncate_artifact_one_byte(data: bytes) -> bytes:
    return data[:-1]


def mutate_append_trailing_bytes(data: bytes) -> bytes:
    return data + b"\xAA"


def mutate_build_hash_bytes(data: bytes) -> bytes:
    out = _mutated(data)
    out[V1_OFF_BUILD_HASH] ^= 0x80
    return bytes(out)


def mutate_config_hash_bytes(data: bytes) -> bytes:
    out = _mutated(data)
    out[V1_OFF_CONFIG_HASH + 1] ^= 0x40
    return bytes(out)


def mutate_board_id_bytes(data: bytes) -> bytes:
    out = _mutated(data)
    out[V1_OFF_BOARD_ID + 2] ^= 0x20
    return bytes(out)


def mutate_clock_profile_bytes(data: bytes) -> bytes:
    out = _mutated(data)
    out[V1_OFF_CLOCK_PROFILE + 3] ^= 0x10
    return bytes(out)


def mutate_capture_boundary(data: bytes) -> bytes:
    out = _mutated(data)
    boundary = int.from_bytes(
        out[V1_OFF_CAPTURE_BOUNDARY : V1_OFF_CAPTURE_BOUNDARY + 2], "little", signed=False
    )
    struct.pack_into("<H", out, V1_OFF_CAPTURE_BOUNDARY, (boundary + 1) & 0xFFFF)
    return bytes(out)


REJECT_MUTATIONS: list[MutationCase] = [
    MutationCase("flip_magic_byte", mutate_flip_magic_byte, False),
    MutationCase("set_version_2", mutate_set_version_2, False),
    MutationCase("set_header_len_below_min", mutate_set_header_len_below_min, False),
    MutationCase("set_header_len_above_file", mutate_set_header_len_above_file, False),
    MutationCase("set_invalid_frame_size", mutate_set_invalid_frame_size, False),
    MutationCase("set_flags_nonzero", mutate_set_flags_nonzero, False),
    MutationCase("set_reserved_nonzero", mutate_set_reserved_nonzero, False),
    MutationCase("corrupt_schema_hash", mutate_corrupt_schema_hash, False),
    MutationCase(
        "increase_schema_len_without_extending_file",
        mutate_increase_schema_len_without_extending_file,
        False,
    ),
    MutationCase("truncate_artifact_one_byte", mutate_truncate_artifact_one_byte, False),
    MutationCase("append_trailing_bytes_strict_mode", mutate_append_trailing_bytes, False),
]

ACCEPT_MUTATIONS: list[MutationCase] = [
    MutationCase("modify_build_hash_bytes", mutate_build_hash_bytes, True),
    MutationCase("modify_config_hash_bytes", mutate_config_hash_bytes, True),
    MutationCase("modify_board_id_bytes", mutate_board_id_bytes, True),
    MutationCase("modify_clock_profile_bytes", mutate_clock_profile_bytes, True),
    MutationCase("modify_capture_boundary", mutate_capture_boundary, True),
]


def parse_expect(name: str, payload: bytes, expect_success: bool) -> None:
    try:
        inspect_artifact.parse_artifact_bytes(payload, allow_trailing=False)
    except ValueError as exc:
        if expect_success:
            raise AssertionError(f"{name}: expected success, got {exc}") from exc
        return
    if not expect_success:
        raise AssertionError(f"{name}: expected parse failure")


def build_base_corpus(rng: random.Random) -> list[bytes]:
    bases: list[bytes] = []
    for frame_count, schema_len in [(1, 0), (1, 64), (32, 0), (32, 64)]:
        artifact, _, _ = build_valid_v1_artifact(
            rng, frame_count=frame_count, schema_len=schema_len
        )
        bases.append(artifact)
    for _ in range(BASE_CORPUS_SIZE - len(bases)):
        artifact, _, _ = build_valid_v1_artifact(rng)
        bases.append(artifact)
    return bases


def main() -> int:
    rng = random.Random(0)
    base_corpus = build_base_corpus(rng)
    mutation_set = REJECT_MUTATIONS + ACCEPT_MUTATIONS

    case_count = 0
    for base_idx, artifact in enumerate(base_corpus):
        parse_expect(f"base[{base_idx}]", artifact, expect_success=True)
        for mutation in mutation_set:
            mutated = mutation.fn(artifact)
            if mutated == artifact:
                raise AssertionError(
                    f"base[{base_idx}] {mutation.name}: mutation produced identical bytes"
                )
            parse_expect(
                f"base[{base_idx}] {mutation.name}",
                mutated,
                expect_success=mutation.expect_success,
            )
            case_count += 1

    print(
        "PASS: mutation corpus "
        f"(bases={len(base_corpus)}, reject_mutations={len(REJECT_MUTATIONS)}, "
        f"accept_mutations={len(ACCEPT_MUTATIONS)}, total={case_count})"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
