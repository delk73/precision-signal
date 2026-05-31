#!/usr/bin/env python3
"""Regression tests for HIL timing capture profile handling."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent


def run_capture(profile: str, report: str, out_dir: Path) -> subprocess.CompletedProcess[str]:
    report_path = out_dir.parent / f"{profile}.txt"
    report_path.write_text(report, encoding="utf-8")
    return subprocess.run(
        [
            "python3",
            "scripts/hil_timing_capture.py",
            "--profile",
            profile,
            "--input",
            str(report_path),
            "--out",
            str(out_dir),
        ],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )


def run_live_capture(profile: str, serial: str, out_dir: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            "python3",
            "scripts/hil_timing_capture.py",
            "--profile",
            profile,
            "--serial",
            serial,
            "--out",
            str(out_dir),
            "--timeout",
            "0.01",
        ],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )


def valid_report(wiring_profile: str) -> str:
    return f"""SYNC_TIMING_CAPTURE_V1
timer_hz=90000000
threshold_ticks=9
trigger_count=10000
ack_count=10000
missed_ack_count=0
unexpected_ack_count=0
pre_first_trigger_ack_count=0
in_window_unexpected_ack_count=0
first_in_window_unexpected_ack_trigger_count=0
last_in_window_unexpected_ack_trigger_count=0
post_final_trigger_ack_count=0
capture_error_count=0
max_delta_ticks=8
max_delta_ns=88
result=PASS
evidence_window_start_trigger_count=8
evidence_window_trigger_count=10000
evidence_window_ack_count=10000
evidence_window_unexpected_ack_count=0
evidence_window_missed_ack_count=0
evidence_window_capture_error_count=0
evidence_window_max_delta_ticks=8
evidence_window_max_delta_ns=88
evidence_window_result=PASS
capture_trigger=PB8_TIM4_CH3
capture_ack=PB9_TIM4_CH4
wiring_profile={wiring_profile}
measured_path=PB9_PA1_minus_PB8_PA6
"""


def report_with_diagnostics(wiring_profile: str) -> str:
    return valid_report(wiring_profile).replace(
        "measured_path=PB9_PA1_minus_PB8_PA6\n",
        """diagnostic_startup_trigger_input_level=0
diagnostic_startup_ack_input_level=1
diagnostic_capture_clear_attempted=1
diagnostic_capture_sr_before_clear=16
diagnostic_capture_sr_after_clear=0
diagnostic_capture_sr_after_arm=16
diagnostic_capture_event_pending_after_arm=1
diagnostic_capture_trigger_pending_after_arm=0
diagnostic_capture_ack_pending_after_arm=1
measured_path=PB9_PA1_minus_PB8_PA6
""",
    )


def valid_dual_board_context(confirmed: bool = True) -> dict[str, object]:
    return {
        "board_alias_confirmation": {
            "confirmed_from_dev_serial_by_id": confirmed,
            "required_mappings": {
                "066CFF505487525067182651": "actor / Board A",
                "0668FF514988525067215029": "observer / Board B",
            },
        },
        "board_aliases": {
            "actor": {
                "stlink_serial": "066CFF505487525067182651",
                "vcp_by_id": "/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_066CFF505487525067182651-if02",
                "firmware_features": "sync_trigger_out sync_trigger_in sync_timing_capture",
                "role": "external_actor",
            },
            "observer": {
                "stlink_serial": "0668FF514988525067215029",
                "vcp_by_id": "/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_0668FF514988525067215029-if02",
                "firmware_features": "sync_timing_observer",
                "role": "external_observer",
            },
        },
    }


def assert_ok(name: str, proc: subprocess.CompletedProcess[str]) -> None:
    if proc.returncode != 0:
        raise AssertionError(
            f"{name}: expected success, rc={proc.returncode}\n"
            f"stdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )


def assert_fail(name: str, proc: subprocess.CompletedProcess[str], needle: str) -> None:
    if proc.returncode == 0:
        raise AssertionError(f"{name}: expected failure")
    if needle not in proc.stderr:
        raise AssertionError(
            f"{name}: missing {needle!r}\nstdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )


def read_meta(out_dir: Path) -> dict[str, object]:
    return json.loads((out_dir / "meta.json").read_text(encoding="utf-8"))


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="dpw_hil_timing_") as tmp:
        root = Path(tmp)
        single_out = root / "single"
        proc = run_capture(
            "single_board_tim2_hardware_ack_v1",
            valid_report("single_board_split_capture_v1"),
            single_out,
        )
        assert_ok("single_board_profile", proc)
        single_meta = read_meta(single_out)
        if single_meta["feature_set"] != "sync_trigger_out sync_trigger_in sync_timing_capture":
            raise AssertionError("single_board_profile: wrong feature_set")
        if single_meta["wiring_profile"] != "single_board_split_capture_v1":
            raise AssertionError("single_board_profile: wrong wiring_profile")

        observer_out = root / "observer"
        proc = run_capture(
            "dual_edge_timing_observer_v1",
            valid_report("dual_edge_observer_v1"),
            observer_out,
        )
        assert_ok("observer_profile", proc)
        observer_meta = read_meta(observer_out)
        if observer_meta["feature_set"] != "sync_timing_observer":
            raise AssertionError("observer_profile: wrong feature_set")
        if observer_meta["wiring_profile"] != "dual_edge_observer_v1":
            raise AssertionError("observer_profile: wrong wiring_profile")
        if observer_meta["evidence_profile"] != "dual_edge_timing_observer_v1":
            raise AssertionError("observer_profile: wrong evidence_profile")
        if observer_meta["run_profile"] != "dual_board_observer":
            raise AssertionError("observer_profile: wrong run_profile")
        if observer_meta["measurement_path"]["trigger_capture"] != "observer_PB8_TIM4_CH3":
            raise AssertionError("observer_profile: wrong measurement_path")
        if "actor internal PA0-to-PA1 silicon latency" not in observer_meta["claim_boundary"][
            "does_not_prove"
        ]:
            raise AssertionError("observer_profile: missing claim boundary")
        for field in (
            "pre_first_trigger_ack_count",
            "in_window_unexpected_ack_count",
            "first_in_window_unexpected_ack_trigger_count",
            "last_in_window_unexpected_ack_trigger_count",
            "post_final_trigger_ack_count",
        ):
            if observer_meta[field] != 0:
                raise AssertionError(f"observer_profile: wrong {field}")

        diagnostics_out = root / "observer_diagnostics"
        proc = run_capture(
            "dual_edge_timing_observer_v1",
            report_with_diagnostics("dual_edge_observer_v1"),
            diagnostics_out,
        )
        assert_ok("observer_diagnostics", proc)
        diagnostics_meta = read_meta(diagnostics_out)
        expected_diagnostics = {
            "diagnostic_startup_trigger_input_level": 0,
            "diagnostic_startup_ack_input_level": 1,
            "diagnostic_capture_clear_attempted": 1,
            "diagnostic_capture_sr_before_clear": 16,
            "diagnostic_capture_sr_after_clear": 0,
            "diagnostic_capture_sr_after_arm": 16,
            "diagnostic_capture_event_pending_after_arm": 1,
            "diagnostic_capture_trigger_pending_after_arm": 0,
            "diagnostic_capture_ack_pending_after_arm": 1,
        }
        for field, expected in expected_diagnostics.items():
            if diagnostics_meta[field] != expected:
                raise AssertionError(f"observer_diagnostics: wrong {field}")

        boundary_out = root / "boundary_counts"
        boundary_report = (
            valid_report("dual_edge_observer_v1")
            .replace(
                "unexpected_ack_count=0\npre_first_trigger_ack_count=0\n",
                "unexpected_ack_count=3\npre_first_trigger_ack_count=1\n",
            )
            .replace(
                "in_window_unexpected_ack_count=0\n",
                "in_window_unexpected_ack_count=1\n",
            )
            .replace(
                "first_in_window_unexpected_ack_trigger_count=0\n",
                "first_in_window_unexpected_ack_trigger_count=42\n",
            )
            .replace(
                "last_in_window_unexpected_ack_trigger_count=0\n",
                "last_in_window_unexpected_ack_trigger_count=42\n",
            )
            .replace("post_final_trigger_ack_count=0\n", "post_final_trigger_ack_count=1\n")
            .replace("max_delta_ns=88\nresult=PASS\n", "max_delta_ns=88\nresult=FAIL\n")
        )
        proc = run_capture("dual_edge_timing_observer_v1", boundary_report, boundary_out)
        assert_ok("accepts_boundary_unexpected_ack_counts", proc)
        boundary_meta = read_meta(boundary_out)
        if boundary_meta["unexpected_ack_count"] != 3:
            raise AssertionError("accepts_boundary_unexpected_ack_counts: wrong total")
        if boundary_meta["pre_first_trigger_ack_count"] != 1:
            raise AssertionError("accepts_boundary_unexpected_ack_counts: wrong pre count")
        if boundary_meta["in_window_unexpected_ack_count"] != 1:
            raise AssertionError("accepts_boundary_unexpected_ack_counts: wrong in-window count")
        if boundary_meta["first_in_window_unexpected_ack_trigger_count"] != 42:
            raise AssertionError("accepts_boundary_unexpected_ack_counts: wrong first position")
        if boundary_meta["last_in_window_unexpected_ack_trigger_count"] != 42:
            raise AssertionError("accepts_boundary_unexpected_ack_counts: wrong last position")
        if boundary_meta["post_final_trigger_ack_count"] != 1:
            raise AssertionError("accepts_boundary_unexpected_ack_counts: wrong post count")

        bad_boundary_out = root / "bad_boundary_counts"
        bad_boundary_report = boundary_report.replace(
            "post_final_trigger_ack_count=1\n", "post_final_trigger_ack_count=0\n"
        )
        proc = run_capture(
            "dual_edge_timing_observer_v1",
            bad_boundary_report,
            bad_boundary_out,
        )
        assert_fail(
            "rejects_inconsistent_boundary_unexpected_ack_counts",
            proc,
            "inconsistent unexpected_ack_count",
        )

        zero_in_window_nonzero_position_out = root / "zero_in_window_nonzero_position"
        zero_in_window_nonzero_position_report = valid_report("dual_edge_observer_v1").replace(
            "first_in_window_unexpected_ack_trigger_count=0\n",
            "first_in_window_unexpected_ack_trigger_count=1\n",
        )
        proc = run_capture(
            "dual_edge_timing_observer_v1",
            zero_in_window_nonzero_position_report,
            zero_in_window_nonzero_position_out,
        )
        assert_fail(
            "rejects_zero_in_window_count_with_nonzero_position",
            proc,
            "positions must be zero",
        )

        in_window_missing_first_out = root / "in_window_missing_first"
        in_window_missing_first_report = boundary_report.replace(
            "first_in_window_unexpected_ack_trigger_count=42\n",
            "first_in_window_unexpected_ack_trigger_count=0\n",
        )
        proc = run_capture(
            "dual_edge_timing_observer_v1",
            in_window_missing_first_report,
            in_window_missing_first_out,
        )
        assert_fail(
            "rejects_in_window_count_with_zero_first_position",
            proc,
            "positions must be nonzero",
        )

        in_window_missing_last_out = root / "in_window_missing_last"
        in_window_missing_last_report = boundary_report.replace(
            "last_in_window_unexpected_ack_trigger_count=42\n",
            "last_in_window_unexpected_ack_trigger_count=0\n",
        )
        proc = run_capture(
            "dual_edge_timing_observer_v1",
            in_window_missing_last_report,
            in_window_missing_last_out,
        )
        assert_fail(
            "rejects_in_window_count_with_zero_last_position",
            proc,
            "positions must be nonzero",
        )

        first_after_last_out = root / "first_after_last"
        first_after_last_report = boundary_report.replace(
            "first_in_window_unexpected_ack_trigger_count=42\n",
            "first_in_window_unexpected_ack_trigger_count=43\n",
        )
        proc = run_capture(
            "dual_edge_timing_observer_v1",
            first_after_last_report,
            first_after_last_out,
        )
        assert_fail(
            "rejects_first_in_window_position_after_last",
            proc,
            "first 43 > last 42",
        )

        startup_unexpected_evidence_pass_out = root / "startup_unexpected_evidence_pass"
        startup_unexpected_evidence_pass_report = (
            valid_report("dual_edge_observer_v1")
            .replace(
                "unexpected_ack_count=0\npre_first_trigger_ack_count=0\n",
                "unexpected_ack_count=1\npre_first_trigger_ack_count=1\n",
            )
            .replace("max_delta_ns=88\nresult=PASS\n", "max_delta_ns=88\nresult=FAIL\n")
        )
        proc = run_capture(
            "dual_edge_timing_observer_v1",
            startup_unexpected_evidence_pass_report,
            startup_unexpected_evidence_pass_out,
        )
        assert_ok("accepts_startup_unexpected_ack_with_evidence_window_pass", proc)
        startup_meta = read_meta(startup_unexpected_evidence_pass_out)
        if startup_meta["result"] != "FAIL":
            raise AssertionError("startup_unexpected_evidence_pass: wrong raw result")
        if startup_meta["evidence_window_result"] != "PASS":
            raise AssertionError("startup_unexpected_evidence_pass: wrong evidence result")

        trigger5_unexpected_evidence_pass_out = root / "trigger5_unexpected_evidence_pass"
        trigger5_unexpected_evidence_pass_report = (
            valid_report("dual_edge_observer_v1")
            .replace(
                "unexpected_ack_count=0\npre_first_trigger_ack_count=0\n",
                "unexpected_ack_count=1\npre_first_trigger_ack_count=0\n",
            )
            .replace(
                "in_window_unexpected_ack_count=0\n",
                "in_window_unexpected_ack_count=1\n",
            )
            .replace(
                "first_in_window_unexpected_ack_trigger_count=0\n",
                "first_in_window_unexpected_ack_trigger_count=5\n",
            )
            .replace(
                "last_in_window_unexpected_ack_trigger_count=0\n",
                "last_in_window_unexpected_ack_trigger_count=5\n",
            )
            .replace("max_delta_ns=88\nresult=PASS\n", "max_delta_ns=88\nresult=FAIL\n")
        )
        proc = run_capture(
            "dual_edge_timing_observer_v1",
            trigger5_unexpected_evidence_pass_report,
            trigger5_unexpected_evidence_pass_out,
        )
        assert_ok("accepts_trigger5_unexpected_ack_with_evidence_window_pass", proc)
        trigger5_meta = read_meta(trigger5_unexpected_evidence_pass_out)
        if trigger5_meta["evidence_window_result"] != "PASS":
            raise AssertionError("trigger5_unexpected_evidence_pass: wrong evidence result")
        if trigger5_meta["first_in_window_unexpected_ack_trigger_count"] != 5:
            raise AssertionError("trigger5_unexpected_evidence_pass: wrong first position")

        evidence_unexpected_out = root / "evidence_unexpected"
        evidence_unexpected_report = (
            trigger5_unexpected_evidence_pass_report
            .replace(
                "first_in_window_unexpected_ack_trigger_count=5\n",
                "first_in_window_unexpected_ack_trigger_count=8\n",
            )
            .replace(
                "last_in_window_unexpected_ack_trigger_count=5\n",
                "last_in_window_unexpected_ack_trigger_count=8\n",
            )
            .replace(
                "evidence_window_ack_count=10000\n",
                "evidence_window_ack_count=10001\n",
            )
            .replace(
                "evidence_window_unexpected_ack_count=0\n",
                "evidence_window_unexpected_ack_count=1\n",
            )
            .replace("evidence_window_result=PASS\n", "evidence_window_result=FAIL\n")
        )
        proc = run_capture("dual_edge_timing_observer_v1", evidence_unexpected_report, evidence_unexpected_out)
        assert_ok("accepts_evidence_window_unexpected_ack_fail", proc)

        evidence_delta_fail_out = root / "evidence_delta_fail"
        evidence_delta_fail_report = (
            valid_report("dual_edge_observer_v1")
            .replace("evidence_window_max_delta_ticks=8\n", "evidence_window_max_delta_ticks=10\n")
            .replace("evidence_window_result=PASS\n", "evidence_window_result=FAIL\n")
        )
        proc = run_capture("dual_edge_timing_observer_v1", evidence_delta_fail_report, evidence_delta_fail_out)
        assert_ok("accepts_evidence_window_delta_fail", proc)

        evidence_trigger_count_fail_out = root / "evidence_trigger_count_fail"
        evidence_trigger_count_fail_report = (
            valid_report("dual_edge_observer_v1")
            .replace("evidence_window_trigger_count=10000\n", "evidence_window_trigger_count=9999\n")
            .replace("evidence_window_result=PASS\n", "evidence_window_result=FAIL\n")
        )
        proc = run_capture(
            "dual_edge_timing_observer_v1",
            evidence_trigger_count_fail_report,
            evidence_trigger_count_fail_out,
        )
        assert_ok("accepts_evidence_window_trigger_count_fail", proc)

        evidence_ack_count_fail_out = root / "evidence_ack_count_fail"
        evidence_ack_count_fail_report = (
            valid_report("dual_edge_observer_v1")
            .replace("evidence_window_ack_count=10000\n", "evidence_window_ack_count=9999\n")
            .replace("evidence_window_result=PASS\n", "evidence_window_result=FAIL\n")
        )
        proc = run_capture(
            "dual_edge_timing_observer_v1",
            evidence_ack_count_fail_report,
            evidence_ack_count_fail_out,
        )
        assert_ok("accepts_evidence_window_ack_count_fail", proc)

        missing_evidence_out = root / "missing_evidence_fields"
        missing_evidence_report = valid_report("dual_edge_observer_v1").replace(
            "evidence_window_result=PASS\n", ""
        )
        proc = run_capture("dual_edge_timing_observer_v1", missing_evidence_report, missing_evidence_out)
        assert_fail("rejects_missing_evidence_window_fields", proc, "missing required report fields")

        inconsistent_evidence_out = root / "inconsistent_evidence_result"
        inconsistent_evidence_report = valid_report("dual_edge_observer_v1").replace(
            "evidence_window_unexpected_ack_count=0\n",
            "evidence_window_unexpected_ack_count=1\n",
        )
        proc = run_capture(
            "dual_edge_timing_observer_v1",
            inconsistent_evidence_report,
            inconsistent_evidence_out,
        )
        assert_fail(
            "rejects_inconsistent_evidence_window_result",
            proc,
            "inconsistent evidence_window_result",
        )

        context_out = root / "context"
        context_out.mkdir()
        (context_out / "run_context.json").write_text("{}\n", encoding="utf-8")
        (context_out / "notes.txt").write_text("manual notes\n", encoding="utf-8")
        proc = run_capture(
            "dual_edge_timing_observer_v1",
            valid_report("dual_edge_observer_v1"),
            context_out,
        )
        assert_ok("allows_manual_context_files", proc)
        if not (context_out / "run_context.json").is_file():
            raise AssertionError("allows_manual_context_files: run_context.json removed")
        if not (context_out / "notes.txt").is_file():
            raise AssertionError("allows_manual_context_files: notes.txt removed")

        generated_out = root / "generated_exists"
        generated_out.mkdir()
        (generated_out / "timing_report.txt").write_text("old report\n", encoding="utf-8")
        proc = run_capture(
            "dual_edge_timing_observer_v1",
            valid_report("dual_edge_observer_v1"),
            generated_out,
        )
        assert_fail("rejects_existing_generated_output", proc, "generated output already exists")

        unexpected_out = root / "unexpected_exists"
        unexpected_out.mkdir()
        (unexpected_out / "operator.log").write_text("extra\n", encoding="utf-8")
        proc = run_capture(
            "dual_edge_timing_observer_v1",
            valid_report("dual_edge_observer_v1"),
            unexpected_out,
        )
        assert_fail(
            "rejects_unexpected_existing_file",
            proc,
            "output directory has unexpected existing files",
        )

        bad_out = root / "bad_observer"
        proc = run_capture(
            "dual_edge_timing_observer_v1",
            valid_report("single_board_split_capture_v1"),
            bad_out,
        )
        assert_fail("observer_rejects_single_board_wiring", proc, "invalid wiring_profile")

        bad_single_out = root / "bad_single"
        proc = run_capture(
            "single_board_tim2_hardware_ack_v1",
            valid_report("dual_edge_observer_v1"),
            bad_single_out,
        )
        assert_fail("single_board_rejects_observer_wiring", proc, "invalid wiring_profile")

        missing_context_out = root / "missing_context"
        proc = run_live_capture(
            "dual_edge_timing_observer_v1",
            "/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_0668FF514988525067215029-if02",
            missing_context_out,
        )
        assert_fail(
            "live_observer_requires_run_context",
            proc,
            "missing required dual-board run context",
        )

        unconfirmed_out = root / "unconfirmed_context"
        unconfirmed_out.mkdir()
        (unconfirmed_out / "run_context.json").write_text(
            json.dumps(valid_dual_board_context(confirmed=False)) + "\n",
            encoding="utf-8",
        )
        proc = run_live_capture(
            "dual_edge_timing_observer_v1",
            "/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_0668FF514988525067215029-if02",
            unconfirmed_out,
        )
        assert_fail(
            "live_observer_requires_alias_confirmation",
            proc,
            "confirmed from /dev/serial/by-id/",
        )

        missing_role_out = root / "missing_role_context"
        missing_role_out.mkdir()
        context = valid_dual_board_context()
        del context["board_aliases"]["observer"]["role"]
        (missing_role_out / "run_context.json").write_text(
            json.dumps(context) + "\n",
            encoding="utf-8",
        )
        proc = run_live_capture(
            "dual_edge_timing_observer_v1",
            "/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_0668FF514988525067215029-if02",
            missing_role_out,
        )
        assert_fail(
            "live_observer_requires_alias_role",
            proc,
            "missing observer fields: role",
        )

        stale_role_out = root / "stale_role_context"
        stale_role_out.mkdir()
        context = valid_dual_board_context()
        context["board_aliases"]["observer"]["role"] = "external_actor"
        (stale_role_out / "run_context.json").write_text(
            json.dumps(context) + "\n",
            encoding="utf-8",
        )
        proc = run_live_capture(
            "dual_edge_timing_observer_v1",
            "/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_0668FF514988525067215029-if02",
            stale_role_out,
        )
        assert_fail(
            "live_observer_rejects_stale_alias_role",
            proc,
            "invalid observer.role",
        )

        missing_mappings_out = root / "missing_mappings_context"
        missing_mappings_out.mkdir()
        context = valid_dual_board_context()
        del context["board_alias_confirmation"]["required_mappings"]
        (missing_mappings_out / "run_context.json").write_text(
            json.dumps(context) + "\n",
            encoding="utf-8",
        )
        proc = run_live_capture(
            "dual_edge_timing_observer_v1",
            "/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_0668FF514988525067215029-if02",
            missing_mappings_out,
        )
        assert_fail(
            "live_observer_requires_confirmation_mappings",
            proc,
            "board_alias_confirmation.required_mappings",
        )

        stale_mappings_out = root / "stale_mappings_context"
        stale_mappings_out.mkdir()
        context = valid_dual_board_context()
        context["board_alias_confirmation"]["required_mappings"][
            "0668FF514988525067215029"
        ] = "actor / Board A"
        (stale_mappings_out / "run_context.json").write_text(
            json.dumps(context) + "\n",
            encoding="utf-8",
        )
        proc = run_live_capture(
            "dual_edge_timing_observer_v1",
            "/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_0668FF514988525067215029-if02",
            stale_mappings_out,
        )
        assert_fail(
            "live_observer_rejects_stale_confirmation_mappings",
            proc,
            "confirmation mapping mismatch",
        )

        mismatched_serial_out = root / "mismatched_serial_context"
        mismatched_serial_out.mkdir()
        (mismatched_serial_out / "run_context.json").write_text(
            json.dumps(valid_dual_board_context()) + "\n",
            encoding="utf-8",
        )
        proc = run_live_capture(
            "dual_edge_timing_observer_v1",
            "/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_066CFF505487525067182651-if02",
            mismatched_serial_out,
        )
        assert_fail(
            "live_observer_requires_observer_serial",
            proc,
            "must match board_aliases.observer.vcp_by_id",
        )

    print("PASS: HIL timing capture profile regression suite")
    return 0


if __name__ == "__main__":
    sys.exit(main())
