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
post_final_trigger_ack_count=0
capture_error_count=0
max_delta_ticks=8
max_delta_ns=88
result=PASS
capture_trigger=PB8_TIM4_CH3
capture_ack=PB9_TIM4_CH4
wiring_profile={wiring_profile}
measured_path=PB9_PA1_minus_PB8_PA6
"""


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
            "post_final_trigger_ack_count",
        ):
            if observer_meta[field] != 0:
                raise AssertionError(f"observer_profile: wrong {field}")

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
            .replace("post_final_trigger_ack_count=0\n", "post_final_trigger_ack_count=1\n")
            .replace("result=PASS\n", "result=FAIL\n")
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
