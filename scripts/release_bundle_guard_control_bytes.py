#!/usr/bin/env python3
"""Fail if retained transcripts contain control bytes outside TAB/LF."""

from __future__ import annotations

import argparse
from pathlib import Path

TRANSCRIPTS = (
    "cargo_check_dpw4_thumb_locked.txt",
    "kani_evidence.txt",
    "make_gate.txt",
    "release_reproducibility.txt",
)


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser()
    p.add_argument("--dir", required=True)
    return p.parse_args()


def has_forbidden_control_byte(data: bytes) -> bool:
    for b in data:
        if (b < 0x20 and b not in (0x09, 0x0A)) or b == 0x7F:
            return True
    return False


def main() -> int:
    args = parse_args()
    root = Path(args.dir)
    bad: list[str] = []

    for name in TRANSCRIPTS:
        path = root / name
        if not path.is_file():
            continue
        if has_forbidden_control_byte(path.read_bytes()):
            bad.append(str(path))

    if bad:
        print("control bytes detected in retained transcripts")
        for entry in bad:
            print(entry)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
