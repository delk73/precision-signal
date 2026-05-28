#!/usr/bin/env python3

import argparse
import json
import sys
import time
from pathlib import Path

import serial


REPORT_SCHEMA = "SYNC_TIMING_CAPTURE_V1"
TIMER_HZ = 90_000_000
THRESHOLD_TICKS = 9
GENERATED_ARTIFACT_FILES = {"timing_report.txt", "meta.json", "wiring.txt"}
ALLOWED_CONTEXT_FILES = {"run_context.json", "notes.txt"}
PROFILE_DEFINITIONS = {
    "single_board_tim2_hardware_ack_v1": {
        "evidence_profile": "single_board_tim2_hardware_ack_v1",
        "run_profile": "tim2_hardware_ack",
        "feature_set": "sync_trigger_out sync_trigger_in sync_timing_capture",
        "wiring_profile": "single_board_split_capture_v1",
        "measured_path": "PB9_PA1_minus_PB8_PA6",
        "capture_trigger": "PB8_TIM4_CH3",
        "capture_ack": "PB9_TIM4_CH4",
        "trigger_output": "PA6_D12",
        "trigger_input": "PA0_A0",
        "ack_output": "PA1_A1",
        "functional_path": {
            "trigger_output": "PA6_D12",
            "trigger_input": "PA0_A0",
            "ack_mechanism": "tim2_hardware_output_compare",
            "ack_output": "PA1_A1",
        },
        "measurement_path": {
            "trigger_capture": "PB8_TIM4_CH3",
            "ack_capture": "PB9_TIM4_CH4",
            "measured_delta": "ack_capture_minus_trigger_capture",
        },
        "claim_boundary": {
            "proves": "selected TIM2 hardware acknowledgment path split-capture timing",
            "does_not_prove": [
                "software EXTI acknowledgment path timing pass",
                "exact internal PA0-to-PA1 silicon latency",
            ],
        },
        "wiring_text": """PA6/D12 -> PA0/A0
PA6/D12 -> PB8/TIM4_CH3
PA1/A1  -> PB9/TIM4_CH4
GND shared
""",
    },
    "dual_edge_timing_observer_v1": {
        "evidence_profile": "dual_edge_timing_observer_v1",
        "run_profile": "dual_board_observer",
        "feature_set": "sync_timing_observer",
        "wiring_profile": "dual_edge_observer_v1",
        "measured_path": "PB9_PA1_minus_PB8_PA6",
        "capture_trigger": "PB8_TIM4_CH3",
        "capture_ack": "PB9_TIM4_CH4",
        "trigger_output": "external_actor_PA6_D12",
        "trigger_input": "external_actor_PA0_A0_or_profile_defined",
        "ack_output": "external_actor_PA1_A1",
        "functional_path": {
            "trigger_output": "external_actor_PA6_D12",
            "trigger_input": "external_actor_PA0_A0_or_profile_defined",
            "ack_mechanism": "external_actor_defined",
            "ack_output": "external_actor_PA1_A1",
        },
        "measurement_path": {
            "trigger_capture": "observer_PB8_TIM4_CH3",
            "ack_capture": "observer_PB9_TIM4_CH4",
            "measured_delta": "ack_capture_minus_trigger_capture",
        },
        "claim_boundary": {
            "proves": (
                "observer board measured external actor trigger-to-ack timing "
                "through PB8/PB9"
            ),
            "does_not_prove": [
                "actor internal PA0-to-PA1 silicon latency",
                "software EXTI acknowledgment path timing pass unless actor profile states it",
                "platform proof",
                "RPL0/replay authority",
                "release evidence",
            ],
        },
        "wiring_text": """external actor trigger edge -> observer PB8/TIM4_CH3
external actor ack edge     -> observer PB9/TIM4_CH4
GND shared
""",
    },
}
DUAL_BOARD_PROFILES = {"dual_edge_timing_observer_v1"}
REQUIRED_BOARD_ALIAS_FIELDS = ("stlink_serial", "vcp_by_id", "firmware_features", "role")
EXPECTED_BOARD_ALIAS_ROLES = {
    "actor": "external_actor",
    "observer": "external_observer",
}
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


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--profile",
        metavar="PROFILE",
        help=(
            "required timing evidence profile to apply to the retained artifact; "
            f"supported profile: {supported_profile_names()}"
        ),
    )
    source = parser.add_mutually_exclusive_group()
    source.add_argument("--serial")
    source.add_argument("--input")
    parser.add_argument("--out", required=True)
    parser.add_argument("--baud", type=int, default=115200)
    parser.add_argument("--timeout", type=float, default=20.0)
    parser.add_argument("--overwrite", action="store_true")
    args = parser.parse_args()
    if args.profile is None:
        parser.error(
            "--profile is required; supported profile: "
            f"{supported_profile_names()}"
        )
    if args.profile not in PROFILE_DEFINITIONS:
        parser.error(
            f"unsupported profile: {args.profile}; supported profile: "
            f"{supported_profile_names()}"
        )
    if args.serial is None and args.input is None:
        parser.error("exactly one of --input or --serial is required")
    return args


def supported_profile_names() -> str:
    return ", ".join(sorted(PROFILE_DEFINITIONS))


def decode_line(raw: bytes) -> str:
    return raw.decode("utf-8", errors="replace").strip()


def expected_fields(profile: dict[str, object]) -> dict[str, str]:
    return {
        "timer_hz": str(TIMER_HZ),
        "threshold_ticks": str(THRESHOLD_TICKS),
        "trigger_count": "10000",
        "capture_trigger": str(profile["capture_trigger"]),
        "capture_ack": str(profile["capture_ack"]),
        "wiring_profile": str(profile["wiring_profile"]),
        "measured_path": str(profile["measured_path"]),
    }


def parse_report(text: str, profile: dict[str, object]) -> dict[str, str]:
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

    for key, expected in expected_fields(profile).items():
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


def report_complete(lines: list[str], profile: dict[str, object]) -> bool:
    if not lines or lines[0] != REPORT_SCHEMA:
        return False
    try:
        fields = parse_report("\n".join(lines) + "\n", profile)
    except ValueError:
        return False
    return fields.get("measured_path") == profile["measured_path"]


def capture_report(
    serial_path: str, baud: int, timeout: float, profile: dict[str, object]
) -> str:
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
            if report_complete(lines, profile):
                return "\n".join(lines) + "\n"

    raise TimeoutError(f"timed out waiting for {REPORT_SCHEMA} report")


def read_report(input_path: str) -> str:
    return Path(input_path).read_text(encoding="utf-8")


def validate_dual_board_run_context(
    out_dir: Path, serial_path: str, profile_name: str
) -> None:
    if profile_name not in DUAL_BOARD_PROFILES:
        return

    context_path = out_dir / "run_context.json"
    try:
        context = json.loads(context_path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise ValueError(f"missing required dual-board run context: {context_path}") from exc
    except json.JSONDecodeError as exc:
        raise ValueError(f"malformed dual-board run context: {context_path}: {exc}") from exc

    aliases = context.get("board_aliases")
    if not isinstance(aliases, dict):
        raise ValueError("dual-board run context missing board_aliases")

    for alias in ("actor", "observer"):
        entry = aliases.get(alias)
        if not isinstance(entry, dict):
            raise ValueError(f"dual-board run context missing board_aliases.{alias}")
        missing = [
            field
            for field in REQUIRED_BOARD_ALIAS_FIELDS
            if not isinstance(entry.get(field), str) or not entry[field]
        ]
        if missing:
            raise ValueError(
                f"dual-board run context missing {alias} fields: {', '.join(missing)}"
            )
        expected_role = EXPECTED_BOARD_ALIAS_ROLES[alias]
        actual_role = entry["role"]
        if actual_role != expected_role:
            raise ValueError(
                f"dual-board run context invalid {alias}.role: "
                f"expected {expected_role!r}, got {actual_role!r}"
            )

    observer_serial = aliases["observer"]["vcp_by_id"]
    if serial_path != observer_serial:
        raise ValueError(
            "dual-board capture serial must match board_aliases.observer.vcp_by_id: "
            f"expected {observer_serial!r}, got {serial_path!r}"
        )

    confirmation = context.get("board_alias_confirmation")
    if not isinstance(confirmation, dict) or not confirmation.get(
        "confirmed_from_dev_serial_by_id"
    ):
        raise ValueError(
            "dual-board aliases must be confirmed from /dev/serial/by-id/ before capture"
        )

    required_mappings = confirmation.get("required_mappings")
    if not isinstance(required_mappings, dict):
        raise ValueError(
            "dual-board run context missing board_alias_confirmation.required_mappings"
        )

    expected_mappings = {
        aliases["actor"]["stlink_serial"]: "actor / Board A",
        aliases["observer"]["stlink_serial"]: "observer / Board B",
    }
    for stlink_serial, expected_alias in expected_mappings.items():
        actual_alias = required_mappings.get(stlink_serial)
        if actual_alias != expected_alias:
            raise ValueError(
                "dual-board confirmation mapping mismatch: "
                f"{stlink_serial} must map to {expected_alias!r}, got {actual_alias!r}"
            )


def write_artifact(
    out_dir: Path,
    report: str,
    fields: dict[str, str],
    profile: dict[str, object],
    overwrite: bool,
) -> None:
    validate_output_directory(out_dir, overwrite)

    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "timing_report.txt").write_text(report, encoding="utf-8")
    (out_dir / "wiring.txt").write_text(str(profile["wiring_text"]), encoding="utf-8")

    meta = {
        "artifact_kind": "hil_timing_capture",
        "schema": REPORT_SCHEMA,
        "feature_set": profile["feature_set"],
        "timer_hz": TIMER_HZ,
        "threshold_ticks": THRESHOLD_TICKS,
        "wiring_profile": profile["wiring_profile"],
        "run_profile": profile["run_profile"],
        "evidence_profile": profile["evidence_profile"],
        "measured_path": profile["measured_path"],
        "capture_trigger": profile["capture_trigger"],
        "capture_ack": profile["capture_ack"],
        "trigger_output": profile["trigger_output"],
        "trigger_input": profile["trigger_input"],
        "ack_output": profile["ack_output"],
        "functional_path": profile["functional_path"],
        "measurement_path": profile["measurement_path"],
        "claim_boundary": profile["claim_boundary"],
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


def main() -> int:
    args = parse_args()
    profile = PROFILE_DEFINITIONS[args.profile]
    try:
        if args.input is not None:
            report = read_report(args.input)
        else:
            validate_dual_board_run_context(Path(args.out), args.serial, args.profile)
            report = capture_report(args.serial, args.baud, args.timeout, profile)
        fields = parse_report(report, profile)
        write_artifact(Path(args.out), report, fields, profile, args.overwrite)
    except (OSError, TimeoutError, ValueError) as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        return 1

    print(f"wrote HIL timing artifact: {args.out}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
