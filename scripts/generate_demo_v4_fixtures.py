#!/usr/bin/env python3
"""Generate deterministic Demo V4 divergence-region-attribution fixtures."""

from __future__ import annotations

import argparse
import hashlib
import struct
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import inspect_artifact

IRQ_OFFSET = 4
TIMER_DELTA_OFFSET = 8
SAMPLE_OFFSET = 12
DEFAULT_BASE = Path("artifacts/demo_persistent/run_A.rpl")
DEFAULT_OUT_DIR = Path("artifacts/demo_v4")
FIRST_DIVERGENCE_FRAME = 4096
V1_HEADER_LEN = inspect_artifact.V1_MIN_HEADER_SIZE


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate synthetic RPL0 fixtures for Demo V4 region attribution."
    )
    parser.add_argument(
        "--base",
        type=Path,
        default=DEFAULT_BASE,
        help=f"Base artifact path (default: {DEFAULT_BASE}).",
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=DEFAULT_OUT_DIR,
        help=f"Output directory for generated fixtures (default: {DEFAULT_OUT_DIR}).",
    )
    parser.add_argument(
        "--start-frame",
        type=int,
        default=FIRST_DIVERGENCE_FRAME,
        help=f"First divergence frame index (default: {FIRST_DIVERGENCE_FRAME}).",
    )
    return parser.parse_args()


def load_base(path: Path) -> tuple[dict, bytearray]:
    parsed = inspect_artifact.parse_artifact(path, allow_trailing=False)
    return parsed, bytearray(path.read_bytes())


def frame_base(parsed: dict, frame_idx: int) -> int:
    return parsed["frames_offset"] + (frame_idx * parsed["frame_size"])


def read_u8(parsed: dict, data: bytes | bytearray, frame_idx: int, rel_offset: int) -> int:
    return data[frame_base(parsed, frame_idx) + rel_offset]


def write_u8(parsed: dict, data: bytearray, frame_idx: int, rel_offset: int, value: int) -> None:
    data[frame_base(parsed, frame_idx) + rel_offset] = value & 0xFF


def read_u32(parsed: dict, data: bytes | bytearray, frame_idx: int, rel_offset: int) -> int:
    return struct.unpack_from("<I", data, frame_base(parsed, frame_idx) + rel_offset)[0]


def write_u32(parsed: dict, data: bytearray, frame_idx: int, rel_offset: int, value: int) -> None:
    struct.pack_into("<I", data, frame_base(parsed, frame_idx) + rel_offset, value & 0xFFFFFFFF)


def read_i32(parsed: dict, data: bytes | bytearray, frame_idx: int, rel_offset: int) -> int:
    return struct.unpack_from("<i", data, frame_base(parsed, frame_idx) + rel_offset)[0]


def write_i32(parsed: dict, data: bytearray, frame_idx: int, rel_offset: int, value: int) -> None:
    struct.pack_into("<i", data, frame_base(parsed, frame_idx) + rel_offset, value)


def emit_fixture(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    inspect_artifact.parse_artifact(path, allow_trailing=False)


def make_v1_header_schema_artifact(parsed: dict, data: bytes) -> bytes:
    schema = b""
    header = bytearray(V1_HEADER_LEN)
    header[:4] = inspect_artifact.MAGIC
    struct.pack_into("<H", header, inspect_artifact.V1_OFF_VERSION, 1)
    struct.pack_into("<H", header, inspect_artifact.V1_OFF_HEADER_LEN, V1_HEADER_LEN)
    struct.pack_into("<I", header, inspect_artifact.V1_OFF_FRAME_COUNT, parsed["frame_count"])
    struct.pack_into("<H", header, inspect_artifact.V1_OFF_FRAME_SIZE, parsed["frame_size"])
    struct.pack_into("<H", header, inspect_artifact.V1_OFF_FLAGS, 0)
    struct.pack_into("<I", header, inspect_artifact.V1_OFF_SCHEMA_LEN, len(schema))
    header[
        inspect_artifact.V1_OFF_SCHEMA_HASH : inspect_artifact.V1_OFF_SCHEMA_HASH + 32
    ] = hashlib.sha256(schema).digest()
    header[
        inspect_artifact.V1_OFF_BUILD_HASH : inspect_artifact.V1_OFF_BUILD_HASH + 32
    ] = bytes.fromhex("11" * 32)
    header[
        inspect_artifact.V1_OFF_CONFIG_HASH : inspect_artifact.V1_OFF_CONFIG_HASH + 32
    ] = bytes.fromhex("22" * 32)
    header[
        inspect_artifact.V1_OFF_BOARD_ID : inspect_artifact.V1_OFF_BOARD_ID + 16
    ] = b"demo-v4-board-01"
    header[
        inspect_artifact.V1_OFF_CLOCK_PROFILE : inspect_artifact.V1_OFF_CLOCK_PROFILE + 16
    ] = b"offline-fixed-v1"
    struct.pack_into("<H", header, inspect_artifact.V1_OFF_CAPTURE_BOUNDARY, 0)
    struct.pack_into("<H", header, inspect_artifact.V1_OFF_RESERVED, 0)
    frames = data[parsed["frames_offset"] : parsed["canonical_len"]]
    return bytes(header) + schema + frames


def mutate_timer_delta(parsed: dict, data: bytearray, frame_idx: int) -> None:
    value = read_u32(parsed, data, frame_idx, TIMER_DELTA_OFFSET)
    write_u32(parsed, data, frame_idx, TIMER_DELTA_OFFSET, value + 1)


def mutate_irq(parsed: dict, data: bytearray, frame_idx: int) -> None:
    value = read_u8(parsed, data, frame_idx, IRQ_OFFSET)
    write_u8(parsed, data, frame_idx, IRQ_OFFSET, value + 1)


def mutate_sample_persistent(parsed: dict, data: bytearray, start_frame: int) -> None:
    for frame_idx in range(start_frame, parsed["frame_count"]):
        sample = read_i32(parsed, data, frame_idx, SAMPLE_OFFSET)
        write_i32(parsed, data, frame_idx, SAMPLE_OFFSET, sample + 1)


def main() -> int:
    args = parse_args()
    if args.start_frame < 0:
        print(f"FAIL: start frame must be non-negative (got {args.start_frame})")
        return 1

    try:
        parsed, base_data = load_base(args.base)
    except ValueError as exc:
        print(f"FAIL: invalid base artifact ({exc})")
        return 1
    except OSError as exc:
        print(f"FAIL: base artifact read error ({exc})")
        return 1

    if args.start_frame >= parsed["frame_count"]:
        print(
            f"FAIL: start frame {args.start_frame} out of range "
            f"(frame_count={parsed['frame_count']})"
        )
        return 1

    out_dir = args.out_dir
    cases = {
        "header_schema": (bytes(base_data), bytearray(base_data)),
        "header_schema_sample_payload": (bytes(base_data), bytearray(base_data)),
        "timer_delta": (bytes(base_data), bytearray(base_data)),
        "irq_state": (bytes(base_data), bytearray(base_data)),
        "sample_payload": (bytes(base_data), bytearray(base_data)),
        "mixed": (bytes(base_data), bytearray(base_data)),
    }

    header_schema_data = make_v1_header_schema_artifact(parsed, bytes(base_data))
    cases["header_schema"] = (bytes(base_data), bytearray(header_schema_data))
    header_schema_sample_data = bytearray(header_schema_data)
    header_schema_sample_parsed = inspect_artifact.parse_artifact_bytes(
        bytes(header_schema_sample_data), allow_trailing=False
    )
    mutate_sample_persistent(header_schema_sample_parsed, header_schema_sample_data, 0)
    cases["header_schema_sample_payload"] = (
        bytes(base_data),
        header_schema_sample_data,
    )
    mutate_timer_delta(parsed, cases["timer_delta"][1], args.start_frame)
    mutate_irq(parsed, cases["irq_state"][1], args.start_frame)
    mutate_sample_persistent(parsed, cases["sample_payload"][1], args.start_frame)
    mutate_timer_delta(parsed, cases["mixed"][1], args.start_frame)
    mutate_sample_persistent(parsed, cases["mixed"][1], args.start_frame)

    wrote_paths: list[Path] = []
    for stem, (data_a, data_b) in cases.items():
        path_a = out_dir / f"{stem}_A.rpl"
        path_b = out_dir / f"{stem}_B.rpl"
        emit_fixture(path_a, data_a)
        emit_fixture(path_b, bytes(data_b))
        wrote_paths.extend((path_a, path_b))

    print("PASS: demo-v4 fixtures generated")
    print(f"base: {args.base}")
    print(f"out_dir: {out_dir}")
    print(f"first_divergence_frame: {args.start_frame}")
    for path in wrote_paths:
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        print(f"wrote: {path} sha256={digest}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
