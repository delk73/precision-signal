#!/usr/bin/env python3
"""Prompt-driven metadata helper for the STM32 power sweep run matrix."""

from __future__ import annotations

import argparse
import csv
import sys
from datetime import datetime
from pathlib import Path


DEFAULT_MATRIX = Path(
    "artifacts/physical_characterization/"
    "stm32_power_sweep_2026-05-12/run_matrix.csv"
)

COLUMNS = [
    "run_id",
    "label",
    "status",
    "setpoint_v",
    "measured_v",
    "measured_current_ma",
    "power_source",
    "power_instrument",
    "measurement_time",
    "settle_seconds",
    "power_interaction",
    "input_rail_observation",
    "vdd_rail_observation",
    "supply_path",
    "reset_mode",
    "capture_state",
    "capture_rows",
    "artifact_path",
    "artifact_sha256",
    "replay_result",
    "first_divergence",
    "classification",
    "notes",
]

DEFAULTS = {
    "power_source": "external bench supply",
    "power_interaction": (
        "external_power_dominant; ST-LINK/USB connected; "
        "residual interface-side contribution not isolated"
    ),
    "input_rail_observation": "flat; no visible sag; no visible oscillation",
    "vdd_rail_observation": (
        "flat/tracking; no visible oscillation; exact dropout not quantified"
    ),
    "reset_mode": "stlink",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "List or update one row in the STM32 power sweep run matrix. "
            "This helper only reads and writes CSV metadata."
        )
    )
    parser.add_argument(
        "--matrix",
        type=Path,
        default=DEFAULT_MATRIX,
        help=f"run matrix path (default: {DEFAULT_MATRIX})",
    )
    parser.add_argument(
        "--list",
        action="store_true",
        help="list current runs and exit without modifying the matrix",
    )
    parser.add_argument(
        "--run-id",
        help="run_id to list or update; avoids prompting for the id after launch",
    )
    parser.add_argument(
        "--stage",
        choices=["pre_capture", "post_capture", "post_replay"],
        help="staged update to perform",
    )
    return parser.parse_args()


def read_matrix(path: Path) -> list[dict[str, str]]:
    if not path.exists():
        return []

    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        if reader.fieldnames != COLUMNS:
            expected = ",".join(COLUMNS)
            found = ",".join(reader.fieldnames or [])
            raise ValueError(
                "matrix header does not match fixed column order\n"
                f"expected: {expected}\n"
                f"found:    {found}"
            )
        return [{column: row.get(column, "") for column in COLUMNS} for row in reader]


def write_matrix(path: Path, rows: list[dict[str, str]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=COLUMNS, lineterminator="\n")
        writer.writeheader()
        for row in rows:
            writer.writerow({column: row.get(column, "") for column in COLUMNS})


def list_runs(rows: list[dict[str, str]], requested_run_id: str = "") -> None:
    if requested_run_id:
        rows = [row for row in rows if row["run_id"] == requested_run_id]
        if not rows:
            raise ValueError(f"run_id {requested_run_id!r} not found")

    writer = csv.writer(sys.stdout, lineterminator="\n")
    if not rows:
        print("No runs found.")
        return

    writer.writerow(["run_id", "label", "status", "classification", "artifact_path"])
    for row in rows:
        writer.writerow(
            [
                row["run_id"],
                row["label"],
                row["status"],
                row["classification"],
                row["artifact_path"],
            ]
        )


def local_measurement_time_default() -> tuple[str, str]:
    now = datetime.now().astimezone()
    return now.isoformat(timespec="minutes"), f"auto: {now:%H:%M}"


def prompt_text(
    field: str, current: str = "", default: str = "", default_label: str = ""
) -> str:
    fallback = current or default
    prompt_default = current or default_label or default
    suffix = f" [{prompt_default}]" if prompt_default else ""
    value = input(f"{field}{suffix}: ").strip()
    return value if value else fallback


def prompt_yes_no(question: str, default: bool = False) -> bool:
    marker = "Y/n" if default else "y/N"
    response = input(f"{question} [{marker}]: ").strip().lower()
    if not response:
        return default
    return response in {"y", "yes"}


def select_row(
    rows: list[dict[str, str]], run_id: str, stage: str
) -> tuple[dict[str, str], bool]:
    existing = next((row for row in rows if row["run_id"] == run_id), None)
    if existing is not None:
        print(f"Updating existing run_id {run_id!r}. Press Enter to keep defaults.")
        return dict(existing), False

    if stage != "pre_capture":
        raise ValueError(f"run_id {run_id!r} not found; run pre_capture first")
    if not prompt_yes_no(f"run_id {run_id!r} is not present; append it?"):
        raise RuntimeError("operator declined append")

    row = {column: "" for column in COLUMNS}
    row["run_id"] = run_id
    return row, True


def prompt_fields(
    row: dict[str, str],
    fields: list[str],
    defaults: dict[str, str],
    default_labels: dict[str, str] | None = None,
) -> None:
    labels = default_labels or {}
    for field in fields:
        row[field] = prompt_text(
            field,
            row.get(field, ""),
            defaults.get(field, ""),
            labels.get(field, ""),
        )


def apply_pre_capture(row: dict[str, str]) -> None:
    measurement_time, measurement_time_label = local_measurement_time_default()
    defaults = dict(DEFAULTS)
    defaults["status"] = "pre_capture"
    defaults["measurement_time"] = measurement_time
    defaults["settle_seconds"] = "10"
    if row["status"] in {"", "pending"}:
        row["status"] = ""

    prompt_fields(
        row,
        [
            "label",
            "status",
            "setpoint_v",
            "measured_v",
            "measured_current_ma",
            "power_source",
            "power_instrument",
            "measurement_time",
            "settle_seconds",
            "power_interaction",
            "input_rail_observation",
            "vdd_rail_observation",
            "supply_path",
            "reset_mode",
            "notes",
        ],
        defaults,
        {"measurement_time": measurement_time_label},
    )
    if not row["label"]:
        raise ValueError("label is required for pre_capture")


def apply_post_capture(row: dict[str, str]) -> None:
    prompt_fields(
        row,
        ["capture_state", "artifact_path", "notes"],
        {},
    )
    if not row["artifact_path"]:
        raise ValueError("artifact_path is required for post_capture")
    if row["status"] in {"", "pending", "pre_capture"}:
        row["status"] = "post_capture"


def apply_post_replay(row: dict[str, str]) -> None:
    prompt_fields(
        row,
        [
            "artifact_sha256",
            "replay_result",
            "first_divergence",
            "classification",
            "notes",
        ],
        {},
    )
    if row["classification"] and row["status"] in {
        "",
        "pending",
        "pre_capture",
        "post_capture",
    }:
        row["status"] = "complete"


def run_artifact_path(matrix: Path, row: dict[str, str]) -> Path:
    if not row["label"]:
        raise ValueError("label is required before printing the next command")
    runs_dir = matrix.parent / "runs"
    return runs_dir / f"{row['run_id']}_{row['label']}.bin"


def print_next_command(matrix: Path, row: dict[str, str], stage: str) -> None:
    if stage == "post_replay":
        return

    if stage == "pre_capture":
        path = run_artifact_path(matrix, row)
        print()
        print("Step 1: Ensure known-good firmware is flashed")
        print()
        print("Run:")
        print("make flash-ur")
        print("make flash-compare-ur")
        print()
        print("Step 2: Confirm board remains at intended voltage setpoint")
        print()
        print("Step 3: Run binary capture")
        print()
        print(
            "mkdir -p "
            "artifacts/physical_characterization/"
            "stm32_power_sweep_2026-05-12/runs"
        )
        print()
        print(
            'SERIAL=/dev/ttyACM0 PYTHONPATH="$PWD" \\\n'
            "python3 scripts/artifact_tool.py capture \\\n"
            "  --quick \\\n"
            "  --reset-context manual \\\n"
            f"  --out {path} \\\n"
            "  --signal-model phase8"
        )
        print()
        print("When capture command starts listening, manually reset the board.")
    elif stage == "post_capture":
        artifact_path = row["artifact_path"]
        print()
        print("Step 4: Verify binary artifact")
        print()
        print(
            'PYTHONPATH="$PWD" python3 scripts/artifact_tool.py verify '
            f"{artifact_path} --signal-model phase8"
        )
        print()
        print("Step 5: Hash binary artifact")
        print()
        print(
            'PYTHONPATH="$PWD" python3 scripts/artifact_tool.py hash '
            f"{artifact_path}"
        )


def apply_stage(row: dict[str, str], stage: str) -> None:
    if stage == "pre_capture":
        apply_pre_capture(row)
    elif stage == "post_capture":
        apply_post_capture(row)
    elif stage == "post_replay":
        apply_post_replay(row)
    else:
        raise ValueError(f"unsupported stage {stage!r}")


def update_rows(rows: list[dict[str, str]], row: dict[str, str], is_new: bool) -> None:
    if is_new:
        rows.append(row)
        return

    for index, existing in enumerate(rows):
        if existing["run_id"] == row["run_id"]:
            rows[index] = row
            return
    raise ValueError(f"run_id {row['run_id']!r} disappeared before update")


def main() -> int:
    args = parse_args()

    try:
        rows = read_matrix(args.matrix)
        if args.list:
            list_runs(rows, args.run_id or "")
            return 0

        if not args.run_id:
            raise ValueError("--run-id is required for staged updates")
        if not args.stage:
            raise ValueError("--stage is required unless --list is used")

        row, is_new = select_row(rows, args.run_id, args.stage)
        apply_stage(row, args.stage)
        update_rows(rows, row, is_new)
        write_matrix(args.matrix, rows)
    except (OSError, RuntimeError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1

    action = "appended" if is_new else "updated"
    print(f"{action} run_id {row['run_id']} in {args.matrix}")
    print_next_command(args.matrix, row, args.stage)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
