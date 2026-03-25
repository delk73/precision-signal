#!/usr/bin/env python3
"""Regression tests for documentation link integrity enforcement."""

from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent


def run_check(root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["python3", "scripts/check_doc_links.py", "--root", str(root)],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )


def write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def assert_ok(name: str, proc: subprocess.CompletedProcess[str]) -> None:
    if proc.returncode != 0:
        raise AssertionError(
            f"{name}: expected success, rc={proc.returncode}\nstdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )


def assert_fail(name: str, proc: subprocess.CompletedProcess[str], needle: str) -> None:
    if proc.returncode == 0:
        raise AssertionError(f"{name}: expected failure")
    if needle not in proc.stdout:
        raise AssertionError(f"{name}: missing failure marker {needle!r}\nstdout:\n{proc.stdout}")


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="dpw_doc_links_") as tmp:
        root = Path(tmp)
        write(root / "README.md", "See [docs/guide.md](docs/guide.md).\n")
        write(root / "docs" / "guide.md", "Use [VERIFICATION_GUIDE.md](../VERIFICATION_GUIDE.md).\n")
        write(root / "VERIFICATION_GUIDE.md", "# Guide\n")
        proc = run_check(root)
        assert_ok("valid_links_pass", proc)

    with tempfile.TemporaryDirectory(prefix="dpw_doc_links_") as tmp:
        root = Path(tmp)
        write(root / "README.md", "See [missing](docs/missing.md).\n")
        write(root / "docs" / "guide.md", "# Guide\n")
        proc = run_check(root)
        assert_fail("broken_target_fails", proc, "broken_target")

    with tempfile.TemporaryDirectory(prefix="dpw_doc_links_") as tmp:
        root = Path(tmp)
        write(root / "README.md", "# Root\n")
        write(root / "VERIFICATION_GUIDE.md", "# Guide\n")
        write(
            root / "docs" / "guide.md",
            "See [`../../VERIFICATION_GUIDE.md`](../VERIFICATION_GUIDE.md).\n",
        )
        proc = run_check(root)
        assert_fail("bad_label_fails", proc, "bad_label")

    with tempfile.TemporaryDirectory(prefix="dpw_doc_links_") as tmp:
        root = Path(tmp)
        write(root / "README.md", "See `docs/guide.md` for details.\n")
        write(root / "docs" / "guide.md", "# Guide\n")
        proc = run_check(root)
        assert_fail("non_clickable_navigation_fails", proc, "non_clickable_navigation")

    with tempfile.TemporaryDirectory(prefix="dpw_doc_links_") as tmp:
        root = Path(tmp)
        write(root / "README.md", "```md\nSee `docs/missing.md`.\n```\n")
        write(root / "docs" / "guide.md", "# Guide\n")
        proc = run_check(root)
        assert_ok("code_fence_ignored", proc)

    with tempfile.TemporaryDirectory(prefix="dpw_doc_links_") as tmp:
        root = Path(tmp)
        write(root / "README.md", "See crates/dpw4/src/ for implementation details.\n")
        write(root / "docs" / "guide.md", "# Guide\n")
        proc = run_check(root)
        assert_ok("non_doc_paths_ignored", proc)

    print("PASS: documentation link integrity regression suite")
    return 0


if __name__ == "__main__":
    sys.exit(main())