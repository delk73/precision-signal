#!/usr/bin/env python3
"""Deterministic regression tests for Demo V4 region attribution."""

from __future__ import annotations

import hashlib
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
FIXTURE_GEN = ["python3", "scripts/generate_demo_v4_fixtures.py", "--out-dir", "artifacts/demo_v4"]
DIFF = ["python3", "scripts/artifact_diff.py"]
EXPECTED_SHA256 = {
    "artifacts/demo_v4/header_schema_A.rpl": "b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3",
    "artifacts/demo_v4/header_schema_B.rpl": "d9f2e7cd55f77d7ad53c69de7d8248ef1fb81aa3c9c8d226ae4e7cad745f0251",
    "artifacts/demo_v4/header_schema_sample_payload_A.rpl": "b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3",
    "artifacts/demo_v4/header_schema_sample_payload_B.rpl": "70e69d3923ead927b2feb77885650117a525ec6760d1a141ca734dceff8ba839",
    "artifacts/demo_v4/timer_delta_A.rpl": "b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3",
    "artifacts/demo_v4/timer_delta_B.rpl": "ad2041a65ca78fb86ad46db095e613198a11710ad27c08eedcd7b9573555565b",
    "artifacts/demo_v4/irq_state_A.rpl": "b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3",
    "artifacts/demo_v4/irq_state_B.rpl": "2767848e930ce03abb9e242d3558d3b75f8f1d001037808549aa6c47c8fd2ef7",
    "artifacts/demo_v4/sample_payload_A.rpl": "b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3",
    "artifacts/demo_v4/sample_payload_B.rpl": "b0f93d0c2e4a674c30c358f799d0b798a7d4989220477583c53d37f08fa48256",
    "artifacts/demo_v4/mixed_A.rpl": "b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3",
    "artifacts/demo_v4/mixed_B.rpl": "06f4f7063be7215c8b03346313b62a8765b4b5b2c472009f0e54148bf77465d4",
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
    require_contains(fixture_proc, ["PASS: demo-v4 fixtures generated"], "fixture_gen")

    for rel_path, expected_sha in EXPECTED_SHA256.items():
        actual_sha = sha256_file(REPO_ROOT / rel_path)
        if actual_sha != expected_sha:
            raise AssertionError(
                f"hash mismatch for {rel_path}: expected {expected_sha}, got {actual_sha}"
            )

    checks = [
        (
            "header_schema",
            "artifacts/demo_v4/header_schema_A.rpl",
            "artifacts/demo_v4/header_schema_B.rpl",
            [
                "First divergence frame: 0",
                "Classification: none",
                "first_divergence_frame: 0",
                "shape_class: none",
                "primary_region: header_schema",
                "all_regions_at_first_divergence: [header_schema]",
                "region_summary: header_schema",
            ],
        ),
        (
            "timer_delta",
            "artifacts/demo_v4/timer_delta_A.rpl",
            "artifacts/demo_v4/timer_delta_B.rpl",
            [
                "First divergence frame: 4096",
                "Classification: none",
                "first_divergence_frame: 4096",
                "shape_class: none",
                "primary_region: timer_delta",
                "all_regions_at_first_divergence: [timer_delta]",
                "region_summary: timer_delta",
            ],
        ),
        (
            "header_schema_sample_payload",
            "artifacts/demo_v4/header_schema_sample_payload_A.rpl",
            "artifacts/demo_v4/header_schema_sample_payload_B.rpl",
            [
                "First divergence frame: 0",
                "Classification: persistent_offset",
                "first_divergence_frame: 0",
                "shape_class: persistent_offset",
                "primary_region: header_schema",
                "all_regions_at_first_divergence: [header_schema, sample_payload]",
                "region_summary: mixed",
            ],
        ),
        (
            "irq_state",
            "artifacts/demo_v4/irq_state_A.rpl",
            "artifacts/demo_v4/irq_state_B.rpl",
            [
                "First divergence frame: 4096",
                "Classification: none",
                "first_divergence_frame: 4096",
                "shape_class: none",
                "primary_region: irq_state",
                "all_regions_at_first_divergence: [irq_state]",
                "region_summary: irq_state",
            ],
        ),
        (
            "sample_payload",
            "artifacts/demo_v4/sample_payload_A.rpl",
            "artifacts/demo_v4/sample_payload_B.rpl",
            [
                "First divergence frame: 4096",
                "Classification: persistent_offset",
                "first_divergence_frame: 4096",
                "shape_class: persistent_offset",
                "primary_region: sample_payload",
                "all_regions_at_first_divergence: [sample_payload]",
                "region_summary: sample_payload",
            ],
        ),
        (
            "mixed",
            "artifacts/demo_v4/mixed_A.rpl",
            "artifacts/demo_v4/mixed_B.rpl",
            [
                "First divergence frame: 4096",
                "Classification: persistent_offset",
                "first_divergence_frame: 4096",
                "shape_class: persistent_offset",
                "primary_region: timer_delta",
                "all_regions_at_first_divergence: [timer_delta, sample_payload]",
                "region_summary: mixed",
            ],
        ),
        (
            "null",
            "artifacts/demo_v4/sample_payload_A.rpl",
            "artifacts/demo_v4/sample_payload_A.rpl",
            [
                "NO DIVERGENCE: artifacts identical",
                "first_divergence_frame: none",
                "shape_class: none",
                "primary_region: none",
                "all_regions_at_first_divergence: []",
                "region_summary: none",
            ],
        ),
    ]

    for name, artifact_a, artifact_b, expected_lines in checks:
        proc = run(DIFF + [artifact_a, artifact_b])
        require_rc(proc, 0, name)
        require_contains(proc, expected_lines, name)

    print("PASS: Demo V4 region attribution regression suite")
    return 0


if __name__ == "__main__":
    sys.exit(main())
