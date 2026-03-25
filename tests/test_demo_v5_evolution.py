#!/usr/bin/env python3
"""Deterministic regression tests for Demo V5 evolution semantics."""

from __future__ import annotations

import hashlib
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
FIXTURE_GEN = ["python3", "scripts/generate_demo_v5_fixtures.py", "--out-dir", "artifacts/demo_v5"]
DIFF = ["python3", "scripts/artifact_diff.py"]
EXPECTED_SHA256 = {
    "artifacts/demo_v5/self_healing_A.rpl": "b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3",
    "artifacts/demo_v5/self_healing_B.rpl": "5957b539722fdd0021b56882c7eb04b9c68ef16484c18b6293cb4ff0d80a5d6d",
    "artifacts/demo_v5/bounded_persistent_A.rpl": "b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3",
    "artifacts/demo_v5/bounded_persistent_B.rpl": "b0f93d0c2e4a674c30c358f799d0b798a7d4989220477583c53d37f08fa48256",
    "artifacts/demo_v5/monotonic_growth_A.rpl": "b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3",
    "artifacts/demo_v5/monotonic_growth_B.rpl": "126bacf334716541ecbc7cbd1f57a7b5ae573d8f9feffe0c3cd47fab9aafdec4",
    "artifacts/demo_v5/region_transition_A.rpl": "b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3",
    "artifacts/demo_v5/region_transition_B.rpl": "5847c400779e7867f71b56b2494097c00c3c7bbbc0d5bcd3bb3141980497f5a8",
}


def run(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=REPO_ROOT, text=True, capture_output=True, check=False)


def require_rc(proc: subprocess.CompletedProcess[str], rc: int, name: str) -> None:
    if proc.returncode != rc:
        raise AssertionError(
            f"{name}: expected rc={rc}, got rc={proc.returncode}\nstdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )


def require_contains(proc: subprocess.CompletedProcess[str], parts: list[str], name: str) -> None:
    for part in parts:
        if part not in proc.stdout:
            raise AssertionError(
                f"{name}: missing output '{part}'\nstdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
            )


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    fixture_proc = run(FIXTURE_GEN)
    require_rc(fixture_proc, 0, "fixture_gen")
    require_contains(fixture_proc, ["PASS: demo-v5 fixtures generated"], "fixture_gen")

    for rel_path, expected_sha in EXPECTED_SHA256.items():
        actual_sha = sha256_file(REPO_ROOT / rel_path)
        if actual_sha != expected_sha:
            raise AssertionError(
                f"hash mismatch for {rel_path}: expected {expected_sha}, got {actual_sha}"
            )

    checks = [
        (
            "self_healing",
            "artifacts/demo_v5/self_healing_A.rpl",
            "artifacts/demo_v5/self_healing_B.rpl",
            [
                "First divergence frame: 4096",
                "Classification: transient",
                "first_divergence_frame: 4096",
                "shape_class: transient",
                "primary_region: sample_payload",
                "all_regions_at_first_divergence: [sample_payload]",
                "region_summary: sample_payload",
                "evolution_class: self_healing",
                "timeline_summary: divergence resolves within 1 frame",
            ],
        ),
        (
            "bounded_persistent",
            "artifacts/demo_v5/bounded_persistent_A.rpl",
            "artifacts/demo_v5/bounded_persistent_B.rpl",
            [
                "First divergence frame: 4096",
                "Classification: persistent_offset",
                "shape_class: persistent_offset",
                "evolution_class: bounded_persistent",
                "timeline_summary: divergence remains in sample_payload through final frame",
            ],
        ),
        (
            "monotonic_growth",
            "artifacts/demo_v5/monotonic_growth_A.rpl",
            "artifacts/demo_v5/monotonic_growth_B.rpl",
            [
                "First divergence frame: 4096",
                "Classification: rate_divergence",
                "shape_class: rate_divergence",
                "evolution_class: monotonic_growth",
                "timeline_summary: sample_payload magnitude grows from 1 to 5904 by frame 9999",
            ],
        ),
        (
            "region_transition",
            "artifacts/demo_v5/region_transition_A.rpl",
            "artifacts/demo_v5/region_transition_B.rpl",
            [
                "First divergence frame: 4096",
                "Classification: none",
                "shape_class: none",
                "primary_region: timer_delta",
                "all_regions_at_first_divergence: [timer_delta]",
                "region_summary: timer_delta",
                "evolution_class: region_transition",
                "timeline_summary: divergence reaches sample_payload at frame 4098",
            ],
        ),
        (
            "no_divergence",
            "artifacts/demo_v5/self_healing_A.rpl",
            "artifacts/demo_v5/self_healing_A.rpl",
            [
                "NO DIVERGENCE DETECTED",
                "first_divergence_frame: none",
                "shape_class: none",
                "primary_region: none",
                "region_summary: none",
                "evolution_class: none",
                "timeline_summary: none",
            ],
        ),
    ]

    for name, artifact_a, artifact_b, expected_lines in checks:
        proc = run(DIFF + [artifact_a, artifact_b])
        require_rc(proc, 0, name)
        require_contains(proc, expected_lines, name)

    print("PASS: Demo V5 evolution regression suite")
    return 0


if __name__ == "__main__":
    sys.exit(main())
