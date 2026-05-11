#!/usr/bin/env python3
"""Retain firmware capture evidence in a release directory."""

from __future__ import annotations

import argparse
import hashlib
import shutil
import sys
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--version", required=True)
    parser.add_argument("--release-root", default="docs/verification/releases")
    parser.add_argument("--replay-run", required=True, type=Path)
    parser.add_argument("--repeat-dir", required=True, type=Path)
    return parser.parse_args()


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main() -> int:
    args = parse_args()
    if not args.replay_run.is_file():
        print(f"missing replay capture: {args.replay_run}", file=sys.stderr)
        return 1
    manifest = args.repeat_dir / "replay_manifest_v1.txt"
    if not manifest.is_file():
        print(f"missing repeat manifest: {manifest}", file=sys.stderr)
        return 1
    repeat_runs = sorted(args.repeat_dir.glob("run_*.bin"))
    if not repeat_runs:
        print(f"missing repeat run artifacts in {args.repeat_dir}", file=sys.stderr)
        return 1

    release_dir = Path(args.release_root) / args.version
    release_dir.mkdir(parents=True, exist_ok=True)
    capture_dst = release_dir / "fw_capture.bin"
    repeat_dst = release_dir / "fw_repeat"
    if capture_dst.exists():
        capture_dst.unlink()
    if repeat_dst.exists():
        shutil.rmtree(repeat_dst)

    shutil.copy(args.replay_run, capture_dst)
    shutil.copytree(args.repeat_dir, repeat_dst, copy_function=shutil.copy)

    capture_hash = release_dir / "fw_capture_hash_check.txt"
    repeat_hash = release_dir / "fw_repeat_hash_check.txt"
    capture_hash.write_text(f"{sha256(args.replay_run)}  {args.replay_run}\n", encoding="utf-8")
    repeat_hash.write_text(
        "".join(f"{sha256(path)}  {path}\n" for path in repeat_runs),
        encoding="utf-8",
    )

    evidence = release_dir / "firmware_release_evidence.md"
    with evidence.open("w", encoding="utf-8") as handle:
        handle.write(f"# Firmware Release Evidence ({args.version})\n")
        handle.write("\n")
        handle.write(f"REPLAY_RUN={args.replay_run}\n")
        handle.write(f"REPLAY_REPEAT_DIR={args.repeat_dir}\n")
        handle.write("\n")
        handle.write("## capture hash check\n")
        handle.write(capture_hash.read_text(encoding="utf-8"))
        handle.write("\n")
        handle.write("## repeat manifest\n")
        handle.write(manifest.read_text(encoding="utf-8"))
    print(f"Archived firmware evidence in {release_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
