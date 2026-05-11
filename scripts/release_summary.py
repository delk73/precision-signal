#!/usr/bin/env python3
"""Generate machine and operator summaries for retained release bundles."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any


SUMMARY_FILES = {"summary.json", "summary.md"}
VALIDATION_OUTPUT_NAMES = {
    "cargo_check_dpw4_thumb_locked.txt",
    "kani_evidence.txt",
    "make_demo_evidence_package.txt",
    "make_doc_link_check.txt",
    "make_gate.txt",
    "make_release_bundle_check.txt",
    "make_replay_tests.txt",
    "release_reproducibility.txt",
    "firmware_release_evidence.md",
    "fw_capture_hash_check.txt",
    "fw_repeat_hash_check.txt",
    "hash_check.txt",
    "sha256_summary.txt",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Generate release bundle summaries.")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument("--version")
    parser.add_argument("--release-root", default="docs/verification/releases")
    parser.add_argument("--bundle-dir", type=Path)
    return parser.parse_args()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def rel(path: Path, root: Path) -> str:
    try:
        return path.relative_to(root).as_posix()
    except ValueError:
        return path.as_posix()


def git_value(root: Path, *args: str) -> str | None:
    result = subprocess.run(
        ["git", *args],
        cwd=root,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        return None
    value = result.stdout.strip()
    return value or None


def classify(path: str) -> str:
    name = Path(path).name
    if name in VALIDATION_OUTPUT_NAMES:
        return "validation"
    if path.endswith(".bin") or "/fw_repeat/run_" in path:
        return "artifact"
    if name in {"index.md", "replay_manifest_v1.txt", "replay_manifest_v0.txt"}:
        return "metadata"
    return "support"


def build_summary(root: Path, bundle_dir: Path, version: str) -> dict[str, Any]:
    files: list[dict[str, Any]] = []
    for path in sorted(bundle_dir.rglob("*")):
        if not path.is_file() or path.name in SUMMARY_FILES:
            continue
        relative = path.relative_to(bundle_dir).as_posix()
        files.append(
            {
                "path": relative,
                "bundle_path": rel(path, root),
                "size_bytes": path.stat().st_size,
                "sha256": sha256_file(path),
                "kind": classify(relative),
            }
        )

    return {
        "schema": "precision.release_summary.v1",
        "version": version,
        "bundle": {
            "path": rel(bundle_dir, root),
            "summary_json": rel(bundle_dir / "summary.json", root),
            "summary_md": rel(bundle_dir / "summary.md", root),
        },
        "metadata": {
            "git_commit": git_value(root, "rev-parse", "HEAD"),
            "git_branch": git_value(root, "branch", "--show-current"),
        },
        "hashes": {entry["path"]: entry["sha256"] for entry in files},
        "artifacts": [entry for entry in files if entry["kind"] == "artifact"],
        "validation_outputs": [entry for entry in files if entry["kind"] == "validation"],
        "metadata_outputs": [entry for entry in files if entry["kind"] == "metadata"],
        "files": files,
    }


def render_markdown(summary: dict[str, Any]) -> str:
    lines = [
        f"# Release Bundle Summary ({summary['version']})",
        "",
        "## Bundle Paths",
        f"- bundle: {summary['bundle']['path']}",
        f"- summary_json: {summary['bundle']['summary_json']}",
        f"- summary_md: {summary['bundle']['summary_md']}",
        "",
        "## Key Metadata",
        f"- schema: {summary['schema']}",
        f"- git_commit: {summary['metadata'].get('git_commit') or 'unknown'}",
        f"- git_branch: {summary['metadata'].get('git_branch') or 'unknown'}",
        f"- artifact_count: {len(summary['artifacts'])}",
        f"- validation_output_count: {len(summary['validation_outputs'])}",
        f"- retained_file_count: {len(summary['files'])}",
        "",
        "## Artifacts",
    ]
    if summary["artifacts"]:
        for entry in summary["artifacts"]:
            lines.append(
                f"- {entry['path']} sha256={entry['sha256']} size_bytes={entry['size_bytes']}"
            )
    else:
        lines.append("- none")

    lines.extend(["", "## Validation Outputs"])
    if summary["validation_outputs"]:
        for entry in summary["validation_outputs"]:
            lines.append(
                f"- {entry['path']} sha256={entry['sha256']} size_bytes={entry['size_bytes']}"
            )
    else:
        lines.append("- none")

    lines.extend(["", "## Metadata Outputs"])
    if summary["metadata_outputs"]:
        for entry in summary["metadata_outputs"]:
            lines.append(
                f"- {entry['path']} sha256={entry['sha256']} size_bytes={entry['size_bytes']}"
            )
    else:
        lines.append("- none")

    lines.extend(["", "## Hashes"])
    for path, digest in sorted(summary["hashes"].items()):
        lines.append(f"- {path} {digest}")
    lines.append("")
    return "\n".join(lines)


def main() -> int:
    args = parse_args()
    root = args.root.resolve()
    if args.bundle_dir is not None:
        bundle_dir = args.bundle_dir.resolve()
        version = args.version or bundle_dir.name
    elif args.version:
        bundle_dir = (root / args.release_root / args.version).resolve()
        version = args.version
    else:
        raise SystemExit("pass --version or --bundle-dir")

    if not bundle_dir.is_dir():
        raise SystemExit(f"missing bundle path: {rel(bundle_dir, root)}")

    summary = build_summary(root, bundle_dir, version)
    (bundle_dir / "summary.json").write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (bundle_dir / "summary.md").write_text(render_markdown(summary), encoding="utf-8")
    print(f"wrote release summaries: {rel(bundle_dir / 'summary.md', root)}, {rel(bundle_dir / 'summary.json', root)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
