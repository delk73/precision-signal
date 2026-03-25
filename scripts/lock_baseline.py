#!/usr/bin/env python3
import argparse
import hashlib
import json
import os
import re
import shutil
import socket
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

import inspect_artifact

DEFAULT_ARTIFACTS_DIR = Path("artifacts")
DEFAULT_SOURCE = Path("artifacts/run_01.bin")
DEFAULT_SERIAL = "/dev/ttyACM0"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Lock a canonical baseline artifact from a verified repeatability run."
    )
    parser.add_argument(
        "--source",
        type=Path,
        default=DEFAULT_SOURCE,
        help="Known-good run artifact to promote (default: artifacts/run_01.bin).",
    )
    parser.add_argument(
        "--artifacts-dir",
        type=Path,
        default=DEFAULT_ARTIFACTS_DIR,
        help="Artifacts directory (default: artifacts).",
    )
    parser.add_argument(
        "--serial-port",
        default=os.environ.get("SERIAL", DEFAULT_SERIAL),
        help="Serial port used during capture (default: SERIAL env or /dev/ttyACM0).",
    )
    parser.add_argument(
        "--fw-features",
        default="debug-irq-count",
        help="Firmware feature string used for capture (default: debug-irq-count).",
    )
    parser.add_argument(
        "--signal-model",
        required=True,
        help="Signal model used for capture (e.g. phase8, ramp). Required.",
    )
    parser.add_argument(
        "--manifest-name",
        default="manifest.txt",
        help="Manifest filename inside artifacts-dir (default: manifest.txt).",
    )
    parser.add_argument(
        "--notes",
        default="",
        help="Optional notes to include in baseline.json.",
    )
    return parser.parse_args()


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def git_commit_short() -> str:
    try:
        out = subprocess.check_output(["git", "rev-parse", "--short=12", "HEAD"], text=True)
        return out.strip()
    except Exception:
        return "unknown"


def find_manifest_entry(manifest_path: Path, source_name: str) -> tuple[bool, str]:
    if not manifest_path.exists():
        return False, "manifest.txt not found"

    pat = re.compile(r"file=(\S+)\s+bytes=(\d+)\s+sha256=([0-9a-f]{64})\s+status=(\S+)")
    with manifest_path.open("r", encoding="utf-8") as f:
        for line in f:
            m = pat.search(line)
            if not m:
                continue
            file_name, _bytes, digest, status = m.groups()
            if file_name == source_name:
                if status == "PASS":
                    return True, digest
                return False, f"source entry status is {status}"

    return False, "source artifact not found in manifest PASS entries"


def verify_source(source: Path, signal_model: str) -> bool:
    cmd = [
        sys.executable,
        str(Path(__file__).parent / "artifact_tool.py"),
        "verify",
        str(source),
        "--signal-model",
        signal_model,
    ]
    proc = subprocess.run(cmd, capture_output=True, text=True)
    if proc.returncode != 0:
        print(f"FAIL: pre-promotion verify failed for {source}")
        if proc.stdout:
            print(proc.stdout.rstrip())
        if proc.stderr:
            print(proc.stderr.rstrip())
        return False
    return True


def main() -> int:
    args = parse_args()
    artifacts_dir = args.artifacts_dir
    artifacts_dir.mkdir(parents=True, exist_ok=True)

    source = args.source
    if not source.exists():
        print(f"FAIL: source artifact not found: {source}")
        return 1

    if not verify_source(source, args.signal_model):
        return 1

    manifest_path = artifacts_dir / args.manifest_name
    source_name = source.name
    ok, manifest_value = find_manifest_entry(manifest_path, source_name)
    if not ok:
        print(f"FAIL: {manifest_value}")
        return 1

    computed_hash = sha256_file(source)
    if computed_hash != manifest_value:
        print("FAIL: source hash does not match manifest")
        return 1

    try:
        artifact = inspect_artifact.parse_artifact(source, allow_trailing=False)
    except ValueError as exc:
        print(f"FAIL: invalid source artifact ({exc})")
        return 1

    baseline_bin = artifacts_dir / "baseline.bin"
    baseline_sha = artifacts_dir / "baseline.sha256"
    baseline_json = artifacts_dir / "baseline.json"

    shutil.copyfile(source, baseline_bin)
    baseline_hash = sha256_file(baseline_bin)

    with baseline_sha.open("w", encoding="utf-8") as f:
        f.write(f"{baseline_hash}  baseline.bin\n")


    metadata = {
        "artifact_file": "baseline.bin",
        "sha256": baseline_hash,
        "artifact_version": artifact["version"],
        "schema_hash": artifact["schema_hash"].hex() if artifact["schema_hash"] is not None else None,
        "header_len": artifact["header_len"],
        "schema_len": artifact["schema_len"],
        "signal_model": args.signal_model,
        "frame_count": artifact["frame_count"],
        "frame_size": artifact["frame_size"],
        "irq_id": 2,
        "timer_delta_nominal": 1000,
        "input_xor": "0x5A5A1F1F",
        "serial_port": args.serial_port,
        "firmware_crate": "replay-fw-f446",
        "firmware_target": "thumbv7em-none-eabihf",
        "fw_features": args.fw_features,
        "capture_method": "manual reset button",
        "verification_status": "PASS",
        "git_commit": git_commit_short(),
        "captured_at_utc": datetime.now(timezone.utc).isoformat(),
        "host_machine": socket.gethostname(),
        "notes": args.notes,
        "source_run_file": source_name,
    }

    with baseline_json.open("w", encoding="utf-8") as f:
        json.dump(metadata, f, indent=2)
        f.write("\n")

    print(f"PASS: wrote {baseline_bin}")
    print(f"PASS: wrote {baseline_sha}")
    print(f"PASS: wrote {baseline_json}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
