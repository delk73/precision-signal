#!/usr/bin/env python3
"""Regression tests for the dual-board replay witness runner."""

from __future__ import annotations

import contextlib
import io
import json
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import hil_dual_replay_witness_run as runner

ACTOR_STLINK = "066CFF505487525067182651"
WITNESS_STLINK = "0668FF514988525067215029"
WITNESS_VCP = "/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_0668FF514988525067215029-if02"

PASS_REPORT = """RESULT: PASS
PROFILE: dual_stm32_replay_witness_v1
RULE_ID: dual_stm32_pa6_pa1_pair_v0
ACTOR_EVENT_COUNT: 10007
WITNESS_DIGEST: 0123456789abcdef
FIRST_INVALID_EVENT: none
ERROR: none
ACTOR_BOARD: STM32F446RE
WITNESS_BOARD: STM32F446RE
ACTOR_FIRMWARE: sync_trigger_out+sync_trigger_in+sync_timing_capture
WITNESS_FIRMWARE: replay_witness_observer
EVENT_WINDOW: all_observed_events
"""

FAIL_REPORT = PASS_REPORT.replace("RESULT: PASS", "RESULT: FAIL").replace(
    "FIRST_INVALID_EVENT: none", "FIRST_INVALID_EVENT: 7"
).replace("ERROR: none", "ERROR: rule_mismatch")

MALFORMED_REPORT = PASS_REPORT.replace("RULE_ID: dual_stm32_pa6_pa1_pair_v0\n", "")


def valid_context() -> dict[str, object]:
    return {
        "board_alias_confirmation": {
            "confirmed_from_dev_serial_by_id": True,
            "required_mappings": {
                ACTOR_STLINK: "actor / Board A",
                WITNESS_STLINK: "observer / Board B",
            },
        },
        "board_aliases": {
            "actor": {
                "stlink_serial": ACTOR_STLINK,
                "vcp_by_id": "/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_066CFF505487525067182651-if02",
                "firmware_features": "sync_trigger_out sync_trigger_in sync_timing_capture",
                "role": "external_actor",
            },
            "observer": {
                "stlink_serial": WITNESS_STLINK,
                "vcp_by_id": WITNESS_VCP,
                "firmware_features": "replay_witness_observer",
                "role": "external_observer",
            },
        },
    }


def write_context(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(valid_context()) + "\n", encoding="utf-8")


class FakeCaptureProc:
    def __init__(self, harness: "Harness", out_dir: Path) -> None:
        self.harness = harness
        self.out_dir = out_dir
        self.returncode: int | None = None
        self.terminated = False

    def poll(self) -> int | None:
        return self.returncode

    def wait(self, timeout: float | None = None) -> int:
        self.harness.events.append(("capture-wait", timeout))
        if self.returncode is None:
            if self.harness.capture_writes_report:
                self.out_dir.mkdir(parents=True, exist_ok=True)
                (self.out_dir / "witness_report.txt").write_text(
                    self.harness.report, encoding="utf-8"
                )
                (self.out_dir / "meta.json").write_text("{}\n", encoding="utf-8")
                (self.out_dir / "wiring.txt").write_text("wiring\n", encoding="utf-8")
            self.returncode = self.harness.capture_returncode
        return self.returncode

    def terminate(self) -> None:
        self.harness.events.append(("capture-terminate", None))
        self.terminated = True
        if self.returncode is None:
            self.returncode = -15


class Harness:
    def __init__(self) -> None:
        self.events: list[tuple[str, object]] = []
        self.report = PASS_REPORT
        self.capture_returncode = 0
        self.capture_writes_report = True
        self._orig_run_flash = runner.run_flash
        self._orig_require_flash_identity = runner.require_flash_identity
        self._orig_require_observer_vcp = runner.require_observer_vcp_for_capture
        self._orig_start_capture = runner.start_capture

    def __enter__(self) -> "Harness":
        def fake_run_flash(make_cmd: str, phase: str, role: str, alias: dict[str, str], features: str) -> None:
            self.events.append(("flash", {"phase": phase, "role": role, "features": features, "serial": alias["stlink_serial"]}))

        def fake_require_flash_identity(role: str, phase: str, alias: dict[str, str]) -> str:
            self.events.append(("flash-ready", (role, phase)))
            return alias["stlink_serial"]

        def fake_require_observer_vcp(vcp: str) -> str:
            self.events.append(("vcp-ready", vcp))
            return vcp

        def fake_start_capture(
            out_dir: Path,
            witness_vcp: str,
            baud: int,
            timeout: float,
            overwrite: bool,
            retention: str,
        ) -> FakeCaptureProc:
            self.events.append(
                (
                    "capture-start",
                    {
                        "out": out_dir,
                        "vcp": witness_vcp,
                        "overwrite": overwrite,
                        "retention": retention,
                    },
                )
            )
            return FakeCaptureProc(self, out_dir)

        runner.run_flash = fake_run_flash
        runner.require_flash_identity = fake_require_flash_identity
        runner.require_observer_vcp_for_capture = fake_require_observer_vcp
        runner.start_capture = fake_start_capture
        return self

    def __exit__(self, exc_type: object, exc: object, tb: object) -> None:
        runner.run_flash = self._orig_run_flash
        runner.require_flash_identity = self._orig_require_flash_identity
        runner.require_observer_vcp_for_capture = self._orig_require_observer_vcp
        runner.start_capture = self._orig_start_capture


def run_main(argv: list[str]) -> tuple[int, str, str]:
    stdout = io.StringIO()
    stderr = io.StringIO()
    with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
        rc = runner.main(argv)
    return rc, stdout.getvalue(), stderr.getvalue()


def assert_ok(name: str, rc: int, stderr: str = "") -> None:
    if rc != 0:
        raise AssertionError(f"{name}: expected success, rc={rc}, stderr={stderr!r}")


def assert_fail(name: str, rc: int, stderr: str, needle: str) -> None:
    if rc == 0:
        raise AssertionError(f"{name}: expected failure")
    if needle not in stderr:
        raise AssertionError(f"{name}: missing {needle!r}, stderr={stderr!r}")


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="dpw_witness_runner_") as tmp:
        root = Path(tmp)
        context_path = root / "run_context.json"
        out_dir = root / "out"
        write_context(context_path)

        with Harness() as harness:
            rc, stdout, stderr = run_main([
                "--context", str(context_path), "--out", str(out_dir), "--scratch", "--overwrite-generated"
            ])
        assert_ok("success", rc, stderr)
        flash_events = [event for event in harness.events if event[0] == "flash"]
        features = [event[1]["features"] for event in flash_events]
        if features != ["", "replay_witness_observer", "sync_trigger_out sync_trigger_in sync_timing_capture"]:
            raise AssertionError(f"success: wrong flash features {features!r}")
        names = [event[0] for event in harness.events]
        if names.index("capture-start") > max(i for i, event in enumerate(harness.events) if event[0] == "flash" and event[1]["phase"] == "actor active"):
            raise AssertionError("success: capture did not start before actor active flash")
        if "WITNESS_DIGEST=0123456789abcdef" not in stdout:
            raise AssertionError("success: summary missing digest")
        for name in ("witness_report.txt", "meta.json", "wiring.txt", "run_context.json"):
            if not (out_dir / name).is_file():
                raise AssertionError(f"success: missing {name}")
        capture_events = [event for event in harness.events if event[0] == "capture-start"]
        if capture_events[0][1]["retention"] != "non_retained_scratch":
            raise AssertionError(
                f"success: wrong scratch retention {capture_events[0][1]['retention']!r}"
            )

        retained_root = root / "retained_runs"
        retained_run = retained_root / "0001"
        write_context(retained_run / "run_context.json")
        original_run_root = runner.DEFAULT_RUN_ROOT
        runner.DEFAULT_RUN_ROOT = retained_root
        try:
            with Harness() as harness:
                rc, _stdout, stderr = run_main(["--run-id", "0001"])
        finally:
            runner.DEFAULT_RUN_ROOT = original_run_root
        assert_ok("retained_success", rc, stderr)
        capture_events = [event for event in harness.events if event[0] == "capture-start"]
        if capture_events[0][1]["retention"] != "retained_hardware_witness":
            raise AssertionError(
                f"retained_success: wrong retention {capture_events[0][1]['retention']!r}"
            )

        outside = runner.REPO_ROOT / "witness_probe"
        rc, _stdout, stderr = run_main(["--context", str(context_path), "--out", str(outside), "--scratch", "--overwrite-generated"])
        assert_fail("scratch_rejects_outside_tmp", rc, stderr, "--scratch output must be under /tmp")

        timeout_out = root / "timeout"
        with Harness() as harness:
            harness.capture_writes_report = False
            harness.capture_returncode = 1
            rc, _stdout, stderr = run_main(["--context", str(context_path), "--out", str(timeout_out), "--scratch", "--overwrite-generated"])
        assert_fail("timeout_failure", rc, stderr, "witness capture failed")

        malformed_out = root / "malformed"
        with Harness() as harness:
            harness.report = MALFORMED_REPORT
            rc, _stdout, stderr = run_main(["--context", str(context_path), "--out", str(malformed_out), "--scratch", "--overwrite-generated"])
        assert_fail("malformed_report_failure", rc, stderr, "missing required witness fields")

        fail_out = root / "fail"
        with Harness() as harness:
            harness.report = FAIL_REPORT
            rc, _stdout, stderr = run_main(["--context", str(context_path), "--out", str(fail_out), "--scratch", "--overwrite-generated"])
        assert_fail("witness_fail_handling", rc, stderr, "witness RESULT was FAIL")

    print("PASS: replay witness runner regression suite")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
