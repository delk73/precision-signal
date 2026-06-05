#!/usr/bin/env python3
"""Capture and validate a dual-STM32 replay witness report."""

from __future__ import annotations

import argparse
import json
import re
import sys
import time
from pathlib import Path

try:
    import serial
except ModuleNotFoundError:
    serial = None


PROFILE = "dual_stm32_replay_witness_v1"
RULE_ID = "dual_stm32_pa6_pa1_pair_v0"
ACTOR_EVENT_COUNT = 10_007
GENERATED_ARTIFACT_FILES = {"witness_report.txt", "meta.json", "wiring.txt"}
ALLOWED_CONTEXT_FILES = {"run_context.json", "notes.txt"}
REQUIRED_FIELDS = (
    "RESULT",
    "PROFILE",
    "RULE_ID",
    "ACTOR_EVENT_COUNT",
    "WITNESS_DIGEST",
    "FIRST_INVALID_EVENT",
    "ERROR",
    "ACTOR_BOARD",
    "WITNESS_BOARD",
    "ACTOR_FIRMWARE",
    "WITNESS_FIRMWARE",
    "EVENT_WINDOW",
)
STATIC_FIELDS = {
    "ACTOR_BOARD": "STM32F446RE",
    "WITNESS_BOARD": "STM32F446RE",
    "ACTOR_FIRMWARE": "sync_trigger_out+sync_trigger_in+sync_timing_capture",
    "WITNESS_FIRMWARE": "replay_witness_observer",
    "EVENT_WINDOW": "all_observed_events",
}
HEX64_RE = re.compile(r"^[0-9a-f]{16}$")
ERROR_RE = re.compile(r"^[A-Za-z0-9_+.-]+$")
WIRING_TEXT = """actor PA6/D12 -> witness PB8/TIM4_CH3
actor PA1/A1  -> witness PB9/TIM4_CH4
actor PA6/D12 -> actor PA0/A0
actor GND     -> witness GND
"""


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--serial")
    source.add_argument("--input")
    parser.add_argument("--out", required=True)
    parser.add_argument("--baud", type=int, default=115200)
    parser.add_argument("--timeout", type=float, default=20.0)
    parser.add_argument("--overwrite", action="store_true")
    return parser.parse_args(argv)


def decode_line(raw: bytes) -> str:
    return raw.decode("utf-8", errors="replace").strip()


def parse_report(text: str) -> dict[str, str]:
    lines = [line.strip() for line in text.splitlines() if line.strip()]
    fields: dict[str, str] = {}
    for line in lines:
        if ":" not in line:
            raise ValueError(f"malformed report line: {line!r}")
        key, value = line.split(":", 1)
        key = key.strip()
        value = value.strip()
        if not key or not value:
            raise ValueError(f"malformed report line: {line!r}")
        fields[key] = value

    missing = [key for key in REQUIRED_FIELDS if key not in fields]
    if missing:
        raise ValueError(f"missing required witness fields: {', '.join(missing)}")

    if fields["RESULT"] not in ("PASS", "FAIL"):
        raise ValueError(f"invalid RESULT: {fields['RESULT']!r}")
    if fields["PROFILE"] != PROFILE:
        raise ValueError(f"unknown PROFILE: {fields['PROFILE']!r}")
    if fields["RULE_ID"] != RULE_ID:
        raise ValueError(f"unknown RULE_ID: {fields['RULE_ID']!r}")
    for key, expected in STATIC_FIELDS.items():
        actual = fields[key]
        if actual != expected:
            raise ValueError(f"invalid {key}: expected {expected!r}, got {actual!r}")

    try:
        count = int(fields["ACTOR_EVENT_COUNT"], 10)
    except ValueError as exc:
        raise ValueError(f"malformed ACTOR_EVENT_COUNT: {fields['ACTOR_EVENT_COUNT']!r}") from exc
    if count != ACTOR_EVENT_COUNT:
        raise ValueError(
            f"invalid ACTOR_EVENT_COUNT: expected {ACTOR_EVENT_COUNT}, got {count}"
        )

    digest = fields["WITNESS_DIGEST"]
    digest_is_hex = bool(HEX64_RE.fullmatch(digest))
    if digest != "none" and not digest_is_hex:
        raise ValueError(f"malformed WITNESS_DIGEST: {digest!r}")

    first_invalid = fields["FIRST_INVALID_EVENT"]
    first_invalid_is_none = first_invalid == "none"
    if not first_invalid_is_none:
        try:
            parsed_first_invalid = int(first_invalid, 10)
        except ValueError as exc:
            raise ValueError(f"malformed FIRST_INVALID_EVENT: {first_invalid!r}") from exc
        if parsed_first_invalid < 0:
            raise ValueError(f"invalid FIRST_INVALID_EVENT: {parsed_first_invalid}")

    error = fields["ERROR"]
    error_is_none = error == "none"
    if not error_is_none and not ERROR_RE.fullmatch(error):
        raise ValueError(f"malformed ERROR: {error!r}")

    if fields["RESULT"] == "PASS":
        if not first_invalid_is_none:
            raise ValueError("inconsistent PASS: FIRST_INVALID_EVENT must be none")
        if not error_is_none:
            raise ValueError("inconsistent PASS: ERROR must be none")
        if not digest_is_hex:
            raise ValueError("inconsistent PASS: WITNESS_DIGEST must be 16-hex")
    else:
        if first_invalid_is_none and error_is_none:
            raise ValueError(
                "inconsistent FAIL: FIRST_INVALID_EVENT or ERROR must identify failure"
            )

    return fields


def report_complete(lines: list[str]) -> bool:
    try:
        parse_report("\n".join(lines) + "\n")
    except ValueError:
        return False
    return True


def capture_report(serial_path: str, baud: int, timeout: float) -> str:
    if serial is None:
        raise RuntimeError("pyserial is required for live witness capture")
    deadline = time.monotonic() + timeout
    lines: list[str] = []
    with serial.Serial(serial_path, baud, timeout=0.25) as ser:
        ser.reset_input_buffer()
        print("Witness listener active; reset/flash actor when ready", flush=True)
        while time.monotonic() < deadline:
            raw = ser.readline()
            if not raw:
                continue
            line = decode_line(raw)
            if not line:
                continue
            if not lines:
                if not line.startswith("RESULT:"):
                    continue
            lines.append(line)
            if report_complete(lines):
                return "\n".join(lines) + "\n"
    raise TimeoutError("timed out waiting for replay witness report")


def read_report(input_path: str) -> str:
    return Path(input_path).read_text(encoding="utf-8")


def validate_output_directory(out_dir: Path, overwrite: bool) -> None:
    if not out_dir.exists():
        return
    existing = {path.name for path in out_dir.iterdir()}
    if not existing or overwrite:
        return
    generated = sorted(existing & GENERATED_ARTIFACT_FILES)
    if generated:
        raise ValueError(
            "generated output already exists: "
            + ", ".join(generated)
            + f" in {out_dir}"
        )
    unexpected = sorted(existing - ALLOWED_CONTEXT_FILES)
    if unexpected:
        raise ValueError(
            "output directory has unexpected existing files: "
            + ", ".join(unexpected)
            + f" in {out_dir}"
        )


def write_artifact(out_dir: Path, report: str, fields: dict[str, str], overwrite: bool) -> None:
    validate_output_directory(out_dir, overwrite)
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "witness_report.txt").write_text(report, encoding="utf-8")
    (out_dir / "wiring.txt").write_text(WIRING_TEXT, encoding="utf-8")
    meta = {
        "artifact_kind": "hil_replay_witness",
        "retention": "non_retained_scratch",
        "profile": fields["PROFILE"],
        "rule_id": fields["RULE_ID"],
        "result": fields["RESULT"],
        "actor_event_count": int(fields["ACTOR_EVENT_COUNT"], 10),
        "witness_digest": fields["WITNESS_DIGEST"],
        "first_invalid_event": fields["FIRST_INVALID_EVENT"],
        "error": fields["ERROR"],
        "digest_tuple": "event_index/channel/timer_delta",
        "claim": (
            "A second STM32F446RE board observed the existing actor PA6/PA1 "
            "event path through PB8/PB9 and applied a fixed count/order/channel rule."
        ),
    }
    for key in STATIC_FIELDS:
        meta[key.lower()] = fields[key]
    (out_dir / "meta.json").write_text(
        json.dumps(meta, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        report = read_report(args.input) if args.input is not None else capture_report(
            args.serial, args.baud, args.timeout
        )
        fields = parse_report(report)
        write_artifact(Path(args.out), report, fields, args.overwrite)
    except (OSError, RuntimeError, TimeoutError, ValueError) as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        return 1
    print(f"wrote replay witness artifact: {args.out}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
