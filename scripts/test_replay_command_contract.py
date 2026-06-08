#!/usr/bin/env python3
"""Regression tests for the retained replay/release command contract."""

from __future__ import annotations

import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
RETAINED_VERSION = "1.9.1"

CONTRACT_TARGETS = (
    "gate",
    "authoritative-replay-cli-tests",
    "replay-witness-check",
    "release-bundle-check",
)

DRY_RUN_CASES = (
    (
        ("make", "-n", "gate"),
        ("sig-util", "validate", "--mode quick"),
    ),
    (
        ("make", "-n", "authoritative-replay-cli-tests"),
        ("cargo test", "precision-cli", "precision_authoritative_surface"),
    ),
    (
        ("make", "-n", "replay-witness-check", f"VERSION={RETAINED_VERSION}"),
        ("scripts/check_replay_witness.py", "--version", RETAINED_VERSION),
    ),
    (
        ("make", "-n", "release-bundle-check", f"VERSION={RETAINED_VERSION}"),
        ("scripts/check_release_bundle.py", "--version", RETAINED_VERSION),
    ),
)

HELP_COMMANDS = (
    "make gate",
    "make authoritative-replay-cli-tests",
    "make release-bundle-check VERSION=1.9.1",
    "make replay-witness-check VERSION=1.9.1",
)

README_COMMANDS = (
    "make gate",
    "make authoritative-replay-cli-tests",
    "make replay-witness-check VERSION=1.9.1",
    "make release-bundle-check VERSION=1.9.1",
)

RETAINED_INDEX_COMMANDS = (
    "make gate",
    "make authoritative-replay-cli-tests",
    "make release-bundle-check VERSION=1.9.1",
)


def read_text(rel_path: str) -> str:
    return (REPO_ROOT / rel_path).read_text(encoding="utf-8")


def assert_contains(name: str, text: str, needles: tuple[str, ...]) -> None:
    missing = [needle for needle in needles if needle not in text]
    if missing:
        raise AssertionError(f"{name}: missing expected text: {missing!r}")


def run_command(args: tuple[str, ...]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        list(args),
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )


def assert_command_ok(name: str, proc: subprocess.CompletedProcess[str]) -> str:
    output = proc.stdout + proc.stderr
    if proc.returncode != 0:
        raise AssertionError(
            f"{name}: expected success, rc={proc.returncode}\n"
            f"stdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )
    return output


def check_phony_targets() -> None:
    makefile = read_text("Makefile")
    phony_lines = [line for line in makefile.splitlines() if line.startswith(".PHONY:")]
    if not phony_lines:
        raise AssertionError("Makefile: missing .PHONY declaration")
    phony_targets = set()
    for line in phony_lines:
        phony_targets.update(line.removeprefix(".PHONY:").split())
    missing = sorted(set(CONTRACT_TARGETS) - phony_targets)
    if missing:
        raise AssertionError(f"Makefile .PHONY missing replay command targets: {missing!r}")


def check_dry_run_routes() -> None:
    for args, needles in DRY_RUN_CASES:
        name = " ".join(args)
        output = assert_command_ok(name, run_command(args))
        assert_contains(name, output, needles)


def check_help_surface() -> None:
    output = assert_command_ok("make help", run_command(("make", "help")))
    assert_contains("make help", output, HELP_COMMANDS)


def check_retained_review_surfaces() -> None:
    assert_contains("README.md", read_text("README.md"), README_COMMANDS)
    assert_contains(
        "docs/verification/releases/1.9.1/index.md",
        read_text("docs/verification/releases/1.9.1/index.md"),
        RETAINED_INDEX_COMMANDS,
    )


def main() -> int:
    check_phony_targets()
    check_dry_run_routes()
    check_help_surface()
    check_retained_review_surfaces()
    print("PASS: replay command contract regression suite")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
