#!/usr/bin/env python3
"""Validate retained release-bundle coherence."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


RUN_ID_RE = re.compile(r"run_\d{8}T\d{6}Z")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")


def load_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def parse_manifest(path: Path) -> dict[str, str]:
    manifest: dict[str, str] = {}
    for raw_line in load_text(path).splitlines():
        line = raw_line.strip()
        if not line or "=" not in line:
            continue
        key, value = line.split("=", 1)
        manifest[key] = value
    return manifest


def parse_sha256_summary(path: Path) -> tuple[str, list[tuple[str, str]]]:
    lines = [line.strip() for line in load_text(path).splitlines() if line.strip()]
    if not lines:
        raise ValueError("sha256_summary.txt is empty")
    baseline_parts = lines[0].split()
    if len(baseline_parts) != 3 or baseline_parts[0] != "baseline_sha256" or baseline_parts[2] != "baseline":
        raise ValueError("sha256_summary.txt baseline line must be: baseline_sha256 <hash> baseline")
    baseline_sha = baseline_parts[1]
    if not SHA256_RE.fullmatch(baseline_sha):
        raise ValueError("sha256_summary.txt baseline hash is not a valid sha256")
    runs: list[tuple[str, str]] = []
    for line in lines[1:]:
        parts = line.split()
        if len(parts) != 2:
            raise ValueError(f"sha256_summary.txt run line has unexpected format: {line}")
        sha, name = parts
        if not SHA256_RE.fullmatch(sha):
            raise ValueError(f"sha256_summary.txt run hash is not a valid sha256: {line}")
        runs.append((sha, name))
    return baseline_sha, runs


def parse_hash_check(path: Path) -> list[tuple[str, str]]:
    entries: list[tuple[str, str]] = []
    for raw_line in load_text(path).splitlines():
        line = raw_line.strip()
        if not line:
            continue
        parts = line.split()
        if len(parts) != 2:
            raise ValueError(f"hash_check.txt line has unexpected format: {line}")
        sha, rel_path = parts
        if not SHA256_RE.fullmatch(sha):
            raise ValueError(f"hash_check.txt hash is not a valid sha256: {line}")
        entries.append((sha, rel_path))
    return entries


def validate_bundle(bundle_dir: Path, repo_root: Path) -> list[str]:
    required = {
        "firmware_release_evidence.md": bundle_dir / "firmware_release_evidence.md",
        "replay_manifest_v1.txt": bundle_dir / "replay_manifest_v1.txt",
        "sha256_summary.txt": bundle_dir / "sha256_summary.txt",
        "hash_check.txt": bundle_dir / "hash_check.txt",
    }
    errors: list[str] = []
    for name, path in required.items():
        if not path.is_file():
            errors.append(f"missing required retained file: {name}")
    if errors:
        return errors

    evidence_text = load_text(required["firmware_release_evidence.md"])
    manifest_text = load_text(required["replay_manifest_v1.txt"])
    hash_check_text = load_text(required["hash_check.txt"])

    run_ids = set(RUN_ID_RE.findall(evidence_text))
    run_ids.update(RUN_ID_RE.findall(manifest_text))
    run_ids.update(RUN_ID_RE.findall(hash_check_text))
    if len(run_ids) != 1:
        errors.append(f"expected exactly one run id across retained files, found: {sorted(run_ids)}")
        return errors
    run_id = next(iter(run_ids))

    manifest = parse_manifest(required["replay_manifest_v1.txt"])
    for key in ("baseline_path", "baseline_sha256", "completed_runs", "timestamp_utc", "run_dir"):
        if key not in manifest:
            errors.append(f"replay_manifest_v1.txt missing required key: {key}")
    if errors:
        return errors

    baseline_sha, summary_runs = parse_sha256_summary(required["sha256_summary.txt"])
    hash_entries = parse_hash_check(required["hash_check.txt"])
    hash_map = {path: sha for sha, path in hash_entries}
    expected_baseline_path = manifest["baseline_path"]
    baseline_path = repo_root / expected_baseline_path
    run_dir = Path(manifest["run_dir"])

    if run_id not in manifest["run_dir"]:
        errors.append("replay_manifest_v1.txt run_dir does not match retained run id")
    if not run_dir.is_dir():
        errors.append(f"manifest run_dir does not exist: {run_dir}")
    if not baseline_path.is_file():
        errors.append(f"manifest baseline_path does not exist: {baseline_path}")

    manifest_baseline_sha = manifest["baseline_sha256"]
    if baseline_sha != manifest_baseline_sha:
        errors.append("baseline sha mismatch between sha256_summary.txt and replay_manifest_v1.txt")
    if hash_map.get(expected_baseline_path) != manifest_baseline_sha:
        errors.append("baseline sha mismatch between hash_check.txt and replay_manifest_v1.txt")
    if hash_map.get("artifacts/run.bin") != manifest_baseline_sha:
        errors.append("artifacts/run.bin hash does not match retained baseline sha")

    completed_runs = int(manifest["completed_runs"])
    if completed_runs != len(summary_runs):
        errors.append("completed_runs does not match sha256_summary.txt run count")

    run_entries = [(sha, rel_path) for sha, rel_path in hash_entries if rel_path.startswith("artifacts/replay_runs/")]
    if len(run_entries) != completed_runs:
        errors.append("hash_check.txt run count does not match replay_manifest_v1.txt completed_runs")

    expected_run_paths = [f"artifacts/replay_runs/{run_id}/run_{idx:02d}.bin" for idx in range(1, completed_runs + 1)]
    actual_run_paths = [rel_path for _, rel_path in run_entries]
    if actual_run_paths != expected_run_paths:
        errors.append(f"hash_check.txt run paths do not match retained run id/path set: {actual_run_paths}")

    summary_map = {name: sha for sha, name in summary_runs}
    if len(summary_map) != len(summary_runs):
        errors.append("sha256_summary.txt contains duplicate run filenames")
    for expected_path in expected_run_paths:
        repo_path = repo_root / expected_path
        if not repo_path.is_file():
            errors.append(f"referenced run artifact does not exist: {repo_path}")
            continue
        expected_name = repo_path.name
        if expected_name not in summary_map:
            errors.append(f"sha256_summary.txt missing retained run entry: {expected_name}")
            continue
        summary_sha = summary_map[expected_name]
        hash_sha = hash_map.get(expected_path)
        if hash_sha != summary_sha:
            errors.append(f"hash mismatch for {expected_name} between sha256_summary.txt and hash_check.txt")
        if summary_sha != manifest_baseline_sha:
            errors.append(f"retained run hash diverges from baseline sha for {expected_name}")

    evidence_run_line = re.search(r"^RUN_ID=(.+)$", evidence_text, re.MULTILINE)
    if evidence_run_line and evidence_run_line.group(1).strip() != run_id:
        errors.append("firmware_release_evidence.md RUN_ID does not match retained run id")
    evidence_timestamp = re.search(r"^TIMESTAMP_UTC=(.+)$", evidence_text, re.MULTILINE)
    if evidence_timestamp and evidence_timestamp.group(1).strip() != manifest["timestamp_utc"]:
        errors.append("firmware_release_evidence.md TIMESTAMP_UTC does not match replay_manifest_v1.txt")
    evidence_run_dir = re.search(r"^RUN_DIR=(.+)$", evidence_text, re.MULTILINE)
    if evidence_run_dir:
        evidence_run_dir_value = evidence_run_dir.group(1).strip()
        allowed_run_dirs = {
            f"artifacts/replay_runs/{run_id}",
            str(run_dir),
        }
        if evidence_run_dir_value not in allowed_run_dirs:
            errors.append("firmware_release_evidence.md RUN_DIR does not match retained run path")

    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate retained release-bundle coherence.")
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parent.parent,
        help="repository root",
    )
    parser.add_argument(
        "--version",
        help="release version under docs/verification/releases/",
    )
    parser.add_argument(
        "--bundle-dir",
        type=Path,
        help="explicit retained bundle directory",
    )
    args = parser.parse_args()

    repo_root = args.root.resolve()
    if args.bundle_dir is not None:
        bundle_dir = args.bundle_dir.resolve()
    elif args.version is not None:
        bundle_dir = (repo_root / "docs" / "verification" / "releases" / args.version).resolve()
    else:
        parser.error("pass --version or --bundle-dir")

    try:
        errors = validate_bundle(bundle_dir, repo_root)
    except ValueError as exc:
        print(f"FAIL: retained release bundle coherence: {bundle_dir.relative_to(repo_root)}")
        print(f"- {exc}")
        return 1
    if errors:
        print(f"FAIL: retained release bundle coherence: {bundle_dir.relative_to(repo_root)}")
        for error in errors:
            print(f"- {error}")
        return 1

    print(f"PASS: retained release bundle coherence: {bundle_dir.relative_to(repo_root)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
