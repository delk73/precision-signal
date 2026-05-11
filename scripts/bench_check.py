#!/usr/bin/env python3
"""Check host and bench readiness for STM32 release operations."""

from __future__ import annotations

import argparse
import os
import shutil
import stat
import subprocess
import sys
from pathlib import Path


STLINK_VENDOR_ID = "0483"
STLINK_PRODUCT_IDS = {
    "3744",
    "3748",
    "374b",
    "374d",
    "374e",
    "374f",
    "3752",
    "3753",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Check bench readiness for release operations.")
    parser.add_argument("--serial", required=True)
    parser.add_argument("--stflash", required=True)
    parser.add_argument("--cargo", required=True)
    parser.add_argument("--python", required=True)
    parser.add_argument("--make", required=True)
    parser.add_argument("--fw-elf", required=True, type=Path)
    parser.add_argument("--fw-bin", required=True, type=Path)
    parser.add_argument(
        "--firmware-artifacts",
        choices=("required", "skip"),
        default="required",
        help="whether prebuilt firmware ELF/BIN artifacts are required",
    )
    return parser.parse_args()


def ok(message: str) -> None:
    print(f"OK: {message}")


def add_missing_binary(errors: list[str], name: str, value: str, hint: str) -> None:
    if shutil.which(value) is None:
        errors.append(
            f"missing required binary for {name}: {value!r}. {hint}"
        )


def serial_errors(serial_path: str) -> list[str]:
    errors: list[str] = []
    if not serial_path:
        return [
            "missing serial device: SERIAL is empty. "
            "Set SERIAL=/dev/ttyACM0 or the ST-LINK VCP device path."
        ]

    path = Path(serial_path)
    if not path.exists():
        return [
            f"missing serial device: SERIAL={serial_path} does not exist. "
            "Attach the STM32 ST-LINK VCP, then retry or pass SERIAL=/dev/ttyACM<N>."
        ]
    try:
        mode = path.stat().st_mode
    except OSError as exc:
        return [
            f"cannot inspect serial device: SERIAL={serial_path}: {exc}. "
            "Check device permissions and that the board remains attached."
        ]
    if not stat.S_ISCHR(mode):
        errors.append(
            f"serial device is not a character device: SERIAL={serial_path}. "
            "Pass the actual ST-LINK VCP device, usually /dev/ttyACM0."
        )
    if not os.access(path, os.R_OK | os.W_OK):
        errors.append(
            f"serial device is not readable/writable by this user: SERIAL={serial_path}. "
            "Fix dialout/udev permissions or run from a user with access."
        )
    return errors


def sysfs_has_stlink() -> bool:
    usb_root = Path("/sys/bus/usb/devices")
    if not usb_root.is_dir():
        return False
    for device in usb_root.iterdir():
        vendor = device / "idVendor"
        product = device / "idProduct"
        if not vendor.is_file() or not product.is_file():
            continue
        try:
            vendor_id = vendor.read_text(encoding="utf-8").strip().lower()
            product_id = product.read_text(encoding="utf-8").strip().lower()
        except OSError:
            continue
        if vendor_id == STLINK_VENDOR_ID and product_id in STLINK_PRODUCT_IDS:
            return True
    return False


def stinfo_has_probe() -> bool | None:
    stinfo = shutil.which("st-info")
    if stinfo is None:
        return None
    result = subprocess.run(
        [stinfo, "--probe"],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        return False
    output = result.stdout.lower()
    return "serial" in output or "stlink" in output or "st-link" in output


def firmware_artifact_errors(fw_elf: Path, fw_bin: Path) -> list[str]:
    errors: list[str] = []
    for label, path in (("firmware ELF", fw_elf), ("firmware BIN", fw_bin)):
        if not path.is_file():
            errors.append(
                f"missing {label} build artifact: {path}. Run `make fw-bin` before bench-check."
            )
            continue
        if path.stat().st_size == 0:
            errors.append(
                f"empty {label} build artifact: {path}. Rebuild with `make fw-bin`."
            )
    return errors


def main() -> int:
    args = parse_args()
    errors: list[str] = []

    add_missing_binary(errors, "Python", args.python, "Install python3 or pass PYTHON=/path/to/python3.")
    add_missing_binary(errors, "Cargo", args.cargo, "Install Rust/Cargo or pass CARGO=/path/to/cargo.")
    add_missing_binary(errors, "Make", args.make, "Install make or pass MAKE_CMD=/path/to/make.")
    add_missing_binary(errors, "ST-LINK flash", args.stflash, "Install stlink tools or pass STFLASH=/path/to/st-flash.")
    add_missing_binary(errors, "timeout", "timeout", "Install GNU coreutils; firmware capture uses timeout.")
    add_missing_binary(errors, "sha256sum", "sha256sum", "Install GNU coreutils; release summaries use sha256sum-compatible hashing.")

    errors.extend(serial_errors(args.serial))

    probe_state = stinfo_has_probe()
    if probe_state is False:
        errors.append(
            "missing ST-LINK: `st-info --probe` did not find a probe. "
            "Attach the STM32 board/ST-LINK and check USB permissions."
        )
    elif probe_state is None and not sysfs_has_stlink():
        errors.append(
            "missing ST-LINK: no ST-LINK USB probe was detected. "
            "Attach the STM32 board/ST-LINK; install `st-info` for a stronger probe check if needed."
        )

    if args.firmware_artifacts == "required":
        errors.extend(firmware_artifact_errors(args.fw_elf, args.fw_bin))

    if errors:
        print("FAIL: bench readiness check")
        for error in errors:
            print(f"- {error}")
        return 1

    ok(f"serial device present: {args.serial}")
    ok("ST-LINK probe detected")
    ok("required host binaries present")
    if args.firmware_artifacts == "required":
        ok(f"firmware artifacts present: {args.fw_elf}, {args.fw_bin}")
    else:
        ok("firmware artifact check skipped; release-proof will build firmware")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
