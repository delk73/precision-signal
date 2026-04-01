#!/usr/bin/env python3
"""Deterministic CLI regression tests for `artifact_diff.py`."""

import struct
import subprocess
import sys
import tempfile
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import inspect_artifact


REPO_ROOT = Path(__file__).resolve().parent.parent
FIXTURE_GEN = ["python3", "scripts/generate_demo_v3_fixtures.py"]
DIFF = ["python3", "scripts/artifact_diff.py"]
SAMPLE_OFFSET = 12


def run(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=REPO_ROOT, text=True, capture_output=True, check=False)


def require(proc: subprocess.CompletedProcess[str], rc: int, contains: str, name: str) -> None:
    if proc.returncode != rc:
        raise AssertionError(
            f"{name}: expected rc={rc}, got rc={proc.returncode}\nstdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )
    if contains not in proc.stdout:
        raise AssertionError(
            f"{name}: missing output '{contains}'\nstdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )


def canonical_capture_csv() -> str:
    rows = ["index,interval_us"]
    for idx in range(138):
        interval = 305564 if idx == 0 else 304000
        rows.append(f"{idx},{interval}")
    return "\n".join(rows) + "\n"


def controlled_perturbation_capture_csv() -> str:
    rows = ["index,interval_us"]
    for idx in range(138):
        if idx == 0:
            interval = 305564
        elif idx == 17:
            interval = 304001
        else:
            interval = 304000
        rows.append(f"{idx},{interval}")
    return "\n".join(rows) + "\n"


def run_import(csv_path: Path, out_path: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "replay-host",
            "--",
            "import-interval-csv",
            str(csv_path),
            str(out_path),
        ],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )


def set_sample(path: Path, frame_idx: int, new_sample: int) -> None:
    parsed = inspect_artifact.parse_artifact(path, allow_trailing=False)
    data = bytearray(path.read_bytes())
    frame_base = parsed["frames_offset"] + (frame_idx * parsed["frame_size"])
    sample_offset = frame_base + SAMPLE_OFFSET
    struct.pack_into("<i", data, sample_offset, new_sample)
    path.write_bytes(data)


def main() -> int:
    require(run(FIXTURE_GEN), 0, "PASS: demo-v3 fixtures generated", "fixture_gen")

    require(
        run(DIFF + ["artifacts/demo_v3/transient_A.rpl", "artifacts/demo_v3/transient_B.rpl"]),
        0,
        "Classification: transient",
        "transient",
    )
    require(
        run(DIFF + ["artifacts/demo_v3/offset_A.rpl", "artifacts/demo_v3/offset_B.rpl"]),
        0,
        "Classification: persistent_offset",
        "persistent_offset",
    )
    require(
        run(DIFF + ["artifacts/demo_v3/rate_A.rpl", "artifacts/demo_v3/rate_B.rpl"]),
        0,
        "Classification: rate_divergence",
        "rate_divergence",
    )
    require(
        run(DIFF + ["artifacts/demo_v3/rate_A.rpl", "artifacts/demo_v3/rate_A.rpl"]),
        0,
        "NO DIVERGENCE: artifacts identical",
        "no_divergence",
    )
    with tempfile.TemporaryDirectory(prefix="dpw_artifact_diff_") as tmp:
        tmp_dir = Path(tmp)
        baseline_csv = tmp_dir / "baseline.csv"
        perturbed_csv = tmp_dir / "perturbed.csv"
        imported_a = tmp_dir / "baseline_a.rpl"
        imported_b = tmp_dir / "baseline_b.rpl"
        perturbed_artifact = tmp_dir / "perturbed.rpl"
        baseline_csv.write_text(canonical_capture_csv(), encoding="utf-8")
        perturbed_csv.write_text(controlled_perturbation_capture_csv(), encoding="utf-8")
        require(run_import(baseline_csv, imported_a), 0, "wrote:", "rpl0_import_baseline_a")
        require(run_import(baseline_csv, imported_b), 0, "wrote:", "rpl0_import_baseline_b")
        require(
            run_import(perturbed_csv, perturbed_artifact),
            0,
            "wrote:",
            "rpl0_import_perturbed",
        )

        require(
            run(DIFF + [str(imported_a), str(imported_b)]),
            0,
            "NO DIVERGENCE: artifacts identical",
            "rpl0_no_divergence",
        )
        controlled = run(DIFF + [str(imported_a), str(perturbed_artifact)])
        require(controlled, 0, "First divergence frame: 17", "rpl0_controlled_frame")
        for needle in (
            "Classification: transient",
            "Sample A: 0x0004A380",
            "Sample B: 0x0004A381",
            "Differing fields: sample",
            "first_divergence_frame: 17",
            "shape_class: transient",
            "all_regions_at_first_divergence: [sample_payload]",
            "evolution_class: self_healing",
            "timeline_summary: divergence resolves within 1 frame",
            "reconvergence_summary: reconverged_at_frame=18",
        ):
            if needle not in controlled.stdout:
                raise AssertionError(
                    f"rpl0_controlled_divergence: missing output {needle!r}\n"
                    f"stdout:\n{controlled.stdout}\nstderr:\n{controlled.stderr}"
                )

        a_path = tmp_dir / "nonclass_a.rpl"
        b_path = tmp_dir / "nonclass_b.rpl"
        baseline = REPO_ROOT / "artifacts" / "demo_v3" / "rate_A.rpl"
        a_path.write_bytes(baseline.read_bytes())
        b_path.write_bytes(baseline.read_bytes())

        # Nonclassifiable by design:
        # - no sustained reconvergence in transient window
        # - non-constant offset
        # - abs(diff) is not nondecreasing
        base = inspect_artifact.parse_artifact(b_path, allow_trailing=False)
        start = 4096
        offsets = [1, 2, 1, 2, 1, 2, 1, 2, 1]
        for rel, offset in enumerate(offsets):
            frame_idx = start + rel
            sample = base["frames"][frame_idx]["input_sample"]
            set_sample(b_path, frame_idx, sample + offset)

        require(
            run(DIFF + [str(a_path), str(b_path)]),
            1,
            "does not satisfy rate-divergence monotonic-growth rule",
            "nonclassifiable_shape",
        )

    print("PASS: artifact_diff CLI regression suite")
    return 0


if __name__ == "__main__":
    sys.exit(main())
