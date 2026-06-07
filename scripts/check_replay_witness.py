#!/usr/bin/env python3
"""Validate retained release replay witness evidence."""

from __future__ import annotations

import argparse
import re
import sys
from collections.abc import Mapping
from pathlib import Path

SCRIPTS_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPTS_DIR.parent
sys.path.insert(0, str(SCRIPTS_DIR))

import rpl0_witness


REQUIRED_FIELDS = ("RESULT", "WITNESS_DIGEST")
WITNESS_DIGEST_RE = re.compile(r"^[0-9a-f]{16}$")


class WitnessCheckError(ValueError):
    """Raised when retained or recomputed witness evidence is invalid."""


def resolve_release_root(release_root: Path) -> Path:
    if release_root.is_absolute():
        return release_root
    return REPO_ROOT / release_root


def validate_witness_fields(fields: dict[str, str], label: str) -> dict[str, str]:
    for key in REQUIRED_FIELDS:
        if key not in fields:
            raise WitnessCheckError(f"{label}: missing {key}")
        if fields[key] == "":
            raise WitnessCheckError(f"{label}: empty {key}")
    return fields


def require_witness_digest_format(fields: dict[str, str], label: str) -> None:
    if not WITNESS_DIGEST_RE.fullmatch(fields["WITNESS_DIGEST"]):
        raise WitnessCheckError(f"{label}: malformed WITNESS_DIGEST: {fields['WITNESS_DIGEST']}")


def parse_witness_fields(report: str | Mapping[str, object], label: str) -> dict[str, str]:
    if isinstance(report, Mapping):
        fields: dict[str, str] = {}
        for key in REQUIRED_FIELDS:
            if key not in report:
                raise WitnessCheckError(f"{label}: missing {key}")
            fields[key] = str(report[key]).strip()
        return validate_witness_fields(fields, label)

    fields: dict[str, str] = {}
    for line_number, raw_line in enumerate(report.splitlines(), start=1):
        line = raw_line.strip()
        if not line:
            continue
        if ":" not in line:
            raise WitnessCheckError(f"{label}: malformed line {line_number}: {raw_line!r}")
        key, value = line.split(":", 1)
        key = key.strip()
        value = value.strip()
        if not key:
            raise WitnessCheckError(f"{label}: malformed line {line_number}: empty field name")
        if key in REQUIRED_FIELDS:
            if key in fields:
                raise WitnessCheckError(f"{label}: duplicate {key}")
            fields[key] = value

    return validate_witness_fields(fields, label)


def check_release_witness(version: str, release_root: Path) -> str:
    root = resolve_release_root(release_root)
    release_dir = root / version
    artifact_path = release_dir / "fw_capture.bin"
    retained_report_path = release_dir / "rpl0_witness_fw_capture.txt"

    if not release_dir.is_dir():
        raise WitnessCheckError(f"missing retained release directory: {release_dir}")
    if not artifact_path.is_file():
        raise WitnessCheckError(f"missing retained artifact: {artifact_path}")
    if not retained_report_path.is_file():
        raise WitnessCheckError(f"missing retained witness report: {retained_report_path}")

    try:
        retained_report = retained_report_path.read_text(encoding="utf-8")
    except UnicodeDecodeError as exc:
        raise WitnessCheckError(f"cannot decode retained witness report: {retained_report_path}: {exc}") from exc
    except OSError as exc:
        raise WitnessCheckError(f"cannot read retained witness report: {retained_report_path}: {exc}") from exc
    try:
        artifact = artifact_path.read_bytes()
    except OSError as exc:
        raise WitnessCheckError(f"cannot read retained artifact: {artifact_path}: {exc}") from exc

    retained = parse_witness_fields(retained_report, "retained witness report")
    if retained["RESULT"] != "PASS":
        raise WitnessCheckError(f"retained witness report RESULT is not PASS: {retained['RESULT']}")
    require_witness_digest_format(retained, "retained witness report")

    recomputed_report = rpl0_witness.witness(artifact, str(artifact_path))
    recomputed = parse_witness_fields(recomputed_report, "recomputed witness report")
    if recomputed["RESULT"] != "PASS":
        raise WitnessCheckError(f"recomputed witness report RESULT is not PASS: {recomputed['RESULT']}")
    require_witness_digest_format(recomputed, "recomputed witness report")

    retained_digest = retained["WITNESS_DIGEST"]
    recomputed_digest = recomputed["WITNESS_DIGEST"]
    if retained_digest != recomputed_digest:
        raise WitnessCheckError(
            "witness digest mismatch: "
            f"retained={retained_digest} recomputed={recomputed_digest}"
        )

    return retained_digest


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate retained release replay witness evidence.")
    parser.add_argument("--version", required=True, help="release version under docs/verification/releases/")
    parser.add_argument(
        "--release-root",
        type=Path,
        default=Path("docs/verification/releases"),
        help="retained release root (default: docs/verification/releases)",
    )
    args = parser.parse_args()

    try:
        digest = check_release_witness(args.version, args.release_root)
    except WitnessCheckError as exc:
        print(f"FAIL: replay witness check: {exc}", file=sys.stderr)
        return 1

    print(f"replay witness check: PASS VERSION={args.version} WITNESS_DIGEST={digest}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
