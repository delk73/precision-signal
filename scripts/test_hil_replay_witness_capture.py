#!/usr/bin/env python3
"""Regression tests for replay witness report parsing."""

from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent


def valid_report() -> str:
    return """RESULT: PASS
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


def run_capture(report: str, out_dir: Path) -> subprocess.CompletedProcess[str]:
    report_path = out_dir.parent / f"{out_dir.name}.txt"
    report_path.write_text(report, encoding="utf-8")
    return subprocess.run(
        [
            "python3",
            "scripts/hil_replay_witness_capture.py",
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


def assert_ok(name: str, proc: subprocess.CompletedProcess[str]) -> None:
    if proc.returncode != 0:
        raise AssertionError(
            f"{name}: expected success, rc={proc.returncode}\nstdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )


def assert_fail(name: str, proc: subprocess.CompletedProcess[str], needle: str) -> None:
    if proc.returncode == 0:
        raise AssertionError(f"{name}: expected failure")
    if needle not in proc.stderr:
        raise AssertionError(
            f"{name}: missing {needle!r}\nstdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="dpw_witness_capture_") as tmp:
        root = Path(tmp)

        out = root / "valid"
        proc = run_capture(valid_report(), out)
        assert_ok("valid_pass", proc)
        for name in ("witness_report.txt", "meta.json", "wiring.txt"):
            if not (out / name).is_file():
                raise AssertionError(f"valid_pass: missing {name}")

        cases = [
            ("missing_field", valid_report().replace("ERROR: none\n", ""), "missing required"),
            ("invalid_result", valid_report().replace("RESULT: PASS", "RESULT: OK"), "invalid RESULT"),
            ("unknown_profile", valid_report().replace("dual_stm32_replay_witness_v1", "unknown_profile", 1), "unknown PROFILE"),
            ("unknown_rule", valid_report().replace("dual_stm32_pa6_pa1_pair_v0", "unknown_rule", 1), "unknown RULE_ID"),
            ("malformed_count", valid_report().replace("ACTOR_EVENT_COUNT: 10007", "ACTOR_EVENT_COUNT: ten"), "malformed ACTOR_EVENT_COUNT"),
            ("wrong_count", valid_report().replace("ACTOR_EVENT_COUNT: 10007", "ACTOR_EVENT_COUNT: 10000"), "invalid ACTOR_EVENT_COUNT: expected 10007, got 10000"),
            ("malformed_digest", valid_report().replace("WITNESS_DIGEST: 0123456789abcdef", "WITNESS_DIGEST: xyz"), "malformed WITNESS_DIGEST"),
            ("missing_event_window", valid_report().replace("EVENT_WINDOW: all_observed_events\n", ""), "missing required witness fields: EVENT_WINDOW"),
            ("invalid_actor_board", valid_report().replace("ACTOR_BOARD: STM32F446RE", "ACTOR_BOARD: STM32F407"), "invalid ACTOR_BOARD"),
            ("invalid_witness_board", valid_report().replace("WITNESS_BOARD: STM32F446RE", "WITNESS_BOARD: STM32F407"), "invalid WITNESS_BOARD"),
            ("invalid_actor_firmware", valid_report().replace("ACTOR_FIRMWARE: sync_trigger_out+sync_trigger_in+sync_timing_capture", "ACTOR_FIRMWARE: unknown"), "invalid ACTOR_FIRMWARE"),
            ("invalid_witness_firmware", valid_report().replace("WITNESS_FIRMWARE: replay_witness_observer", "WITNESS_FIRMWARE: sync_timing_observer"), "invalid WITNESS_FIRMWARE"),
            ("invalid_event_window", valid_report().replace("EVENT_WINDOW: all_observed_events", "EVENT_WINDOW: partial"), "invalid EVENT_WINDOW"),
            ("first_mismatch", valid_report().replace("FIRST_INVALID_EVENT: none", "FIRST_INVALID_EVENT: 3").replace("RESULT: PASS", "RESULT: FAIL").replace("ERROR: none", "ERROR: rule_mismatch"), ""),
            ("inconsistent_pass", valid_report().replace("ERROR: none", "ERROR: rule_mismatch"), "inconsistent PASS"),
            ("inconsistent_fail", valid_report().replace("RESULT: PASS", "RESULT: FAIL"), "inconsistent FAIL"),
        ]
        for name, report, needle in cases:
            proc = run_capture(report, root / name)
            if name == "first_mismatch":
                assert_ok(name, proc)
            else:
                assert_fail(name, proc, needle)

    print("PASS: replay witness capture parser regression suite")
    return 0


if __name__ == "__main__":
    sys.exit(main())
