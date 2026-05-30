#!/usr/bin/env python3
"""Regression tests for the dual-board HIL observer runner."""

from __future__ import annotations

import contextlib
import io
import json
import tempfile
from pathlib import Path

import hil_dual_observer_run as runner


ACTOR_STLINK = "066CFF505487525067182651"
OBSERVER_STLINK = "0668FF514988525067215029"
OBSERVER_VCP = (
    "/dev/serial/by-id/"
    "usb-STMicroelectronics_STM32_STLink_0668FF514988525067215029-if02"
)


VALID_REPORT = """SYNC_TIMING_CAPTURE_V1
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
wiring_profile=dual_edge_observer_v1
measured_path=PB9_PA1_minus_PB8_PA6
"""


def valid_context() -> dict[str, object]:
    return {
        "board_alias_confirmation": {
            "confirmed_from_dev_serial_by_id": True,
            "required_mappings": {
                ACTOR_STLINK: "actor / Board A",
                OBSERVER_STLINK: "observer / Board B",
            },
        },
        "board_aliases": {
            "actor": {
                "stlink_serial": ACTOR_STLINK,
                "vcp_by_id": (
                    "/dev/serial/by-id/"
                    "usb-STMicroelectronics_STM32_STLink_066CFF505487525067182651-if02"
                ),
                "firmware_features": "sync_trigger_out sync_trigger_in sync_timing_capture",
                "role": "external_actor",
            },
            "observer": {
                "stlink_serial": OBSERVER_STLINK,
                "vcp_by_id": OBSERVER_VCP,
                "firmware_features": "sync_timing_observer",
                "role": "external_observer",
            },
        },
    }


def write_context(path: Path, context: dict[str, object] | None = None) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(context or valid_context()) + "\n", encoding="utf-8")


class FakeStdout:
    def __init__(self, text: str) -> None:
        self.lines = text.splitlines(keepends=True)

    def __iter__(self) -> "FakeStdout":
        return self

    def __next__(self) -> str:
        if not self.lines:
            raise StopIteration
        return self.lines.pop(0)


class FakeFlashPopen:
    def __init__(
        self,
        command: list[str],
        kwargs: dict[str, object],
        harness: "Harness",
    ) -> None:
        env = kwargs.get("env")
        assert isinstance(env, dict)
        serial = env.get("STFLASH_SERIAL")
        has_features = "FW_FEATURES" in env
        features = env["FW_FEATURES"] if has_features else None
        cwd = kwargs.get("cwd")
        results = harness.flash_results_by_features.get(str(features), [])
        if results:
            returncode, output = results.pop(0)
        else:
            failed = serial in harness.flash_fail_serials or features in harness.flash_fail_features
            returncode = 1 if failed else 0
            output = "flash failed\n" if failed else "flash ok\n"
        self.returncode = returncode
        self.stdout = FakeStdout(output)
        harness.events.append(
            (
                "flash",
                {
                    "command": command,
                    "cwd": cwd,
                    "serial": serial,
                    "features": features,
                    "has_features": has_features,
                    "returncode": returncode,
                    "output": output,
                },
            )
        )

    def wait(self, timeout: float | None = None) -> int:
        return self.returncode


class FakePopen:
    def __init__(self, command: list[str], harness: "Harness") -> None:
        self.command = command
        self.harness = harness
        self.returncode: int | None = None
        self.terminated = False
        self.killed = False
        harness.events.append(("capture-start", command))

    def poll(self) -> int | None:
        return self.returncode

    def wait(self, timeout: float | None = None) -> int:
        self.harness.events.append(("capture-wait", timeout))
        if self.returncode is None:
            if not self.terminated and self.harness.capture_writes_report:
                out_dir = Path(self.command[self.command.index("--out") + 1])
                out_dir.mkdir(parents=True, exist_ok=True)
                (out_dir / "timing_report.txt").write_text(VALID_REPORT, encoding="utf-8")
            self.returncode = self.harness.capture_returncode
        return self.returncode

    def terminate(self) -> None:
        self.harness.events.append(("capture-terminate", None))
        self.terminated = True
        if self.harness.terminate_exits:
            self.returncode = -15

    def kill(self) -> None:
        self.harness.events.append(("capture-kill", None))
        self.killed = True
        self.returncode = -9


class Harness:
    def __init__(self) -> None:
        self.events: list[tuple[str, object]] = []
        self.flash_fail_serials: set[str] = set()
        self.flash_fail_features: set[str] = set()
        self.flash_results_by_features: dict[str, list[tuple[int, str]]] = {}
        self.flash_identity_present = {"actor": True, "observer": True}
        self.observer_vcp_present = True
        self.capture_returncode = 0
        self.capture_writes_report = True
        self.terminate_exits = True
        self._orig_popen = runner.subprocess.Popen
        self._orig_require_flash_identity = runner.require_flash_identity
        self._orig_require_observer_vcp_for_capture = runner.require_observer_vcp_for_capture

    def __enter__(self) -> "Harness":
        def fake_popen(command: list[str], **kwargs: object) -> FakePopen:
            if command[-1:] == ["flash-ur"]:
                return FakeFlashPopen(command, kwargs, self)
            return FakePopen(command, self)

        def fake_require_flash_identity(
            role: str, phase: str, alias_entry: dict[str, str]
        ) -> str:
            self.events.append(("flash-ready", (role, phase)))
            if not self.flash_identity_present[role]:
                raise RuntimeError(f"{role} flash identity not ready for {phase}")
            return alias_entry["stlink_serial"]

        def fake_require_observer_vcp_for_capture(observer_vcp: str) -> str:
            self.events.append(("vcp-ready", observer_vcp))
            if not self.observer_vcp_present:
                raise RuntimeError("observer VCP not ready for capture start")
            return observer_vcp

        runner.subprocess.Popen = fake_popen
        runner.require_flash_identity = fake_require_flash_identity
        runner.require_observer_vcp_for_capture = fake_require_observer_vcp_for_capture
        return self

    def __exit__(self, exc_type: object, exc: object, tb: object) -> None:
        runner.subprocess.Popen = self._orig_popen
        runner.require_flash_identity = self._orig_require_flash_identity
        runner.require_observer_vcp_for_capture = self._orig_require_observer_vcp_for_capture


def assert_equal(name: str, actual: object, expected: object) -> None:
    if actual != expected:
        raise AssertionError(f"{name}: expected {expected!r}, got {actual!r}")


def assert_ok(name: str, rc: int) -> None:
    if rc != 0:
        raise AssertionError(f"{name}: expected success, rc={rc}")


def assert_fail(name: str, rc: int) -> None:
    if rc == 0:
        raise AssertionError(f"{name}: expected failure")


def run_id_resolves_retained_directory() -> None:
    args = runner.parse_args(["--run-id", "0003"])
    context_path, out_dir = runner.resolve_context_and_out(args)
    assert_equal("run-id context", context_path, Path("artifacts/hil_timing_dual/0003/run_context.json"))
    assert_equal("run-id out", out_dir, Path("artifacts/hil_timing_dual/0003"))


def scratch_requires_out() -> None:
    try:
        with contextlib.redirect_stderr(io.StringIO()):
            runner.parse_args(["--run-id", "0003", "--scratch"])
    except SystemExit:
        return
    raise AssertionError("scratch_requires_out: expected parser failure")


def run_main_quiet(argv: list[str]) -> int:
    with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
        return runner.main(argv)


def run_main_capture(argv: list[str]) -> tuple[int, str, str]:
    stdout = io.StringIO()
    stderr = io.StringIO()
    with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
        rc = runner.main(argv)
    return rc, stdout.getvalue(), stderr.getvalue()


def scratch_rejects_outside_tmp(root: Path) -> None:
    context_path = root / "run_context.json"
    write_context(context_path)
    rc = run_main_quiet(
        [
            "--context",
            str(context_path),
            "--out",
            str(runner.REPO_ROOT / "dual_observer_probe"),
            "--scratch",
            "--overwrite-generated",
        ]
    )
    assert_fail("scratch_rejects_outside_tmp", rc)


def context_out_scratch_uses_context_aliases(root: Path) -> None:
    context_path = root / "run_context.json"
    out_dir = root / "scratch"
    write_context(context_path)
    source_context_bytes = context_path.read_bytes()
    with Harness() as harness:
        rc = run_main_quiet(
            [
                "--context",
                str(context_path),
                "--out",
                str(out_dir),
                "--scratch",
                "--overwrite-generated",
            ]
        )
    assert_ok("context_out_scratch_uses_context_aliases", rc)
    if not (out_dir / "run_context.json").is_file():
        raise AssertionError("context_out_scratch_uses_context_aliases: missing copied context")
    assert_equal(
        "copied context bytes",
        (out_dir / "run_context.json").read_bytes(),
        source_context_bytes,
    )
    flash_events = [event for event in harness.events if event[0] == "flash"]
    assert_equal("flash count", len(flash_events), 3)
    assert_equal("quiesce serial", flash_events[0][1]["serial"], ACTOR_STLINK)
    assert_equal("observer serial", flash_events[1][1]["serial"], OBSERVER_STLINK)
    assert_equal("actor serial", flash_events[2][1]["serial"], ACTOR_STLINK)


def bad_context_fails_before_flash(root: Path) -> None:
    context = valid_context()
    context["board_alias_confirmation"]["confirmed_from_dev_serial_by_id"] = False
    context_path = root / "bad_context.json"
    out_dir = root / "bad_context_out"
    write_context(context_path, context)
    with Harness() as harness:
        rc = run_main_quiet(
            [
                "--context",
                str(context_path),
                "--out",
                str(out_dir),
                "--scratch",
                "--overwrite-generated",
            ]
        )
    assert_fail("bad_context_fails_before_flash", rc)
    assert_equal("no flash", [event for event in harness.events if event[0] == "flash"], [])


def generated_files_fail_before_flash(root: Path) -> None:
    context_path = root / "existing_context.json"
    out_dir = root / "existing_out"
    write_context(context_path)
    out_dir.mkdir()
    (out_dir / "timing_report.txt").write_text("old\n", encoding="utf-8")
    with Harness() as harness:
        rc = run_main_quiet(
            ["--context", str(context_path), "--out", str(out_dir), "--scratch"]
        )
    assert_fail("generated_files_fail_before_flash", rc)
    assert_equal("no flash", [event for event in harness.events if event[0] == "flash"], [])


def retained_generated_files_fail_before_flash(root: Path) -> None:
    run_root = root / "retained"
    out_dir = run_root / "0009"
    context_path = out_dir / "run_context.json"
    write_context(context_path)
    (out_dir / "timing_report.txt").write_text("old\n", encoding="utf-8")
    original_run_root = runner.DEFAULT_RUN_ROOT
    runner.DEFAULT_RUN_ROOT = run_root
    try:
        with Harness() as harness:
            rc = run_main_quiet(["--run-id", "0009"])
    finally:
        runner.DEFAULT_RUN_ROOT = original_run_root
    assert_fail("retained_generated_files_fail_before_flash", rc)
    assert_equal("no flash", [event for event in harness.events if event[0] == "flash"], [])


def missing_actor_flash_identity_before_quiesce_fails_before_flash(root: Path) -> None:
    context_path = root / "missing_actor_identity_context.json"
    out_dir = root / "missing_actor_identity_out"
    write_context(context_path)
    with Harness() as harness:
        harness.flash_identity_present["actor"] = False
        rc, _stdout, stderr = run_main_capture(
            [
                "--context",
                str(context_path),
                "--out",
                str(out_dir),
                "--scratch",
                "--overwrite-generated",
            ]
        )
    assert_fail("missing_actor_flash_identity_before_quiesce_fails_before_flash", rc)
    if "FAIL: actor flash identity not ready for actor quiesce" not in stderr:
        raise AssertionError(
            "missing_actor_flash_identity_before_quiesce_fails_before_flash: "
            f"unexpected stderr {stderr!r}"
        )
    assert_equal("no flash", [event for event in harness.events if event[0] == "flash"], [])


def missing_observer_flash_identity_before_observer_flash_fails_before_capture(
    root: Path,
) -> None:
    context_path = root / "missing_observer_identity_context.json"
    out_dir = root / "missing_observer_identity_out"
    write_context(context_path)
    with Harness() as harness:
        harness.flash_identity_present["observer"] = False
        rc, _stdout, stderr = run_main_capture(
            [
                "--context",
                str(context_path),
                "--out",
                str(out_dir),
                "--scratch",
                "--overwrite-generated",
            ]
        )
    assert_fail(
        "missing_observer_flash_identity_before_observer_flash_fails_before_capture",
        rc,
    )
    if "FAIL: observer flash identity not ready for observer" not in stderr:
        raise AssertionError(
            "missing_observer_flash_identity_before_observer_flash_fails_before_capture: "
            f"unexpected stderr {stderr!r}"
        )
    flash_events = [event for event in harness.events if event[0] == "flash"]
    assert_equal("only actor quiesce flashed", len(flash_events), 1)
    assert_equal("no capture", [event for event in harness.events if event[0] == "capture-start"], [])


def observer_flash_failure_prevents_capture(root: Path) -> None:
    context_path = root / "observer_fail_context.json"
    out_dir = root / "observer_fail_out"
    write_context(context_path)
    with Harness() as harness:
        harness.flash_fail_serials.add(OBSERVER_STLINK)
        rc = run_main_quiet(
            [
                "--context",
                str(context_path),
                "--out",
                str(out_dir),
                "--scratch",
                "--overwrite-generated",
            ]
        )
    assert_fail("observer_flash_failure_prevents_capture", rc)
    assert_equal("no capture", [event for event in harness.events if event[0] == "capture-start"], [])


def observer_vcp_timeout_after_observer_flash_settle_is_not_flash_retry(root: Path) -> None:
    context_path = root / "observer_vcp_timeout_context.json"
    out_dir = root / "observer_vcp_timeout_out"
    write_context(context_path)
    with Harness() as harness:
        harness.observer_vcp_present = False
        rc, _stdout, stderr = run_main_capture(
            [
                "--context",
                str(context_path),
                "--out",
                str(out_dir),
                "--scratch",
                "--overwrite-generated",
            ]
        )
    assert_fail("observer_vcp_timeout_after_observer_flash_settle_is_not_flash_retry", rc)
    if "FAIL: observer VCP not ready for capture start" not in stderr:
        raise AssertionError(
            "observer_vcp_timeout_after_observer_flash_settle_is_not_flash_retry: "
            f"unexpected stderr {stderr!r}"
        )
    event_names = [event[0] for event in harness.events]
    expected_prefix = [
        "flash-ready",
        "flash",
        "flash-ready",
        "flash-ready",
        "flash",
        "flash-ready",
        "vcp-ready",
    ]
    assert_equal("vcp timeout order", event_names[: len(expected_prefix)], expected_prefix)
    flash_events = [event for event in harness.events if event[0] == "flash"]
    assert_equal("quiesce and observer only", len(flash_events), 2)
    assert_equal("observer flash succeeded", flash_events[1][1]["returncode"], 0)
    assert_equal("no capture", [event for event in harness.events if event[0] == "capture-start"], [])


def actor_flash_failure_terminates_capture(root: Path) -> None:
    context_path = root / "actor_fail_context.json"
    out_dir = root / "actor_fail_out"
    write_context(context_path)
    with Harness() as harness:
        harness.flash_fail_features.add("sync_trigger_out sync_trigger_in sync_timing_capture")
        rc = run_main_quiet(
            [
                "--context",
                str(context_path),
                "--out",
                str(out_dir),
                "--scratch",
                "--overwrite-generated",
            ]
        )
    assert_fail("actor_flash_failure_terminates_capture", rc)
    event_names = [event[0] for event in harness.events]
    if "capture-terminate" not in event_names:
        raise AssertionError("actor_flash_failure_terminates_capture: capture was not terminated")


def retryable_actor_active_flash_failure_recovers_and_waits_for_capture(root: Path) -> None:
    context_path = root / "actor_retry_context.json"
    out_dir = root / "actor_retry_out"
    write_context(context_path)
    active_features = "sync_trigger_out sync_trigger_in sync_timing_capture"
    with Harness() as harness:
        harness.flash_results_by_features[active_features] = [
            (1, "LIBUSB_ERROR_TIMEOUT\n"),
            (0, "flash ok\n"),
        ]
        rc, stdout, _stderr = run_main_capture(
            [
                "--context",
                str(context_path),
                "--out",
                str(out_dir),
                "--scratch",
                "--overwrite-generated",
            ]
        )
    assert_ok("retryable_actor_active_flash_failure_recovers_and_waits_for_capture", rc)
    if (
        "WARN: actor active flash failed with retryable ST-LINK transport error; "
        "waiting for actor flash identity recovery"
    ) not in stdout:
        raise AssertionError(
            "retryable_actor_active_flash_failure_recovers_and_waits_for_capture: "
            "missing retry classification"
        )
    if "WARN: retrying actor active flash attempt 2/2" not in stdout:
        raise AssertionError(
            "retryable_actor_active_flash_failure_recovers_and_waits_for_capture: "
            "missing retry attempt warning"
        )
    active_flashes = [
        event for event in harness.events
        if event[0] == "flash" and event[1]["features"] == active_features
    ]
    assert_equal("active actor attempts", len(active_flashes), 2)
    active_readiness = [
        event for event in harness.events
        if event == ("flash-ready", ("actor", "actor active"))
    ]
    assert_equal("active actor readiness checks", len(active_readiness), 2)
    if "capture-wait" not in [event[0] for event in harness.events]:
        raise AssertionError(
            "retryable_actor_active_flash_failure_recovers_and_waits_for_capture: "
            "capture wait not reached"
        )


def retryable_observer_flash_failure_recovers_before_vcp_check(root: Path) -> None:
    context_path = root / "observer_retry_context.json"
    out_dir = root / "observer_retry_out"
    write_context(context_path)
    with Harness() as harness:
        harness.flash_results_by_features["sync_timing_observer"] = [
            (1, "LIBUSB_ERROR_NO_DEVICE\n"),
            (0, "flash ok\n"),
        ]
        rc, stdout, _stderr = run_main_capture(
            [
                "--context",
                str(context_path),
                "--out",
                str(out_dir),
                "--scratch",
                "--overwrite-generated",
            ]
        )
    assert_ok("retryable_observer_flash_failure_recovers_before_vcp_check", rc)
    if (
        "WARN: observer flash failed with retryable ST-LINK transport error; "
        "waiting for observer flash identity recovery"
    ) not in stdout:
        raise AssertionError(
            "retryable_observer_flash_failure_recovers_before_vcp_check: "
            "missing retry classification"
        )
    observer_flashes = [
        event for event in harness.events
        if event[0] == "flash" and event[1]["features"] == "sync_timing_observer"
    ]
    assert_equal("observer attempts", len(observer_flashes), 2)
    event_names = [event[0] for event in harness.events]
    last_observer_flash = max(
        index for index, event in enumerate(harness.events)
        if event[0] == "flash" and event[1]["features"] == "sync_timing_observer"
    )
    vcp_index = event_names.index("vcp-ready")
    if vcp_index <= last_observer_flash:
        raise AssertionError(
            "retryable_observer_flash_failure_recovers_before_vcp_check: "
            "observer VCP checked before observer retry completed"
        )


def retryable_observer_flash_failure_exhaustion_skips_capture(root: Path) -> None:
    context_path = root / "observer_retry_exhaust_context.json"
    out_dir = root / "observer_retry_exhaust_out"
    write_context(context_path)
    with Harness() as harness:
        harness.flash_results_by_features["sync_timing_observer"] = [
            (1, "LIBUSB_ERROR_NO_DEVICE\n"),
            (1, "LIBUSB_ERROR_NO_DEVICE\n"),
        ]
        rc, _stdout, stderr = run_main_capture(
            [
                "--context",
                str(context_path),
                "--out",
                str(out_dir),
                "--scratch",
                "--overwrite-generated",
            ]
        )
    assert_fail("retryable_observer_flash_failure_exhaustion_skips_capture", rc)
    if (
        "FAIL: observer flash failed after retryable ST-LINK transport recovery attempt"
        not in stderr
    ):
        raise AssertionError(
            "retryable_observer_flash_failure_exhaustion_skips_capture: "
            f"unexpected stderr {stderr!r}"
        )
    observer_flashes = [
        event for event in harness.events
        if event[0] == "flash" and event[1]["features"] == "sync_timing_observer"
    ]
    assert_equal("observer attempts", len(observer_flashes), 2)
    assert_equal("no capture", [event for event in harness.events if event[0] == "capture-start"], [])


def observer_retry_then_nonretryable_failure_reports_recovery_attempt(
    root: Path,
) -> None:
    context_path = root / "observer_retry_then_nonretry_context.json"
    out_dir = root / "observer_retry_then_nonretry_out"
    write_context(context_path)
    with Harness() as harness:
        harness.flash_results_by_features["sync_timing_observer"] = [
            (1, "LIBUSB_ERROR_NO_DEVICE\n"),
            (1, "cargo build failed\n"),
        ]
        rc, _stdout, stderr = run_main_capture(
            [
                "--context",
                str(context_path),
                "--out",
                str(out_dir),
                "--scratch",
                "--overwrite-generated",
            ]
        )
    assert_fail("observer_retry_then_nonretryable_failure_reports_recovery_attempt", rc)
    if (
        "FAIL: observer flash failed after retryable ST-LINK transport recovery attempt"
        not in stderr
    ):
        raise AssertionError(
            "observer_retry_then_nonretryable_failure_reports_recovery_attempt: "
            f"unexpected stderr {stderr!r}"
        )
    observer_flashes = [
        event for event in harness.events
        if event[0] == "flash" and event[1]["features"] == "sync_timing_observer"
    ]
    assert_equal("observer attempts", len(observer_flashes), 2)
    assert_equal("no capture", [event for event in harness.events if event[0] == "capture-start"], [])


def zero_retry_count_retryable_observer_flash_failure_does_not_succeed(
    root: Path,
) -> None:
    context_path = root / "observer_zero_retry_context.json"
    out_dir = root / "observer_zero_retry_out"
    write_context(context_path)
    original_retry_count = runner.FLASH_RETRY_COUNT
    runner.FLASH_RETRY_COUNT = 0
    try:
        with Harness() as harness:
            harness.flash_results_by_features["sync_timing_observer"] = [
                (1, "LIBUSB_ERROR_NO_DEVICE\n"),
            ]
            rc, stdout, stderr = run_main_capture(
                [
                    "--context",
                    str(context_path),
                    "--out",
                    str(out_dir),
                    "--scratch",
                    "--overwrite-generated",
                ]
            )
    finally:
        runner.FLASH_RETRY_COUNT = original_retry_count
    assert_fail("zero_retry_count_retryable_observer_flash_failure_does_not_succeed", rc)
    if (
        "FAIL: observer flash failed after retryable ST-LINK transport recovery attempt"
        not in stderr
    ):
        raise AssertionError(
            "zero_retry_count_retryable_observer_flash_failure_does_not_succeed: "
            f"unexpected stderr {stderr!r}"
        )
    if "WARN: retrying observer flash attempt" in stdout:
        raise AssertionError(
            "zero_retry_count_retryable_observer_flash_failure_does_not_succeed: "
            "unexpected retry warning"
        )
    observer_flashes = [
        event for event in harness.events
        if event[0] == "flash" and event[1]["features"] == "sync_timing_observer"
    ]
    assert_equal("observer attempts", len(observer_flashes), 1)
    assert_equal("no capture", [event for event in harness.events if event[0] == "capture-start"], [])


def non_retryable_observer_flash_failure_is_not_retried(root: Path) -> None:
    context_path = root / "observer_non_retry_context.json"
    out_dir = root / "observer_non_retry_out"
    write_context(context_path)
    with Harness() as harness:
        harness.flash_results_by_features["sync_timing_observer"] = [
            (1, "cargo build failed\n"),
        ]
        rc, _stdout, stderr = run_main_capture(
            [
                "--context",
                str(context_path),
                "--out",
                str(out_dir),
                "--scratch",
                "--overwrite-generated",
            ]
        )
    assert_fail("non_retryable_observer_flash_failure_is_not_retried", rc)
    if "FAIL: observer flash failed with non-retryable exit code 1" not in stderr:
        raise AssertionError(
            "non_retryable_observer_flash_failure_is_not_retried: "
            f"unexpected stderr {stderr!r}"
        )
    observer_flashes = [
        event for event in harness.events
        if event[0] == "flash" and event[1]["features"] == "sync_timing_observer"
    ]
    assert_equal("observer attempts", len(observer_flashes), 1)
    assert_equal("no capture", [event for event in harness.events if event[0] == "capture-start"], [])


def quiesce_failure_prevents_observer_capture_and_generated_artifacts(root: Path) -> None:
    context_path = root / "quiesce_fail_context.json"
    out_dir = root / "quiesce_fail_out"
    write_context(context_path)
    with Harness() as harness:
        harness.flash_fail_features.add("")
        rc = run_main_quiet(
            [
                "--context",
                str(context_path),
                "--out",
                str(out_dir),
                "--scratch",
                "--overwrite-generated",
            ]
        )
    assert_fail("quiesce_failure_prevents_observer_capture_and_generated_artifacts", rc)
    flash_events = [event for event in harness.events if event[0] == "flash"]
    assert_equal("only quiesce flash", len(flash_events), 1)
    assert_equal("quiesce serial", flash_events[0][1]["serial"], ACTOR_STLINK)
    assert_equal("quiesce features", flash_events[0][1]["features"], "")
    assert_equal("no capture", [event for event in harness.events if event[0] == "capture-start"], [])
    for name in ("timing_report.txt", "meta.json", "wiring.txt"):
        if (out_dir / name).exists():
            raise AssertionError(
                "quiesce_failure_prevents_observer_capture_and_generated_artifacts: "
                f"unexpected generated artifact {name}"
            )


def success_order_is_quiesce_observer_capture_actor_wait(root: Path) -> None:
    context_path = root / "success_context.json"
    out_dir = root / "success_out"
    write_context(context_path)
    with Harness() as harness:
        rc = run_main_quiet(
            [
                "--context",
                str(context_path),
                "--out",
                str(out_dir),
                "--scratch",
                "--overwrite-generated",
            ]
        )
    assert_ok("success_order_is_quiesce_observer_capture_actor_wait", rc)
    event_names = [event[0] for event in harness.events]
    assert_equal(
        "success order",
        event_names[:10],
        [
            "flash-ready",
            "flash",
            "flash-ready",
            "flash-ready",
            "flash",
            "flash-ready",
            "vcp-ready",
            "capture-start",
            "flash-ready",
            "flash",
        ],
    )
    if "capture-wait" not in event_names:
        raise AssertionError("success_order_is_quiesce_observer_capture_actor_wait: no capture wait")
    flash_events = [event[1] for event in harness.events if event[0] == "flash"]
    assert_equal("quiesce features", flash_events[0]["features"], "")
    assert_equal("observer features", flash_events[1]["features"], "sync_timing_observer")
    assert_equal(
        "active actor features",
        flash_events[2]["features"],
        "sync_trigger_out sync_trigger_in sync_timing_capture",
    )


def quiesce_uses_existing_under_reset_make_path(root: Path) -> None:
    context_path = root / "make_path_context.json"
    out_dir = root / "make_path_out"
    write_context(context_path)
    with Harness() as harness:
        rc = run_main_quiet(
            [
                "--context",
                str(context_path),
                "--out",
                str(out_dir),
                "--scratch",
                "--overwrite-generated",
                "--make",
                "custom-make",
            ]
        )
    assert_ok("quiesce_uses_existing_under_reset_make_path", rc)
    flash_events = [event[1] for event in harness.events if event[0] == "flash"]
    quiesce = flash_events[0]
    assert_equal("quiesce command", quiesce["command"], ["custom-make", "flash-ur"])
    assert_equal("quiesce cwd", quiesce["cwd"], runner.REPO_ROOT)
    if "st-flash" in quiesce["command"]:
        raise AssertionError("quiesce_uses_existing_under_reset_make_path: direct st-flash call")
    for flash in flash_events:
        assert_equal("flash command", flash["command"], ["custom-make", "flash-ur"])
        assert_equal("flash cwd", flash["cwd"], runner.REPO_ROOT)
        assert_equal("FW_FEATURES present", flash["has_features"], True)


def scratch_rejects_unexpected_existing_file(root: Path) -> None:
    context_path = root / "unexpected_context.json"
    out_dir = root / "unexpected_out"
    write_context(context_path)
    out_dir.mkdir()
    (out_dir / "operator.log").write_text("old\n", encoding="utf-8")
    with Harness() as harness:
        rc = run_main_quiet(
            [
                "--context",
                str(context_path),
                "--out",
                str(out_dir),
                "--scratch",
                "--overwrite-generated",
            ]
        )
    assert_fail("scratch_rejects_unexpected_existing_file", rc)
    assert_equal("no flash", [event for event in harness.events if event[0] == "flash"], [])


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="dpw_hil_dual_runner_") as tmp:
        root = Path(tmp)
        run_id_resolves_retained_directory()
        scratch_requires_out()
        scratch_rejects_outside_tmp(root)
        context_out_scratch_uses_context_aliases(root)
        bad_context_fails_before_flash(root)
        generated_files_fail_before_flash(root)
        retained_generated_files_fail_before_flash(root)
        missing_actor_flash_identity_before_quiesce_fails_before_flash(root)
        missing_observer_flash_identity_before_observer_flash_fails_before_capture(root)
        observer_flash_failure_prevents_capture(root)
        observer_vcp_timeout_after_observer_flash_settle_is_not_flash_retry(root)
        actor_flash_failure_terminates_capture(root)
        retryable_actor_active_flash_failure_recovers_and_waits_for_capture(root)
        retryable_observer_flash_failure_recovers_before_vcp_check(root)
        retryable_observer_flash_failure_exhaustion_skips_capture(root)
        observer_retry_then_nonretryable_failure_reports_recovery_attempt(root)
        zero_retry_count_retryable_observer_flash_failure_does_not_succeed(root)
        non_retryable_observer_flash_failure_is_not_retried(root)
        quiesce_failure_prevents_observer_capture_and_generated_artifacts(root)
        success_order_is_quiesce_observer_capture_actor_wait(root)
        quiesce_uses_existing_under_reset_make_path(root)
        scratch_rejects_unexpected_existing_file(root)

    print("PASS: HIL dual observer runner regression suite")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
