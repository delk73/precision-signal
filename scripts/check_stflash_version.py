#!/usr/bin/env python3
"""Enforce the repo-pinned st-flash host tool version."""

from __future__ import annotations

import argparse
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path


REQUIRED_VERSION = "1.8.0"
VERSION_PATTERNS = (
    re.compile(r"\bst-flash\s+v?([0-9]+(?:\.[0-9]+){1,2})\b", re.IGNORECASE),
    re.compile(r"\bv?([0-9]+(?:\.[0-9]+){1,2})\b", re.IGNORECASE),
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Check the st-flash version pin.")
    parser.add_argument("--stflash", required=True, help="st-flash executable path/name")
    parser.add_argument(
        "--required-version",
        default=REQUIRED_VERSION,
        help=argparse.SUPPRESS,
    )
    return parser.parse_args()


def resolve_executable(stflash: str) -> Path:
    if os.sep in stflash or (os.altsep is not None and os.altsep in stflash):
        path = Path(stflash)
    else:
        found = shutil.which(stflash)
        if found is None:
            raise ValueError(
                f"missing ST-LINK flash binary: {stflash}. "
                "Install stlink tools or pass STFLASH=/path/to/st-flash."
            )
        path = Path(found)

    if not path.exists():
        raise ValueError(
            f"missing ST-LINK flash binary: {stflash}. "
            "Install stlink tools or pass STFLASH=/path/to/st-flash."
        )
    if not path.is_file():
        raise ValueError(f"ST-LINK flash binary is not a file: {path}")
    if not os.access(path, os.X_OK):
        raise ValueError(f"ST-LINK flash binary is not executable: {path}")
    return path


def parse_version(output: str) -> str:
    match = None
    for pattern in VERSION_PATTERNS:
        match = pattern.search(output)
        if match is not None:
            break
    if match is None:
        raise ValueError(
            f"could not parse st-flash version from output: {output.strip()!r}"
        )
    return match.group(1)


def check_version(stflash: str, required_version: str) -> str:
    path = resolve_executable(stflash)
    result = subprocess.run(
        [str(path), "--version"],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
    )
    output = result.stdout or ""
    if result.returncode != 0:
        raise ValueError(
            f"could not run st-flash --version for {path}: exit {result.returncode}: "
            f"{output.strip()!r}"
        )
    version = parse_version(output)
    if version != required_version:
        raise RuntimeError(
            f"st-flash version mismatch: required {required_version}, found {version}"
        )
    return version


def main() -> int:
    args = parse_args()
    try:
        version = check_version(args.stflash, args.required_version)
    except RuntimeError as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        return 1
    except (OSError, ValueError) as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        return 1

    print(f"OK: st-flash version {version}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
