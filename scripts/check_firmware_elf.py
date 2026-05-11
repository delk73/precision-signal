#!/usr/bin/env python3
"""Validate basic STM32 firmware ELF structure."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("elf", type=Path)
    parser.add_argument("--readelf", default="readelf")
    return parser.parse_args()


def readelf(args: argparse.Namespace, *flags: str) -> str:
    result = subprocess.run(
        [args.readelf, *flags, str(args.elf)],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        print(result.stdout, end="")
        raise SystemExit(result.returncode)
    return result.stdout


def main() -> int:
    args = parse_args()
    if not args.elf.is_file():
        print(f"missing firmware ELF: {args.elf}", file=sys.stderr)
        return 1

    sections = readelf(args, "-S")
    if ".text" not in sections:
        print("firmware ELF is missing .text", file=sys.stderr)
        return 1
    if ".vector_table" not in sections and ".isr_vector" not in sections:
        print("firmware ELF is missing a vector table section", file=sys.stderr)
        return 1

    header = readelf(args, "-h")
    match = re.search(r"Entry point address:\s*(0x[0-9a-fA-F]+)", header)
    if match is None:
        print("firmware ELF entry point not found", file=sys.stderr)
        return 1
    if int(match.group(1), 16) == 0:
        print("firmware ELF entry point is zero", file=sys.stderr)
        return 1

    print(f"OK: firmware ELF sanity checks passed for {args.elf}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
