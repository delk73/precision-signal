#!/usr/bin/env python3
"""Regression tests for reset-context-aware capture prompts."""

import contextlib
import io
import sys
from pathlib import Path


SCRIPTS_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPTS_DIR.parent
sys.path.insert(0, str(REPO_ROOT))
sys.path.insert(0, str(SCRIPTS_DIR))

import artifact_tool  # noqa: E402


def fake_read_artifact_main() -> int:
    print("Opening /dev/null at 115200 baud.")
    print("Listener active; press reset now", flush=True)
    print("Found MAGIC 'RPL0'.")
    return 0


def capture_prompt(reset_context: str) -> str:
    original_main = artifact_tool.read_artifact.main
    artifact_tool.read_artifact.main = fake_read_artifact_main
    try:
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            rc = artifact_tool.run_capture_with_argv(["read_artifact.py"], reset_context)
        if rc != 0:
            raise AssertionError(f"expected success, rc={rc}")
        return output.getvalue()
    finally:
        artifact_tool.read_artifact.main = original_main


def assert_contains(name: str, text: str, expected: str) -> None:
    if expected not in text:
        raise AssertionError(f"{name}: missing {expected!r} in output:\n{text}")


def assert_not_contains(name: str, text: str, unexpected: str) -> None:
    if unexpected in text:
        raise AssertionError(f"{name}: unexpected {unexpected!r} in output:\n{text}")


def main() -> int:
    parser = artifact_tool.build_parser()
    default_args = parser.parse_args(["capture"])
    if default_args.reset_context != "manual":
        raise AssertionError(f"default reset_context drifted: {default_args.reset_context}")
    if default_args.signal_model != "phase8":
        raise AssertionError(f"default signal_model drifted: {default_args.signal_model}")

    manual = capture_prompt("manual")
    assert_contains("manual", manual, "Listener active; press reset now")

    stlink = capture_prompt("stlink")
    assert_contains("stlink", stlink, "Listener active; waiting for target output")
    assert_not_contains("stlink", stlink, "press reset now")

    auto = capture_prompt("auto")
    assert_contains("auto", auto, "Listener active; waiting for target output")
    assert_not_contains("auto", auto, "press reset now")

    unspecified = capture_prompt("unspecified")
    assert_contains("unspecified", unspecified, "Listener active; waiting for target output or reset")
    assert_not_contains("unspecified", unspecified, "press reset now")

    captured: dict[str, object] = {}
    original_runner = artifact_tool.run_capture_with_argv

    def fake_runner(argv: list[str], reset_context: str) -> int:
        captured["argv"] = argv
        captured["reset_context"] = reset_context
        return 0

    artifact_tool.run_capture_with_argv = fake_runner
    try:
        args = parser.parse_args(["capture", "--signal-model", "burst8", "--reset-context", "auto"])
        rc = args.func(args)
    finally:
        artifact_tool.run_capture_with_argv = original_runner

    if rc != 0:
        raise AssertionError(f"capture signal-model forwarding failed: rc={rc}")
    if captured.get("argv") != ["read_artifact.py", "--signal-model", "burst8"]:
        raise AssertionError(f"capture did not forward burst8 cleanly: {captured}")
    if captured.get("reset_context") != "auto":
        raise AssertionError(f"capture reset_context drifted: {captured}")
    if artifact_tool.read_artifact.SIGNAL_MODELS != artifact_tool.SIGNAL_MODELS:
        raise AssertionError("read_artifact adapter did not install signal model choices")

    print("PASS: artifact_tool capture prompt reset-context regression suite")
    return 0


if __name__ == "__main__":
    sys.exit(main())
