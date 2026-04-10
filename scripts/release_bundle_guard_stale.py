#!/usr/bin/env python3
"""Fail if retained transcripts contain stale workspace crate versions."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

WORKSPACE_CRATES = {
    "audit-float-boundary",
    "dpw4",
    "geom-signal",
    "geom-spatial",
    "replay-cli",
    "replay-core",
    "replay-embed",
    "replay-fw-f446",
    "replay-host",
    "xtask",
}

TRANSCRIPTS = (
    "cargo_check_dpw4_thumb_locked.txt",
    "kani_evidence.txt",
    "make_gate.txt",
    "verify_release_repro.txt",
)

COMPILE_LINE = re.compile(r"Compiling\s+([A-Za-z0-9_-]+)\s+v([0-9]+\.[0-9]+\.[0-9]+)")


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser()
    p.add_argument("--dir", required=True)
    p.add_argument("--version", required=True)
    return p.parse_args()


def main() -> int:
    args = parse_args()
    root = Path(args.dir)
    stale: list[str] = []

    for name in TRANSCRIPTS:
        path = root / name
        if not path.is_file():
            # Keep this guard focused: producer phases are responsible for file creation.
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        for idx, line in enumerate(text.splitlines(), start=1):
            m = COMPILE_LINE.search(line)
            if not m:
                continue
            crate, seen = m.group(1), m.group(2)
            if crate in WORKSPACE_CRATES and seen != args.version:
                stale.append(f"{path}:{idx}: {crate} v{seen} (expected v{args.version})")

    if stale:
        print("stale prior-release workspace evidence detected")
        for entry in stale:
            print(entry)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
