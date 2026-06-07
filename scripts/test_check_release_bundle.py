#!/usr/bin/env python3
"""Regression tests for retained release-bundle authority coherence."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
CHECKER = REPO_ROOT / "scripts" / "check_release_bundle.py"
AUTHORITY_FILES = (
    "index.md",
    "summary.md",
    "summary.json",
    "fw_capture.bin",
    "rpl0_witness_fw_capture.txt",
)
HASHED_AUTHORITY_FILES = (
    "fw_capture.bin",
    "rpl0_witness_fw_capture.txt",
    "index.md",
    "summary.md",
)
INDEX_NAMES = (
    "fw_capture.bin",
    "rpl0_witness_fw_capture.txt",
    "summary.md",
    "summary.json",
)
SUMMARY_COMMANDS = (
    "make gate",
    "make authoritative-replay-cli-tests",
    "make replay-witness-check VERSION=2.0.0",
    "make release-bundle-check VERSION=2.0.0",
)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def run_check(bundle_dir: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(CHECKER), "--bundle-dir", str(bundle_dir)],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )


def run_make_191() -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["make", "release-bundle-check", "VERSION=1.9.1"],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )


def write_summary_json(bundle_dir: Path, version: str, *, omit: str | None = None, stale: str | None = None) -> None:
    hashes = {
        path.relative_to(bundle_dir).as_posix(): sha256_file(path)
        for path in bundle_dir.rglob("*")
        if path.is_file() and path.name != "summary.json"
    }
    if omit is not None:
        hashes.pop(omit, None)
    if stale is not None:
        hashes[stale] = "0" * 64
    summary = {
        "schema": "precision.release_summary.v1",
        "version": version,
        "hashes": hashes,
    }
    (bundle_dir / "summary.json").write_text(json.dumps(summary, indent=2, sort_keys=True), encoding="utf-8")


def make_fixture(root: Path, *, witness_text: str = "not a semantic witness") -> Path:
    version = "2.0.0"
    bundle_dir = root / version
    bundle_dir.mkdir()
    (bundle_dir / "fw_capture.bin").write_bytes(b"synthetic firmware capture\n")
    (bundle_dir / "rpl0_witness_fw_capture.txt").write_text(witness_text, encoding="utf-8")
    (bundle_dir / "index.md").write_text(
        "# Release Evidence Bundle (2.0.0)\n\n"
        "- fw_capture.bin\n"
        "- rpl0_witness_fw_capture.txt\n"
        "- summary.md\n"
        "- summary.json\n",
        encoding="utf-8",
    )
    (bundle_dir / "summary.md").write_text(
        "# Release Bundle Summary (2.0.0)\n\n"
        "## Validation Commands\n"
        "make gate\n"
        "make authoritative-replay-cli-tests\n"
        "make replay-witness-check VERSION=2.0.0\n"
        "make release-bundle-check VERSION=2.0.0\n\n"
        "## Hashes\n",
        encoding="utf-8",
    )
    write_summary_json(bundle_dir, version)
    return bundle_dir


def assert_ok(name: str, proc: subprocess.CompletedProcess[str]) -> None:
    if proc.returncode != 0:
        raise AssertionError(
            f"{name}: expected success, rc={proc.returncode}\n"
            f"stdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )


def assert_fail(name: str, proc: subprocess.CompletedProcess[str], needle: str) -> None:
    if proc.returncode == 0:
        raise AssertionError(f"{name}: expected failure")
    if needle not in proc.stdout:
        raise AssertionError(
            f"{name}: missing failure marker {needle!r}\nstdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )


def case_root(root: Path, name: str) -> Path:
    path = root / name
    path.mkdir()
    return path


def check_valid_fixture(root: Path) -> None:
    bundle_dir = make_fixture(case_root(root, "valid"))
    assert_ok("valid_2_0_authority_bundle", run_check(bundle_dir))


def check_missing_required_files(root: Path) -> None:
    for rel_path in AUTHORITY_FILES:
        bundle_dir = make_fixture(case_root(root, f"missing_{rel_path.replace('.', '_')}"))
        (bundle_dir / rel_path).unlink()
        assert_fail(
            f"missing_{rel_path}",
            run_check(bundle_dir),
            f"missing required retained authority file: {rel_path}",
        )


def check_missing_hashes(root: Path) -> None:
    for rel_path in HASHED_AUTHORITY_FILES:
        bundle_dir = make_fixture(case_root(root, f"missing_hash_{rel_path.replace('.', '_')}"))
        write_summary_json(bundle_dir, "2.0.0", omit=rel_path)
        assert_fail(
            f"missing_hash_{rel_path}",
            run_check(bundle_dir),
            "summary.json missing hash entries",
        )


def check_stale_hashes(root: Path) -> None:
    for rel_path in HASHED_AUTHORITY_FILES:
        bundle_dir = make_fixture(case_root(root, f"stale_hash_{rel_path.replace('.', '_')}"))
        write_summary_json(bundle_dir, "2.0.0", stale=rel_path)
        assert_fail(
            f"stale_hash_{rel_path}",
            run_check(bundle_dir),
            f"summary.json hash mismatch for {rel_path}",
        )


def check_summary_json_self_hash_rejected(root: Path) -> None:
    bundle_dir = make_fixture(case_root(root, "summary_json_self_hash"))
    summary_path = bundle_dir / "summary.json"
    summary = json.loads(summary_path.read_text(encoding="utf-8"))
    summary["hashes"]["summary.json"] = "0" * 64
    summary_path.write_text(json.dumps(summary, indent=2, sort_keys=True), encoding="utf-8")
    assert_fail(
        "summary_json_self_hash",
        run_check(bundle_dir),
        "summary.json has stale hash entries",
    )


def check_missing_index_routes(root: Path) -> None:
    for name in INDEX_NAMES:
        bundle_dir = make_fixture(case_root(root, f"missing_index_{name.replace('.', '_')}"))
        index_path = bundle_dir / "index.md"
        index_path.write_text(index_path.read_text(encoding="utf-8").replace(name, "omitted"), encoding="utf-8")
        write_summary_json(bundle_dir, "2.0.0")
        assert_fail(
            f"missing_index_{name}",
            run_check(bundle_dir),
            f"index.md missing retained authority route/name: {name}",
        )


def check_missing_summary_commands(root: Path) -> None:
    for idx, command in enumerate(SUMMARY_COMMANDS):
        bundle_dir = make_fixture(case_root(root, f"missing_command_{idx}"))
        summary_path = bundle_dir / "summary.md"
        summary_path.write_text(summary_path.read_text(encoding="utf-8").replace(command, "omitted"), encoding="utf-8")
        write_summary_json(bundle_dir, "2.0.0")
        assert_fail(
            f"missing_summary_command_{command}",
            run_check(bundle_dir),
            f"summary.md missing authority validation command: {command}",
        )


def check_no_semantic_witness_validation(root: Path) -> None:
    bundle_dir = make_fixture(
        case_root(root, "bad_witness_semantics"),
        witness_text="RESULT=FAIL\nWITNESS_DIGEST=this-is-not-a-real-digest\n",
    )
    assert_ok("semantically_bad_witness_still_bundle_coherent", run_check(bundle_dir))


def main() -> int:
    assert_ok("existing_1_9_1_release_bundle_check", run_make_191())
    with tempfile.TemporaryDirectory(prefix="precision_release_bundle_") as tmp:
        root = Path(tmp)
        check_valid_fixture(root)
        check_missing_required_files(root)
        check_missing_hashes(root)
        check_stale_hashes(root)
        check_summary_json_self_hash_rejected(root)
        check_missing_index_routes(root)
        check_missing_summary_commands(root)
        check_no_semantic_witness_validation(root)
    print("PASS: release bundle authority coherence regression suite")
    return 0


if __name__ == "__main__":
    sys.exit(main())
