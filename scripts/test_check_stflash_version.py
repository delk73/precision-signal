#!/usr/bin/env python3
"""Regression tests for the st-flash version policy check."""

from __future__ import annotations

import stat
import subprocess
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent


def write_fake_stflash(path: Path, output: str) -> None:
    path.write_text(
        "#!/usr/bin/env sh\n"
        "if [ \"$1\" = \"--version\" ]; then\n"
        f"  printf '%s\\n' {output!r}\n"
        "  exit 0\n"
        "fi\n"
        "exit 2\n",
        encoding="utf-8",
    )
    path.chmod(path.stat().st_mode | stat.S_IXUSR)


def run_check(stflash: Path | str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["python3", "scripts/check_stflash_version.py", "--stflash", str(stflash)],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )


def assert_ok(name: str, proc: subprocess.CompletedProcess[str]) -> None:
    if proc.returncode != 0:
        raise AssertionError(
            f"{name}: expected success, rc={proc.returncode}\n"
            f"stdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )


def assert_fail(name: str, proc: subprocess.CompletedProcess[str], needle: str) -> None:
    if proc.returncode == 0:
        raise AssertionError(f"{name}: expected failure")
    combined = proc.stdout + proc.stderr
    if needle not in combined:
        raise AssertionError(
            f"{name}: missing {needle!r}\nstdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )


def makefile_wires_stflash_check() -> None:
    text = (REPO_ROOT / "Makefile").read_text(encoding="utf-8")
    if "REQUIRED_STFLASH_VERSION := 1.8.0" not in text:
        raise AssertionError("Makefile missing REQUIRED_STFLASH_VERSION pin")
    if "scripts/check_stflash_version.py --stflash" not in text:
        raise AssertionError("Makefile stflash-check does not invoke version script")
    if "--required-version \"$(REQUIRED_STFLASH_VERSION)\"" not in text:
        raise AssertionError("Makefile stflash-check does not pass required version")


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="dpw_stflash_version_") as tmp:
        root = Path(tmp)
        good = root / "st-flash-good"
        old = root / "st-flash-old"
        bad = root / "st-flash-bad"
        bare = root / "st-flash-bare"
        missing = root / "st-flash-missing"

        write_fake_stflash(good, "st-flash 1.8.0")
        write_fake_stflash(old, "st-flash 1.7.0")
        write_fake_stflash(bad, "st-flash release candidate")
        write_fake_stflash(bare, "v1.7.0")

        assert_ok("accepts_required_version", run_check(good))
        assert_fail(
            "rejects_old_version",
            run_check(old),
            "FAIL: st-flash version mismatch: required 1.8.0, found 1.7.0",
        )
        assert_fail(
            "rejects_bare_old_version",
            run_check(bare),
            "FAIL: st-flash version mismatch: required 1.8.0, found 1.7.0",
        )
        assert_fail("rejects_missing_binary", run_check(missing), "missing ST-LINK flash binary")
        assert_fail("rejects_unparsable_output", run_check(bad), "could not parse st-flash version")

    makefile_wires_stflash_check()
    print("PASS: st-flash version policy regression suite")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
