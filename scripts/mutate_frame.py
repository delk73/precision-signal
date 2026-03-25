#!/usr/bin/env python3
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


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Mutate one artifact frame sample by a signed delta."
    )
    parser.add_argument("artifact", type=Path, help="Input artifact path.")
    parser.add_argument("frame", type=int, help="0-based frame index to mutate.")
    parser.add_argument("delta", type=int, help="Signed sample delta (for example: +1).")
    parser.add_argument("--out", type=Path, required=True, help="Output artifact path.")
    return parser.parse_args()


def fail(message: str) -> int:
    print(f"FAIL: {message}")
    return 1


def main() -> int:
    args = parse_args()

    try:
        parsed = inspect_artifact.parse_artifact(args.artifact, allow_trailing=False)
    except ValueError as exc:
        return fail(f"invalid input artifact ({exc})")

    frame_count = parsed["frame_count"]
    frame_size = parsed["frame_size"]
    if args.frame < 0 or args.frame >= frame_count:
        return fail(f"frame index out of range: {args.frame} (frame_count={frame_count})")

    if frame_size < SAMPLE_OFFSET + 4:
        return fail(f"frame_size too small for sample field: {frame_size}")

    data = bytearray(args.artifact.read_bytes())
    frame_base = parsed["frames_offset"] + (args.frame * frame_size)
    sample_offset = frame_base + SAMPLE_OFFSET
    sample_before = struct.unpack_from("<i", data, sample_offset)[0]
    sample_after = sample_before + args.delta
    if sample_after < S32_MIN or sample_after > S32_MAX:
        return fail(
            f"sample overflow at frame {args.frame}: {sample_before} + {args.delta} out of i32 range"
        )

    struct.pack_into("<i", data, sample_offset, sample_after)

    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_bytes(data)

    try:
        inspect_artifact.parse_artifact(args.out, allow_trailing=False)
    except ValueError as exc:
        return fail(f"output artifact invalid after mutation ({exc})")

    print("PASS: frame mutated")
    print(f"artifact_in: {args.artifact}")
    print(f"artifact_out: {args.out}")
    print(f"frame: {args.frame}")
    print("field: sample")
    print(f"delta: {args.delta:+d}")
    print(f"sample_before_u32: 0x{(sample_before & 0xFFFFFFFF):08X}")
    print(f"sample_after_u32:  0x{(sample_after & 0xFFFFFFFF):08X}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
