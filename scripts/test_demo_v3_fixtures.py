#!/usr/bin/env python3
"""Deterministic hash assertions for Demo V3 fixture artifacts."""

import hashlib
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
FIXTURE_GEN = ["python3", "scripts/generate_demo_v3_fixtures.py", "--out-dir", "artifacts/demo_v3"]
EXPECTED_SHA256 = {
    "artifacts/demo_v3/transient_A.rpl": "b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3",
    "artifacts/demo_v3/transient_B.rpl": "ca846f863ddf055895ec23a50bf6e5c7f5a792461ad539dd4191bdf80b6b1006",
    "artifacts/demo_v3/offset_A.rpl": "b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3",
    "artifacts/demo_v3/offset_B.rpl": "b0f93d0c2e4a674c30c358f799d0b798a7d4989220477583c53d37f08fa48256",
    "artifacts/demo_v3/rate_A.rpl": "b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3",
    "artifacts/demo_v3/rate_B.rpl": "753368ffcb755091cde253b5b8d54d20bd578961e2d8040b9a6fec34eb0bdedf",
}


def run(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=REPO_ROOT, text=True, capture_output=True, check=False)


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    proc = run(FIXTURE_GEN)
    if proc.returncode != 0:
        raise AssertionError(
            "fixture_gen: expected success\n"
            f"stdout:\n{proc.stdout}\n"
            f"stderr:\n{proc.stderr}"
        )

    for rel_path, expected_sha in EXPECTED_SHA256.items():
        path = REPO_ROOT / rel_path
        actual_sha = sha256_file(path)
        if actual_sha != expected_sha:
            raise AssertionError(
                f"hash mismatch for {rel_path}: "
                f"expected {expected_sha}, got {actual_sha}"
            )

    print("PASS: Demo V3 fixture hashes match expected values")
    return 0


if __name__ == "__main__":
    sys.exit(main())
