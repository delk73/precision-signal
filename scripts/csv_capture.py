#!/usr/bin/env python3

import argparse
import re
import subprocess
import sys
import time

import serial

DEFAULT_STFLASH = "st-flash"
DEFAULT_SETTLE_DELAY = 0.25
PREAMBLE_READ_SLICE = 0.25
CSV_HEADER = b"index,interval_us\n"
STATE_PATTERN = re.compile(r"STATE,(CAPTURE_DONE|CAPTURE_INCOMPLETE),(\d+)")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--serial", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--rows", type=int, default=138)
    parser.add_argument("--baud", type=int, default=115200)
    parser.add_argument("--timeout", type=float, default=10.0)
    parser.add_argument(
        "--reset-mode",
        choices=("manual", "stlink"),
        default="manual",
    )
    parser.add_argument("--stflash", default=DEFAULT_STFLASH)
    parser.add_argument("--reset-delay", type=float, default=0.25)
    parser.add_argument("--settle-delay", type=float, default=DEFAULT_SETTLE_DELAY)
    return parser.parse_args()


def trigger_stlink_reset(stflash: str) -> None:
    proc = subprocess.run(
        [stflash, "--connect-under-reset", "--freq=200K", "reset"],
        capture_output=True,
        text=True,
    )
    if proc.stdout:
        print(proc.stdout, end="" if proc.stdout.endswith("\n") else "\n")
    if proc.stderr:
        print(proc.stderr, end="" if proc.stderr.endswith("\n") else "\n", file=sys.stderr)
    if proc.returncode != 0:
        raise SystemExit("stlink reset failed")


def decode_line(raw: bytes) -> str:
    return raw.decode("utf-8", errors="replace").rstrip("\r\n")


def read_valid_state_line(ser: serial.Serial, timeout: float) -> str:
    deadline = time.monotonic() + timeout
    original_timeout = ser.timeout
    ser.timeout = min(timeout, PREAMBLE_READ_SLICE)

    try:
        while time.monotonic() < deadline:
            line = ser.readline()
            if not line:
                continue

            decoded = decode_line(line)
            match = STATE_PATTERN.search(decoded)
            if match is None:
                continue

            return f"STATE,{match.group(1)},{match.group(2)}"
    finally:
        ser.timeout = original_timeout

    raise SystemExit("no STATE preamble observed within timeout")


def read_csv_header(ser: serial.Serial, timeout: float) -> bytes:
    deadline = time.monotonic() + timeout
    original_timeout = ser.timeout
    ser.timeout = min(timeout, PREAMBLE_READ_SLICE)

    try:
        while time.monotonic() < deadline:
            line = ser.readline()
            if not line:
                continue
            if line == CSV_HEADER:
                return line
    finally:
        ser.timeout = original_timeout

    raise SystemExit("serial read timed out before csv header")


def main() -> int:
    args = parse_args()
    received_rows = 0

    with serial.Serial(args.serial, args.baud, timeout=args.timeout) as ser, open(args.out, "wb") as out:
        time.sleep(args.settle_delay)
        ser.reset_input_buffer()
        if args.reset_mode == "stlink":
            print("Listener active; triggering ST-LINK reset", flush=True)
            time.sleep(args.reset_delay)
            trigger_stlink_reset(args.stflash)
        else:
            print("Listener active; press reset now", flush=True)

        state_line = read_valid_state_line(ser, args.timeout)
        print(state_line, flush=True)
        if state_line != f"STATE,CAPTURE_DONE,{args.rows}":
            raise SystemExit(f"capture did not complete: {state_line}")

        out.write(read_csv_header(ser, args.timeout))

        while received_rows < args.rows:
            line = ser.readline()
            if not line:
                raise SystemExit("serial read timed out before capture completed")
            out.write(line)
            received_rows += 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
