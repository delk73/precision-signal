#!/usr/bin/env python3

import argparse
import json
import sys
import time
from pathlib import Path

import serial


REPORT_SCHEMA = "SYNC_TIMING_CAPTURE_V1"
FEATURE_SET = "sync_trigger_out sync_trigger_in sync_timing_capture"
TIMER_HZ = 90_000_000
THRESHOLD_TICKS = 9
WIRING_PROFILE = "single_board_split_capture_v1"
MEASURED_PATH = "PB9_PA1_minus_PB8_PA6"
CAPTURE_TRIGGER = "PB8_TIM4_CH3"
CAPTURE_ACK = "PB9_TIM4_CH4"
REQUIRED_FIELDS = (
    "timer_hz",
    "threshold_ticks",
    "trigger_count",
    "ack_count",
    "missed_ack_count",
    "unexpected_ack_count",
    "capture_error_count",
    "max_delta_ticks",
    "max_delta_ns",
    "result",
)
EXPECTED_FIELDS = {
    "timer_hz": str(TIMER_HZ),
    "threshold_ticks": str(THRESHOLD_TICKS),
    "trigger_count": "10000",
    "capture_trigger": CAPTURE_TRIGGER,
    "capture_ack": CAPTURE_ACK,
    "wiring_profile": WIRING_PROFILE,
    "measured_path": MEASURED_PATH,
}
WIRING_TEXT = """PA6/D12 -> PA0/A0
PA6/D12 -> PB8/TIM4_CH3
PA1/A1  -> PB9/TIM4_CH4
GND shared
"""


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--serial", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--baud", type=int, default=115200)
    parser.add_argument("--timeout", type=float, default=20.0)
    parser.add_argument("--overwrite", action="store_true")
    return parser.parse_args()


def decode_line(raw: bytes) -> str:
    return raw.decode("utf-8", errors="replace").strip()


def parse_report(text: str) -> dict[str, str]:
    lines = [line.strip() for line in text.splitlines() if line.strip()]
    if not lines or lines[0] != REPORT_SCHEMA:
        raise ValueError(f"missing {REPORT_SCHEMA} header")

    fields: dict[str, str] = {}
    for line in lines[1:]:
        if "=" not in line:
            raise ValueError(f"malformed report line: {line!r}")
        key, value = line.split("=", 1)
        fields[key] = value

    missing = [key for key in REQUIRED_FIELDS if key not in fields]
    if missing:
        raise ValueError(f"missing required report fields: {', '.join(missing)}")

    for key, expected in EXPECTED_FIELDS.items():
        actual = fields.get(key)
        if actual != expected:
            raise ValueError(f"invalid {key}: expected {expected!r}, got {actual!r}")

    for key in (
        "timer_hz",
        "threshold_ticks",
        "trigger_count",
        "ack_count",
        "missed_ack_count",
        "unexpected_ack_count",
        "capture_error_count",
        "max_delta_ticks",
        "max_delta_ns",
    ):
        try:
            value = int(fields[key], 10)
        except ValueError as exc:
            raise ValueError(f"non-integer {key}: {fields[key]!r}") from exc
        if value < 0:
            raise ValueError(f"negative {key}: {value}")

    if fields["result"] not in ("PASS", "FAIL"):
        raise ValueError(f"invalid result: {fields['result']!r}")
    expected_result = (
        "PASS"
        if int(fields["missed_ack_count"], 10) == 0
        and int(fields["unexpected_ack_count"], 10) == 0
        and int(fields["capture_error_count"], 10) == 0
        and int(fields["max_delta_ticks"], 10) < THRESHOLD_TICKS
        else "FAIL"
    )
    if fields["result"] != expected_result:
        raise ValueError(
            f"inconsistent result: expected {expected_result!r}, got {fields['result']!r}"
        )

    return fields


def report_complete(lines: list[str]) -> bool:
    if not lines or lines[0] != REPORT_SCHEMA:
        return False
    try:
        fields = parse_report("\n".join(lines) + "\n")
    except ValueError:
        return False
    return fields.get("measured_path") == MEASURED_PATH


def capture_report(serial_path: str, baud: int, timeout: float) -> str:
    deadline = time.monotonic() + timeout
    lines: list[str] = []

    with serial.Serial(serial_path, baud, timeout=0.25) as ser:
        ser.reset_input_buffer()
        print("Listener active; reset the board if needed", flush=True)
        while time.monotonic() < deadline:
            raw = ser.readline()
            if not raw:
                continue

            line = decode_line(raw)
            if not line:
                continue
            if not lines:
                if line != REPORT_SCHEMA:
                    continue
                lines.append(line)
                continue

            lines.append(line)
            if report_complete(lines):
                return "\n".join(lines) + "\n"

    raise TimeoutError(f"timed out waiting for {REPORT_SCHEMA} report")


def write_artifact(
    out_dir: Path, report: str, fields: dict[str, str], overwrite: bool
) -> None:
    if out_dir.exists() and any(out_dir.iterdir()) and not overwrite:
        raise ValueError(f"output directory exists and is non-empty: {out_dir}")

    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "timing_report.txt").write_text(report, encoding="utf-8")
    (out_dir / "wiring.txt").write_text(WIRING_TEXT, encoding="utf-8")

    meta = {
        "artifact_kind": "hil_timing_capture",
        "schema": REPORT_SCHEMA,
        "feature_set": FEATURE_SET,
        "timer_hz": TIMER_HZ,
        "threshold_ticks": THRESHOLD_TICKS,
        "wiring_profile": WIRING_PROFILE,
        "measured_path": MEASURED_PATH,
        "capture_trigger": CAPTURE_TRIGGER,
        "capture_ack": CAPTURE_ACK,
        "trigger_output": "PA6_D12",
        "trigger_input": "PA0_A0",
        "ack_output": "PA1_A1",
        "trigger_count": int(fields["trigger_count"], 10),
        "ack_count": int(fields["ack_count"], 10),
        "missed_ack_count": int(fields["missed_ack_count"], 10),
        "unexpected_ack_count": int(fields["unexpected_ack_count"], 10),
        "capture_error_count": int(fields["capture_error_count"], 10),
        "max_delta_ticks": int(fields["max_delta_ticks"], 10),
        "max_delta_ns": int(fields["max_delta_ns"], 10),
        "result": fields["result"],
    }
    (out_dir / "meta.json").write_text(
        json.dumps(meta, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )


def main() -> int:
    args = parse_args()
    try:
        report = capture_report(args.serial, args.baud, args.timeout)
        fields = parse_report(report)
        write_artifact(Path(args.out), report, fields, args.overwrite)
    except (OSError, TimeoutError, ValueError) as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        return 1

    print(f"wrote HIL timing artifact: {args.out}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
