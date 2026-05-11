#!/usr/bin/env python3
"""Generate retained release verification evidence."""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--version", required=True)
    parser.add_argument("--release-root", default="docs/verification/releases")
    parser.add_argument("--fw-target", default="thumbv7em-none-eabihf")
    parser.add_argument("--cargo", default="cargo")
    parser.add_argument("--dpw4-pkg", default="dpw4")
    parser.add_argument("--make", default="make")
    return parser.parse_args()


def workspace_version(cargo_toml: Path) -> str:
    text = cargo_toml.read_text(encoding="utf-8")
    match = re.search(r'^version = "([^"]+)"', text, flags=re.MULTILINE)
    if match is None:
        raise RuntimeError(f"workspace version not found in {cargo_toml}")
    return match.group(1)


def run_transcript(
    *,
    label: str,
    command: list[str],
    output_path: Path,
    env: dict[str, str] | None = None,
) -> None:
    print(label)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    merged_env = os.environ.copy()
    if env:
        merged_env.update(env)
    with output_path.open("wb") as handle:
        result = subprocess.run(
            command,
            stdout=handle,
            stderr=subprocess.STDOUT,
            env=merged_env,
            check=False,
        )
    if result.returncode != 0:
        print(f"[release-bundle] FAIL file={output_path}", file=sys.stderr)
        raise SystemExit(result.returncode)


def run_guard(label: str, command: list[str]) -> None:
    print(label)
    result = subprocess.run(command, check=False)
    if result.returncode != 0:
        print(f"[release-bundle] FAIL guard={label}", file=sys.stderr)
        raise SystemExit(result.returncode)


def generate_summary(release_dir: Path, requested: str) -> None:
    run_guard(
        "[release-bundle] summary.md summary.json",
        [
            sys.executable,
            "scripts/release_summary.py",
            "--version",
            requested,
            "--bundle-dir",
            str(release_dir),
        ],
    )


def main() -> int:
    args = parse_args()
    repo_root = Path.cwd()
    requested = args.version
    found = workspace_version(repo_root / "Cargo.toml")
    if found != requested:
        print(
            "[release-bundle] FAIL metadata: "
            f"workspace version={found}, requested VERSION={requested}",
            file=sys.stderr,
        )
        return 1

    release_dir = repo_root / args.release_root / requested
    release_dir.mkdir(parents=True, exist_ok=True)

    run_transcript(
        label="[release-bundle] 1/6 cargo_check_dpw4_thumb_locked.txt",
        command=[args.cargo, "check", "--locked", "-p", args.dpw4_pkg, "--target", args.fw_target],
        output_path=release_dir / "cargo_check_dpw4_thumb_locked.txt",
    )
    run_transcript(
        label="[release-bundle] 2/6 kani_evidence.txt",
        command=["bash", "scripts/verify_kani.sh"],
        output_path=release_dir / "kani_evidence.txt",
        env={"NO_COLOR": "1"},
    )
    run_transcript(
        label="[release-bundle] 3/6 make_gate.txt",
        command=[args.make, "--no-print-directory", "gate"],
        output_path=release_dir / "make_gate.txt",
    )
    run_transcript(
        label="[release-bundle] 4/6 release_reproducibility.txt",
        command=["bash", "scripts/verify_release_repro.sh"],
        output_path=release_dir / "release_reproducibility.txt",
        env={"RELEASE_EVIDENCE_DIR": str(release_dir)},
    )
    run_guard(
        "[release-bundle] 5/6 stale prior-release guard",
        [
            sys.executable,
            "scripts/release_bundle_guard_stale.py",
            "--dir",
            str(release_dir),
            "--version",
            requested,
        ],
    )
    run_guard(
        "[release-bundle] 6/6 control-byte guard (non-lossy)",
        [
            sys.executable,
            "scripts/release_bundle_guard_control_bytes.py",
            "--dir",
            str(release_dir),
        ],
    )
    generate_summary(release_dir, requested)
    print(f"[release-bundle] OK bundle generated: {release_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
