#!/usr/bin/env python3
"""Run the retained replay demo ladder from one supported entrypoint."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent


def env() -> dict[str, str]:
    merged = os.environ.copy()
    existing = merged.get("PYTHONPATH")
    merged["PYTHONPATH"] = "." if not existing else f".{os.pathsep}{existing}"
    return merged


def run(label: str, args: list[str], *, expected_rc: int = 0) -> None:
    print(f"== {label} ==")
    result = subprocess.run(args, cwd=REPO_ROOT, env=env(), check=False)
    if result.returncode != expected_rc:
        print(
            f"FAIL: {label}: expected rc={expected_rc}, got rc={result.returncode}",
            file=sys.stderr,
        )
        raise SystemExit(result.returncode or 1)


def require_file(path: str) -> None:
    full_path = REPO_ROOT / path
    if not full_path.is_file() or full_path.stat().st_size == 0:
        print(f"FAIL: missing or empty {path}", file=sys.stderr)
        raise SystemExit(1)


def require_distinct(path_a: str, path_b: str) -> None:
    full_a = REPO_ROOT / path_a
    full_b = REPO_ROOT / path_b
    if full_a.read_bytes() == full_b.read_bytes():
        print(f"FAIL: expected distinct artifacts: {path_a} {path_b}", file=sys.stderr)
        raise SystemExit(1)


def main() -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(line_buffering=True)

    python = sys.executable

    print("Replay demo surface classification:")
    print("active: demo-evidence-package, demo-divergence, replay-demo-audit")
    print("support: demo-captured-verify, demo-captured-release")
    print("historical: Demo V2-V5 lifecycle targets; retained as fixtures/scripts")
    print()

    v2_a = "artifacts/demo_persistent/run_A.rpl"
    v2_b = "artifacts/demo_persistent/run_B.rpl"
    require_file(v2_a)
    require_file(v2_b)
    run("Demo V2 fixture diff", [python, "scripts/artifact_diff.py", v2_a, v2_b])
    run(
        "Demo V2 canonical fixture verifies",
        [python, "scripts/artifact_tool.py", "verify", v2_a, "--signal-model", "phase8"],
    )
    run(
        "Demo V2 persistent fixture fails phase8 as expected",
        [python, "scripts/artifact_tool.py", "verify", v2_b, "--signal-model", "phase8"],
        expected_rc=1,
    )
    run(
        "Demo V2 persistent fixture validates structurally",
        [python, "scripts/artifact_tool.py", "verify", v2_b, "--signal-model", "none"],
    )

    run("Demo V3 fixture hash regression", [python, "scripts/test_demo_v3_fixtures.py"])
    run("Demo V4 region attribution regression", [python, "tests/test_demo_v4_region_attribution.py"])
    run("Demo V5 evolution regression", [python, "tests/test_demo_v5_evolution.py"])

    captured_a = "artifacts/demo_captured/run_A.rpl"
    captured_b = "artifacts/demo_captured/run_B.rpl"
    require_file(captured_a)
    require_file(captured_b)
    require_distinct(captured_a, captured_b)
    run("Captured divergence diff", [python, "scripts/artifact_diff.py", captured_a, captured_b])

    print("PASS: replay demo audit")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
