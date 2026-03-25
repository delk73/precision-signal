#!/usr/bin/env python3
"""Generate deterministic Demo V3 divergence-classification fixtures."""

from __future__ import annotations

import argparse
import struct
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import inspect_artifact

SAMPLE_OFFSET = 12
S32_MIN = -(1 << 31)
S32_MAX = (1 << 31) - 1
DEFAULT_BASE = Path("artifacts/demo_persistent/run_A.rpl")
DEFAULT_OUT_DIR = Path("artifacts/demo_v3")
FIRST_DIVERGENCE_FRAME = 4096
RATE_GROWTH_FRAMES = 64


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate synthetic RPL0 fixtures for Demo V3 divergence classification."
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
    parser.add_argument(
        "--rate-growth-frames",
        type=int,
        default=RATE_GROWTH_FRAMES,
        help=(
            "Frames over which rate fixture offset grows before clamping "
            f"(default: {RATE_GROWTH_FRAMES})."
        ),
    )
    return parser.parse_args()


def load_base(path: Path) -> tuple[dict, bytearray]:
    parsed = inspect_artifact.parse_artifact(path, allow_trailing=False)
    return parsed, bytearray(path.read_bytes())


def write_sample(parsed: dict, data: bytearray, frame_idx: int, value: int) -> None:
    if value < S32_MIN or value > S32_MAX:
        raise ValueError(f"sample out of i32 range at frame {frame_idx}: {value}")
    frame_size = parsed["frame_size"]
    frame_base = parsed["frames_offset"] + (frame_idx * frame_size)
    sample_offset = frame_base + SAMPLE_OFFSET
    struct.pack_into("<i", data, sample_offset, value)


def read_sample(parsed: dict, data: bytes | bytearray, frame_idx: int) -> int:
    frame_size = parsed["frame_size"]
    frame_base = parsed["frames_offset"] + (frame_idx * frame_size)
    sample_offset = frame_base + SAMPLE_OFFSET
    return struct.unpack_from("<i", data, sample_offset)[0]


def verify_artifact(path: Path) -> None:
    inspect_artifact.parse_artifact(path, allow_trailing=False)


def emit_fixture(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    verify_artifact(path)


def main() -> int:
    args = parse_args()

    if args.start_frame < 0:
        print(f"FAIL: start frame must be non-negative (got {args.start_frame})")
        return 1
    if args.rate_growth_frames <= 0:
        print(
            "FAIL: rate growth frames must be positive "
            f"(got {args.rate_growth_frames})"
        )
        return 1

    try:
        parsed, base_data = load_base(args.base)
    except ValueError as exc:
        print(f"FAIL: invalid base artifact ({exc})")
        return 1
    except OSError as exc:
        print(f"FAIL: base artifact read error ({exc})")
        return 1

    frame_count = parsed["frame_count"]
    start = args.start_frame
    if start >= frame_count:
        print(f"FAIL: start frame {start} out of range (frame_count={frame_count})")
        return 1

    out_dir = args.out_dir

    # A artifacts are identical to the base run.
    transient_a = out_dir / "transient_A.rpl"
    offset_a = out_dir / "offset_A.rpl"
    rate_a = out_dir / "rate_A.rpl"
    emit_fixture(transient_a, bytes(base_data))
    emit_fixture(offset_a, bytes(base_data))
    emit_fixture(rate_a, bytes(base_data))

    # Transient divergence: mismatch at start and start+1, reconverge at start+2.
    transient_b_data = bytearray(base_data)
    for frame_idx in (start, start + 1):
        if frame_idx < frame_count:
            sample = read_sample(parsed, transient_b_data, frame_idx)
            write_sample(parsed, transient_b_data, frame_idx, sample + 1)
    transient_b = out_dir / "transient_B.rpl"
    emit_fixture(transient_b, bytes(transient_b_data))

    # Persistent offset divergence: +1 from start onward.
    offset_b_data = bytearray(base_data)
    for frame_idx in range(start, frame_count):
        sample = read_sample(parsed, offset_b_data, frame_idx)
        write_sample(parsed, offset_b_data, frame_idx, sample + 1)
    offset_b = out_dir / "offset_B.rpl"
    emit_fixture(offset_b, bytes(offset_b_data))

    # Rate divergence: bounded growth over a finite window, then clamped offset.
    rate_b_data = bytearray(base_data)
    for frame_idx in range(start, frame_count):
        sample = read_sample(parsed, rate_b_data, frame_idx)
        rel = frame_idx - start + 1
        offset = min(rel, args.rate_growth_frames)
        write_sample(parsed, rate_b_data, frame_idx, sample + offset)
    rate_b = out_dir / "rate_B.rpl"
    emit_fixture(rate_b, bytes(rate_b_data))

    print("PASS: demo-v3 fixtures generated")
    print(f"base: {args.base}")
    print(f"out_dir: {out_dir}")
    print(f"first_divergence_frame: {start}")
    print(f"rate_growth_frames: {args.rate_growth_frames}")
    for path in (
        transient_a,
        transient_b,
        offset_a,
        offset_b,
        rate_a,
        rate_b,
    ):
        print(f"wrote: {path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
