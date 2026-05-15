#!/usr/bin/env python3
"""Local bench helper for live RPL0 signal-model smoke captures."""

from __future__ import annotations

import os
import re
import shlex
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
OUT_ROOT = REPO_ROOT / "artifacts" / "model_validation"
SERIAL_PORT = "/dev/ttyACM0"
HASH_RE = re.compile(r"^sha256:\s*([0-9a-f]{64})$", re.MULTILINE)


@dataclass(frozen=True)
class ModeSpec:
    mode: str
    feature: str | None


@dataclass
class CommandResult:
    rc: int | None = None
    output: str = ""
    skipped: bool = False

    @property
    def status(self) -> str:
        if self.skipped:
            return "SKIPPED"
        if self.rc == 0:
            return "PASS"
        if self.rc is None:
            return "NOT RUN"
        return f"FAIL ({self.rc})"


@dataclass
class ModeResult:
    mode: str
    feature: str
    artifact: Path
    build: CommandResult
    flash_ur: CommandResult
    flash_compare_ur: CommandResult
    capture: CommandResult
    verify: CommandResult
    hash_result: CommandResult


MODES = (
    ModeSpec("phase8", None),
    ModeSpec("burst8", "signal-model-burst8"),
    ModeSpec("seeded_lfsr8", "signal-model-seeded-lfsr8"),
)


def shell_join(argv: list[str], env_extra: dict[str, str] | None = None) -> str:
    prefix = ""
    if env_extra:
        prefix = " ".join(f"{key}={shlex.quote(value)}" for key, value in env_extra.items())
        prefix += " "
    return prefix + " ".join(shlex.quote(part) for part in argv)


def run_command(argv: list[str], env_extra: dict[str, str] | None = None) -> CommandResult:
    env = os.environ.copy()
    if env_extra:
        env.update(env_extra)

    print()
    print(f"$ {shell_join(argv, env_extra)}", flush=True)
    proc = subprocess.Popen(
        argv,
        cwd=REPO_ROOT,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        bufsize=1,
    )

    output_parts: list[str] = []
    assert proc.stdout is not None
    for line in proc.stdout:
        output_parts.append(line)
        print(line, end="")
    rc = proc.wait()
    return CommandResult(rc=rc, output="".join(output_parts))


def build_args(spec: ModeSpec) -> list[str]:
    argv = [
        "cargo",
        "build",
        "-p",
        "replay-fw-f446",
        "--target",
        "thumbv7em-none-eabihf",
        "--locked",
    ]
    if spec.feature is not None:
        argv.extend(["--features", spec.feature])
    return argv


def feature_make_arg(spec: ModeSpec) -> list[str]:
    if spec.feature is None:
        return []
    return [f"REPLAY_FW_FEATURES={spec.feature}"]


def flash_ur_args(spec: ModeSpec) -> list[str]:
    return ["make", "flash-ur", *feature_make_arg(spec)]


def flash_compare_ur_args(spec: ModeSpec) -> list[str]:
    return ["make", "flash-compare-ur", *feature_make_arg(spec)]


def capture_args(mode: str, artifact: Path) -> list[str]:
    artifact_arg = markdown_path(artifact)
    return [
        "python3",
        "scripts/artifact_tool.py",
        "capture",
        "--quick",
        "--reset-context",
        "auto",
        "--out",
        artifact_arg,
        "--signal-model",
        mode,
    ]


def verify_args(mode: str, artifact: Path) -> list[str]:
    artifact_arg = markdown_path(artifact)
    return [
        "python3",
        "scripts/artifact_tool.py",
        "verify",
        artifact_arg,
        "--signal-model",
        mode,
    ]


def hash_args(artifact: Path) -> list[str]:
    return ["python3", "scripts/artifact_tool.py", "hash", markdown_path(artifact)]


def python_env() -> dict[str, str]:
    return {
        "PYTHONPATH": str(REPO_ROOT),
    }


def capture_env() -> dict[str, str]:
    return {
        "SERIAL": SERIAL_PORT,
        "PYTHONPATH": str(REPO_ROOT),
    }


def markdown_path(path: Path) -> str:
    return str(path.relative_to(REPO_ROOT))


def hash_digest(result: CommandResult) -> str:
    match = HASH_RE.search(result.output)
    if match is None:
        return ""
    return match.group(1)


def write_summary(path: Path, results: list[ModeResult]) -> None:
    lines = [
        "# RPL0 Model Live Smoke",
        "",
        f"- timestamp: `{path.parent.name}`",
        f"- serial: `{SERIAL_PORT}`",
        "",
        "| Mode | Feature | Artifact | Verify result | Hash result | SHA256 |",
        "| --- | --- | --- | --- | --- | --- |",
    ]
    for result in results:
        digest = hash_digest(result.hash_result)
        lines.append(
            "| "
            f"`{result.mode}` | "
            f"`{result.feature}` | "
            f"`{markdown_path(result.artifact)}` | "
            f"{result.verify.status} | "
            f"{result.hash_result.status} | "
            f"`{digest}` |"
        )

    lines.extend(
        [
            "",
            "## Command Results",
            "",
            "| Mode | Build | Flash | Flash compare | Capture | Verify | Hash |",
            "| --- | --- | --- | --- | --- | --- | --- |",
        ]
    )
    for result in results:
        lines.append(
            "| "
            f"`{result.mode}` | "
            f"{result.build.status} | "
            f"{result.flash_ur.status} | "
            f"{result.flash_compare_ur.status} | "
            f"{result.capture.status} | "
            f"{result.verify.status} | "
            f"{result.hash_result.status} |"
        )

    path.write_text("\n".join(lines) + "\n")


def run_mode(spec: ModeSpec, out_dir: Path) -> ModeResult:
    feature_label = spec.feature if spec.feature is not None else "default"
    artifact = out_dir / f"{spec.mode}.bin"
    print()
    print(f"=== {spec.mode} ({feature_label}) ===", flush=True)

    build = run_command(build_args(spec))
    flash_ur = CommandResult(skipped=True)
    flash_compare_ur = CommandResult(skipped=True)
    capture = CommandResult(skipped=True)
    verify = CommandResult(skipped=True)
    hash_result = CommandResult(skipped=True)

    if build.rc == 0:
        flash_ur = run_command(flash_ur_args(spec))
    if flash_ur.rc == 0:
        flash_compare_ur = run_command(flash_compare_ur_args(spec))
    if flash_ur.rc == 0 and flash_compare_ur.rc == 0:
        capture = run_command(capture_args(spec.mode, artifact), env_extra=capture_env())
    if capture.rc == 0:
        verify = run_command(verify_args(spec.mode, artifact), env_extra=python_env())
    if verify.rc == 0:
        hash_result = run_command(hash_args(artifact), env_extra=python_env())

    return ModeResult(
        mode=spec.mode,
        feature=feature_label,
        artifact=artifact,
        build=build,
        flash_ur=flash_ur,
        flash_compare_ur=flash_compare_ur,
        capture=capture,
        verify=verify,
        hash_result=hash_result,
    )


def main() -> int:
    timestamp = datetime.now().strftime("%Y%m%d-%H%M%S")
    out_dir = OUT_ROOT / timestamp
    out_dir.mkdir(parents=True, exist_ok=False)
    summary_path = out_dir / "summary.md"

    results: list[ModeResult] = []
    rc = 0
    try:
        for spec in MODES:
            result = run_mode(spec, out_dir)
            results.append(result)
            write_summary(summary_path, results)
            if any(
                step.rc not in (0, None) and not step.skipped
                for step in (
                    result.build,
                    result.flash_ur,
                    result.flash_compare_ur,
                    result.capture,
                    result.verify,
                    result.hash_result,
                )
            ):
                rc = 1
    except KeyboardInterrupt:
        print()
        print("Interrupted.")
        rc = 130
    finally:
        write_summary(summary_path, results)
        print()
        print(f"Summary: {markdown_path(summary_path)}")

    return rc


if __name__ == "__main__":
    raise SystemExit(main())
