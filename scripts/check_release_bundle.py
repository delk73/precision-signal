#!/usr/bin/env python3
"""Validate retained release-bundle coherence."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path


RUN_ID_RE = re.compile(r"run_\d{8}T\d{6}Z")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
RUN_DIR_REL_RE = re.compile(r"^artifacts/replay_runs/(run_\d{8}T\d{6}Z)$")

NON_FIRMWARE_REQUIRED_FILES = (
    "index.md",
    "cargo_check_dpw4_thumb_locked.txt",
    "kani_evidence.txt",
    "make_demo_evidence_package.txt",
    "make_doc_link_check.txt",
    "make_gate.txt",
    "make_release_bundle_check.txt",
    "make_replay_tests.txt",
    "release_reproducibility.txt",
)

FIRMWARE_REQUIRED_FILE_SETS = {
    "rpl0_archive_v1": (
        "firmware_release_evidence.md",
        "fw_capture.bin",
        "fw_capture_hash_check.txt",
        "fw_repeat/replay_manifest_v1.txt",
        "fw_repeat_hash_check.txt",
    ),
    "firmware_v1": (
        "firmware_release_evidence.md",
        "hash_check.txt",
        "replay_manifest_v1.txt",
        "sha256_summary.txt",
    ),
    "firmware_v0": (
        "firmware_release_evidence.md",
        "hash_check.txt",
        "replay_manifest_v0.txt",
        "sha256_summary.txt",
    ),
}

FIRMWARE_SENTINEL_FILES = frozenset(
    {
        "firmware_release_evidence.md",
        "fw_capture_hash_check.txt",
        "fw_repeat_hash_check.txt",
        "hash_check.txt",
        "replay_manifest_v0.txt",
        "replay_manifest_v1.txt",
        "sha256_summary.txt",
    }
)

SUMMARY_FILES = ("summary.md", "summary.json")


def load_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def display_path(path: Path, repo_root: Path) -> str:
    try:
        return path.relative_to(repo_root).as_posix()
    except ValueError:
        return str(path)


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


def parse_repeat_manifest(path: Path) -> tuple[dict[str, str], list[dict[str, str]]]:
    header: dict[str, str] = {}
    runs: list[dict[str, str]] = []
    for raw_line in load_text(path).splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        fields: dict[str, str] = {}
        for part in line.split():
            if "=" not in part:
                continue
            key, value = part.split("=", 1)
            fields[key] = value
        if fields.get("contract") == "rpl0" and "run" in fields:
            runs.append(fields)
        else:
            header.update(fields)
    return header, runs


def validate_rpl0_archive_bundle(bundle_dir: Path, repo_root: Path) -> tuple[list[str], list[str]]:
    errors: list[str] = []
    warnings: list[str] = []
    capture_hash_path = bundle_dir / "fw_capture_hash_check.txt"
    repeat_hash_path = bundle_dir / "fw_repeat_hash_check.txt"
    manifest_path = bundle_dir / "fw_repeat" / "replay_manifest_v1.txt"
    capture_path = bundle_dir / "fw_capture.bin"
    repeat_dir = bundle_dir / "fw_repeat"

    for rel_path in FIRMWARE_REQUIRED_FILE_SETS["rpl0_archive_v1"]:
        path = bundle_dir / rel_path
        if not path.is_file():
            errors.append(f"missing required retained file: {rel_path}")
    if errors:
        return errors, warnings

    capture_entries = parse_hash_check(capture_hash_path)
    if len(capture_entries) != 1:
        errors.append("fw_capture_hash_check.txt must contain exactly one hash entry")
    else:
        capture_sha, capture_label = capture_entries[0]
        retained_capture_sha = sha256_file(capture_path)
        if retained_capture_sha != capture_sha:
            errors.append("fw_capture_hash_check.txt does not match retained fw_capture.bin")
        if capture_label != "artifacts/run.bin":
            warnings.append(f"fw_capture_hash_check.txt path label is non-canonical: {capture_label}")

    repeat_entries = parse_hash_check(repeat_hash_path)
    repeat_hash_map = {Path(label).name: sha for sha, label in repeat_entries}
    if len(repeat_hash_map) != len(repeat_entries):
        errors.append("fw_repeat_hash_check.txt contains duplicate retained run filenames")

    manifest_header, manifest_runs = parse_repeat_manifest(manifest_path)
    if manifest_header.get("contract") != "rpl0":
        errors.append("fw_repeat/replay_manifest_v1.txt contract must be rpl0")
    if manifest_header.get("reset_mode") != "manual":
        errors.append("fw_repeat/replay_manifest_v1.txt reset_mode must be manual")
    if manifest_header.get("signal_model") != "phase8":
        errors.append("fw_repeat/replay_manifest_v1.txt signal_model must be phase8")
    try:
        expected_runs = int(manifest_header.get("runs", ""))
    except ValueError:
        errors.append("fw_repeat/replay_manifest_v1.txt runs must be an integer")
        expected_runs = -1
    if expected_runs != len(manifest_runs):
        errors.append("fw_repeat/replay_manifest_v1.txt run count does not match runs header")

    for run in manifest_runs:
        filename = run.get("file")
        if not filename:
            errors.append("fw_repeat/replay_manifest_v1.txt run entry missing file")
            continue
        retained_run = repeat_dir / filename
        if not retained_run.is_file():
            errors.append(f"retained repeat run artifact missing: fw_repeat/{filename}")
            continue
        expected_sha = run.get("sha256")
        if expected_sha is None or not SHA256_RE.fullmatch(expected_sha):
            errors.append(f"fw_repeat/replay_manifest_v1.txt invalid sha256 for {filename}")
            continue
        actual_sha = sha256_file(retained_run)
        if actual_sha != expected_sha:
            errors.append(f"retained repeat run hash mismatch for fw_repeat/{filename}")
        if repeat_hash_map.get(filename) != expected_sha:
            errors.append(f"fw_repeat_hash_check.txt does not match manifest for {filename}")
        if run.get("status") != "PASS":
            errors.append(f"fw_repeat/replay_manifest_v1.txt retained run did not pass: {filename}")

    capture_sha = capture_entries[0][0] if len(capture_entries) == 1 else None
    manifest_shas = {run.get("sha256") for run in manifest_runs if run.get("sha256")}
    if capture_sha and manifest_shas and manifest_shas != {capture_sha}:
        errors.append("retained repeat run hashes diverge from single capture hash")

    evidence_text = load_text(bundle_dir / "firmware_release_evidence.md")
    for required_text in ("## capture hash check", "## repeat manifest"):
        if required_text not in evidence_text:
            errors.append(f"firmware_release_evidence.md missing section: {required_text}")

    return errors, warnings


def sha256_file(path: Path) -> str:
    import hashlib

    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def validate_summary_files(bundle_dir: Path, repo_root: Path) -> tuple[list[str], list[str]]:
    errors: list[str] = []
    warnings: list[str] = []
    summary_md = bundle_dir / "summary.md"
    summary_json = bundle_dir / "summary.json"
    if not summary_md.exists() and not summary_json.exists():
        return errors, warnings
    for path in (summary_md, summary_json):
        if not path.is_file():
            errors.append(f"missing release summary file: {path.name}")
    if errors:
        return errors, warnings

    try:
        summary = json.loads(load_text(summary_json))
    except json.JSONDecodeError as exc:
        errors.append(f"summary.json is not valid JSON: {exc}")
        return errors, warnings

    if summary.get("schema") != "precision.release_summary.v1":
        errors.append("summary.json schema must be precision.release_summary.v1")
    if summary.get("version") != bundle_dir.name:
        errors.append(f"summary.json version must match bundle directory name: {bundle_dir.name}")
    hashes = summary.get("hashes")
    if not isinstance(hashes, dict):
        errors.append("summary.json hashes must be an object")
        return errors, warnings

    actual_files = {
        path.relative_to(bundle_dir).as_posix(): sha256_file(path)
        for path in bundle_dir.rglob("*")
        if path.is_file() and path.name not in SUMMARY_FILES
    }
    if set(hashes) != set(actual_files):
        missing = sorted(set(actual_files) - set(hashes))
        extra = sorted(set(hashes) - set(actual_files))
        if missing:
            errors.append(f"summary.json missing hash entries: {missing}")
        if extra:
            errors.append(f"summary.json has stale hash entries: {extra}")
    for rel_path, actual_sha in actual_files.items():
        summary_sha = hashes.get(rel_path)
        if summary_sha is not None and summary_sha != actual_sha:
            errors.append(f"summary.json hash mismatch for {rel_path}")

    summary_text = load_text(summary_md)
    if "# Release Bundle Summary" not in summary_text:
        errors.append("summary.md missing release summary heading")
    if "## Hashes" not in summary_text:
        errors.append("summary.md missing hashes section")
    return errors, warnings


def validate_run_dir(
    run_dir_value: str,
    run_id: str,
    manifest_name: str,
    repo_root: Path,
    strict_paths: bool,
) -> tuple[Path, list[str], list[str]]:
    errors: list[str] = []
    warnings: list[str] = []
    manifest_run_dir = Path(run_dir_value)

    if manifest_run_dir.is_absolute():
        resolved_run_dir = manifest_run_dir.resolve()
        if strict_paths:
            errors.append(
                f"{manifest_name} run_dir must be repo-relative in --strict-paths mode: {run_dir_value}"
            )
            return resolved_run_dir, errors, warnings
        warnings.append(
            f"{manifest_name} run_dir is absolute and non-portable: {run_dir_value}"
        )
        if resolved_run_dir.name != run_id:
            errors.append(
                f"{manifest_name} run_dir basename does not match retained run id {run_id}: {run_dir_value}"
            )
        if not resolved_run_dir.is_dir():
            errors.append(f"{manifest_name} run_dir does not exist: {run_dir_value}")
        return resolved_run_dir, errors, warnings

    normalized_run_dir = manifest_run_dir.as_posix()
    match = RUN_DIR_REL_RE.fullmatch(normalized_run_dir)
    if match is None:
        errors.append(
            f"{manifest_name} run_dir must match artifacts/replay_runs/run_<timestamp>: "
            f"{run_dir_value}"
        )
        return (repo_root / manifest_run_dir).resolve(), errors, warnings
    if match.group(1) != run_id:
        errors.append(
            f"{manifest_name} run_dir does not match retained run id {run_id}: {run_dir_value}"
        )
    resolved_run_dir = (repo_root / manifest_run_dir).resolve()
    if not resolved_run_dir.is_dir():
        errors.append(f"{manifest_name} run_dir does not exist: {run_dir_value}")
    return resolved_run_dir, errors, warnings


def validate_bundle(bundle_dir: Path, repo_root: Path, strict_paths: bool) -> tuple[list[str], list[str]]:
    if not bundle_dir.is_dir():
        return [f"retained release bundle directory does not exist: {display_path(bundle_dir, repo_root)}"], []

    file_names = {path.name for path in bundle_dir.iterdir() if path.is_file()}
    rel_file_names = {
        path.relative_to(bundle_dir).as_posix()
        for path in bundle_dir.rglob("*")
        if path.is_file()
    }
    errors: list[str] = []
    warnings: list[str] = []

    if "replay_manifest_v0.txt" in file_names and "replay_manifest_v1.txt" in file_names:
        return ["retained release bundle cannot contain both replay_manifest_v0.txt and replay_manifest_v1.txt"], []

    bundle_class = "non_firmware"
    required_files = NON_FIRMWARE_REQUIRED_FILES
    manifest_name = "replay_manifest_v1.txt"
    if {
        "fw_capture_hash_check.txt",
        "fw_repeat_hash_check.txt",
        "fw_repeat/replay_manifest_v1.txt",
    } & rel_file_names:
        bundle_class = "rpl0_archive_v1"
        required_files = FIRMWARE_REQUIRED_FILE_SETS[bundle_class]
    elif file_names & FIRMWARE_SENTINEL_FILES:
        if "replay_manifest_v0.txt" in file_names:
            bundle_class = "firmware_v0"
            manifest_name = "replay_manifest_v0.txt"
        else:
            bundle_class = "firmware_v1"
            manifest_name = "replay_manifest_v1.txt"
        required_files = FIRMWARE_REQUIRED_FILE_SETS[bundle_class]

    required = {name: bundle_dir / name for name in required_files}
    for name, path in required.items():
        if name == "index.md" and bundle_class != "non_firmware":
            continue
        if not path.is_file():
            errors.append(f"missing required retained file: {name}")
    if errors:
        return errors, warnings

    summary_errors, summary_warnings = validate_summary_files(bundle_dir, repo_root)
    errors.extend(summary_errors)
    warnings.extend(summary_warnings)
    if errors:
        return errors, warnings

    if bundle_class == "non_firmware":
        return errors, warnings
    if bundle_class == "rpl0_archive_v1":
        rpl_errors, rpl_warnings = validate_rpl0_archive_bundle(bundle_dir, repo_root)
        errors.extend(rpl_errors)
        warnings.extend(rpl_warnings)
        return errors, warnings

    evidence_text = load_text(required["firmware_release_evidence.md"])
    manifest_text = load_text(required[manifest_name])
    hash_check_text = load_text(required["hash_check.txt"])

    run_ids = set(RUN_ID_RE.findall(evidence_text))
    run_ids.update(RUN_ID_RE.findall(manifest_text))
    run_ids.update(RUN_ID_RE.findall(hash_check_text))
    if len(run_ids) != 1:
        errors.append(f"expected exactly one run id across retained files, found: {sorted(run_ids)}")
        return errors, warnings
    run_id = next(iter(run_ids))

    manifest = parse_manifest(required[manifest_name])
    for key in ("baseline_path", "baseline_sha256", "completed_runs", "timestamp_utc", "run_dir"):
        if key not in manifest:
            errors.append(f"{manifest_name} missing required key: {key}")
    if errors:
        return errors, warnings

    baseline_sha, summary_runs = parse_sha256_summary(required["sha256_summary.txt"])
    hash_entries = parse_hash_check(required["hash_check.txt"])
    hash_map = {path: sha for sha, path in hash_entries}
    expected_baseline_path = manifest["baseline_path"]
    baseline_path = repo_root / expected_baseline_path
    run_dir_value = manifest["run_dir"]
    run_dir, run_dir_errors, run_dir_warnings = validate_run_dir(
        run_dir_value,
        run_id,
        manifest_name,
        repo_root,
        strict_paths,
    )
    errors.extend(run_dir_errors)
    warnings.extend(run_dir_warnings)

    if not baseline_path.is_file():
        errors.append(f"{manifest_name} baseline_path does not exist: {expected_baseline_path}")

    manifest_baseline_sha = manifest["baseline_sha256"]
    if baseline_sha != manifest_baseline_sha:
        errors.append(f"baseline sha mismatch between sha256_summary.txt and {manifest_name}")
    if hash_map.get(expected_baseline_path) != manifest_baseline_sha:
        errors.append(f"baseline sha mismatch between hash_check.txt and {manifest_name}")
    if hash_map.get("artifacts/run.bin") != manifest_baseline_sha:
        errors.append("artifacts/run.bin hash does not match retained baseline sha")

    completed_runs = int(manifest["completed_runs"])
    if completed_runs != len(summary_runs):
        errors.append("completed_runs does not match sha256_summary.txt run count")

    run_entries = [(sha, rel_path) for sha, rel_path in hash_entries if rel_path.startswith("artifacts/replay_runs/")]
    if len(run_entries) != completed_runs:
        errors.append(f"hash_check.txt run count does not match {manifest_name} completed_runs")

    expected_run_paths = [f"artifacts/replay_runs/{run_id}/run_{idx:02d}.bin" for idx in range(1, completed_runs + 1)]
    actual_run_paths = [rel_path for _, rel_path in run_entries]
    if actual_run_paths != expected_run_paths:
        errors.append(f"hash_check.txt run paths do not match retained run id/path set: {actual_run_paths}")

    summary_map = {name: sha for sha, name in summary_runs}
    if len(summary_map) != len(summary_runs):
        errors.append("sha256_summary.txt contains duplicate run filenames")
    for expected_path in expected_run_paths:
        repo_path = (repo_root / expected_path).resolve()
        if not repo_path.is_file():
            errors.append(f"referenced run artifact does not exist: {expected_path}")
            continue
        if repo_path.parent != run_dir:
            errors.append(
                f"referenced run artifact is not located under {manifest_name} run_dir {run_dir_value}: {expected_path}"
            )
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
        errors.append(f"firmware_release_evidence.md TIMESTAMP_UTC does not match {manifest_name}")
    evidence_run_dir = re.search(r"^RUN_DIR=(.+)$", evidence_text, re.MULTILINE)
    if evidence_run_dir:
        evidence_run_dir_value = evidence_run_dir.group(1).strip()
        allowed_run_dirs = {
            f"artifacts/replay_runs/{run_id}",
            run_dir_value,
            str(run_dir),
        }
        if evidence_run_dir_value not in allowed_run_dirs:
            errors.append("firmware_release_evidence.md RUN_DIR does not match retained run path")

    return errors, warnings


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
    parser.add_argument(
        "--strict-paths",
        action="store_true",
        help="require repo-relative replay_manifest_v1.txt run_dir values",
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
        errors, warnings = validate_bundle(bundle_dir, repo_root, args.strict_paths)
    except ValueError as exc:
        print(f"FAIL: retained release bundle coherence: {display_path(bundle_dir, repo_root)}")
        print(f"- {exc}")
        return 1
    if errors:
        print(f"FAIL: retained release bundle coherence: {display_path(bundle_dir, repo_root)}")
        for error in errors:
            print(f"- {error}")
        return 1

    print(f"PASS: retained release bundle coherence: {display_path(bundle_dir, repo_root)}")
    for warning in warnings:
        print(f"WARN: {warning}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
