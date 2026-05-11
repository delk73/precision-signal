#!/usr/bin/env python3
"""Validate MSP and reset vector read from STM32 flash."""

from __future__ import annotations

import argparse
import struct
import sys
from pathlib import Path


def int_auto(value: str) -> int:
    return int(value, 0)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("image", type=Path)
    parser.add_argument("--ram-start", type=int_auto, default=0x20000000)
    parser.add_argument("--ram-end", type=int_auto, default=0x20020000)
    parser.add_argument("--flash-start", type=int_auto, default=0x08000000)
    parser.add_argument("--flash-end", type=int_auto, default=0x08080000)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    data = args.image.read_bytes()
    if len(data) < 8:
        print(f"{args.image} is too small for MSP/reset-vector validation", file=sys.stderr)
        return 1

    msp, reset_vector = struct.unpack("<II", data[:8])
    if not args.ram_start <= msp <= args.ram_end:
        print(f"MSP outside SRAM range: {hex(msp)}", file=sys.stderr)
        return 1
    if (reset_vector & 1) != 1:
        print(f"reset vector does not have Thumb bit set: {hex(reset_vector)}", file=sys.stderr)
        return 1
    reset_addr = reset_vector & ~1
    if not args.flash_start <= reset_addr <= args.flash_end:
        print(f"reset vector outside flash range: {hex(reset_vector)}", file=sys.stderr)
        return 1

    print(f"OK: MSP {hex(msp)} ResetVec {hex(reset_vector)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
