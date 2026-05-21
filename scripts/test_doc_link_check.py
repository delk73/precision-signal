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
        write(root / "README.md", "Literal path example: `docs/guide.md`.\n")
        write(root / "docs" / "guide.md", "# Guide\n")
        proc = run_check(root)
        assert_ok("quoted_non_navigation_path_ignored", proc)

    with tempfile.TemporaryDirectory(prefix="dpw_doc_links_") as tmp:
        root = Path(tmp)
        write(root / "README.md", "See crates/dpw4/src/ for implementation details.\n")
        write(root / "docs" / "guide.md", "# Guide\n")
        proc = run_check(root)
        assert_ok("non_doc_paths_ignored", proc)

    with tempfile.TemporaryDirectory(prefix="dpw_doc_links_") as tmp:
        root = Path(tmp)
        write(root / "README.md", "See [releases](docs/verification/releases/index.md).\n")
        write(root / "docs" / "VERIFICATION_GUIDE.md", "# Guide\n")
        write(
            root / "docs" / "verification" / "releases" / "index.md",
            "Use [VERIFICATION_GUIDE.md](../../../VERIFICATION_GUIDE.md).\n",
        )
        proc = run_check(root)
        assert_fail("release_index_broken_target_fails", proc, "broken_target")

    with tempfile.TemporaryDirectory(prefix="dpw_doc_links_") as tmp:
        root = Path(tmp)
        write(root / "README.md", "See [docs](docs/DOCS_INDEX.md).\n")
        write(
            root / "docs" / "DOCS_INDEX.md",
            "- Active retained release summary routes:\n"
            "- Active retained release summary route:\n"
            "  [docs/verification/releases/1.7.0/index.md](verification/releases/1.7.0/index.md)\n",
        )
        write(root / "docs" / "verification" / "releases" / "1.7.0" / "index.md", "# 1.7.0\n")
        proc = run_check(root)
        assert_fail("dangling_bullet_fails", proc, "dangling_bullet")

    with tempfile.TemporaryDirectory(prefix="dpw_doc_links_") as tmp:
        root = Path(tmp)
        write(root / "README.md", "See [docs](docs/DOCS_INDEX.md) and [demos](docs/demos/demo.md).\n")
        write(root / "docs" / "DOCS_INDEX.md", "See [demos](demos/demo.md).\n")
        write(root / "docs" / "demos" / "demo.md", "# Demo Landing\n")
        proc = run_check(root)
        assert_fail("parallel_reader_path_fails", proc, "parallel_reader_path")

    with tempfile.TemporaryDirectory(prefix="dpw_doc_links_") as tmp:
        root = Path(tmp)
        write(
            root / "README.md",
            "See [docs](docs/DOCS_INDEX.md) and [guide](docs/VERIFICATION_GUIDE.md).\n",
        )
        write(root / "docs" / "DOCS_INDEX.md", "See [guide](VERIFICATION_GUIDE.md).\n")
        write(root / "docs" / "VERIFICATION_GUIDE.md", "# Guide\n")
        proc = run_check(root)
        assert_ok("parallel_authority_links_pass", proc)

    with tempfile.TemporaryDirectory(prefix="dpw_doc_links_") as tmp:
        root = Path(tmp)
        write(root / "README.md", "See [docs](docs/DOCS_INDEX.md).\n")
        write(root / "docs" / "DOCS_INDEX.md", "See [audits](audits/AUDIT_INDEX.md).\n")
        write(root / "docs" / "audits" / "AUDIT_INDEX.md", "Use [missing](missing.md).\n")
        write(root / "docs" / "audits" / "runlogs" / "payload.md", "Use [missing](missing.md).\n")
        proc = run_check(root)
        assert_fail("linked_audit_index_checked", proc, "docs/audits/AUDIT_INDEX.md")

    with tempfile.TemporaryDirectory(prefix="dpw_doc_links_") as tmp:
        root = Path(tmp)
        write(root / "README.md", "See [docs](docs/DOCS_INDEX.md).\n")
        write(root / "docs" / "DOCS_INDEX.md", "See [wip](wip/WIP_INDEX.md).\n")
        write(root / "docs" / "wip" / "WIP_INDEX.md", "Use [missing](missing.md).\n")
        write(root / "docs" / "wip" / "scratch.md", "Use [missing](missing.md).\n")
        proc = run_check(root)
        assert_fail("linked_wip_index_checked", proc, "docs/wip/WIP_INDEX.md")

    with tempfile.TemporaryDirectory(prefix="dpw_doc_links_") as tmp:
        root = Path(tmp)
        write(root / "README.md", "See [docs](docs/DOCS_INDEX.md).\n")
        write(root / "docs" / "DOCS_INDEX.md", "See [audits](audits/AUDIT_INDEX.md).\n")
        write(root / "docs" / "audits" / "AUDIT_INDEX.md", "Audit index.\n")
        write(root / "docs" / "audits" / "runlogs" / "payload.md", "Use [missing](missing.md).\n")
        write(root / "docs" / "archive" / "historical.md", "Use [missing](missing.md).\n")
        proc = run_check(root)
        assert_ok("ignored_payload_dirs_not_broadly_checked", proc)

    print("PASS: documentation link integrity regression suite")
    return 0


if __name__ == "__main__":
    sys.exit(main())
