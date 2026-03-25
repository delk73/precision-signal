#!/usr/bin/env python3
"""Generate deterministic Demo V5 divergence-evolution fixtures."""

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

TIMER_DELTA_OFFSET = 8
SAMPLE_OFFSET = 12
DEFAULT_BASE = Path("artifacts/demo_persistent/run_A.rpl")
DEFAULT_OUT_DIR = Path("artifacts/demo_v5")
FIRST_DIVERGENCE_FRAME = 4096


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate synthetic RPL0 fixtures for Demo V5 evolution semantics."
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


def mutate_self_healing(parsed: dict, data: bytearray, start_frame: int) -> None:
    sample = read_i32(parsed, data, start_frame, SAMPLE_OFFSET)
    write_i32(parsed, data, start_frame, SAMPLE_OFFSET, sample + 1)


def mutate_bounded_persistent(parsed: dict, data: bytearray, start_frame: int) -> None:
    for frame_idx in range(start_frame, parsed["frame_count"]):
        sample = read_i32(parsed, data, frame_idx, SAMPLE_OFFSET)
        write_i32(parsed, data, frame_idx, SAMPLE_OFFSET, sample + 1)


def mutate_monotonic_growth(parsed: dict, data: bytearray, start_frame: int) -> None:
    growth = 1
    for frame_idx in range(start_frame, parsed["frame_count"]):
        sample = read_i32(parsed, data, frame_idx, SAMPLE_OFFSET)
        write_i32(parsed, data, frame_idx, SAMPLE_OFFSET, sample + growth)
        growth += 1


def mutate_region_transition(parsed: dict, data: bytearray, start_frame: int) -> None:
    delta = read_u32(parsed, data, start_frame, TIMER_DELTA_OFFSET)
    write_u32(parsed, data, start_frame, TIMER_DELTA_OFFSET, delta + 1)
    for frame_idx in range(start_frame + 2, parsed["frame_count"]):
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
    if args.start_frame + 2 >= parsed["frame_count"]:
        print(
            f"FAIL: start frame {args.start_frame} leaves no room for region transition "
            f"(frame_count={parsed['frame_count']})"
        )
        return 1

    out_dir = args.out_dir
    cases = {
        "self_healing": (bytes(base_data), bytearray(base_data)),
        "bounded_persistent": (bytes(base_data), bytearray(base_data)),
        "monotonic_growth": (bytes(base_data), bytearray(base_data)),
        "region_transition": (bytes(base_data), bytearray(base_data)),
    }

    mutate_self_healing(parsed, cases["self_healing"][1], args.start_frame)
    mutate_bounded_persistent(parsed, cases["bounded_persistent"][1], args.start_frame)
    mutate_monotonic_growth(parsed, cases["monotonic_growth"][1], args.start_frame)
    mutate_region_transition(parsed, cases["region_transition"][1], args.start_frame)

    wrote_paths: list[Path] = []
    for stem, (data_a, data_b) in cases.items():
        path_a = out_dir / f"{stem}_A.rpl"
        path_b = out_dir / f"{stem}_B.rpl"
        emit_fixture(path_a, data_a)
        emit_fixture(path_b, bytes(data_b))
        wrote_paths.extend((path_a, path_b))

    print("PASS: demo-v5 fixtures generated")
    print(f"base: {args.base}")
    print(f"out_dir: {out_dir}")
    print(f"first_divergence_frame: {args.start_frame}")
    for path in wrote_paths:
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        print(f"wrote: {path} sha256={digest}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
