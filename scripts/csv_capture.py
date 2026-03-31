#!/usr/bin/env python3

import argparse
import subprocess
import sys
import time

import serial

DEFAULT_STFLASH = "st-flash"


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


def main() -> int:
    args = parse_args()
    received_rows = 0
    synced = False
    saw_state = False

    with serial.Serial(args.serial, args.baud, timeout=args.timeout) as ser, open(args.out, "wb") as out:
        ser.reset_input_buffer()
        if args.reset_mode == "stlink":
            print("Listener active; triggering ST-LINK reset", flush=True)
            time.sleep(args.reset_delay)
            trigger_stlink_reset(args.stflash)
        else:
            print("Listener active; press reset now", flush=True)

        while received_rows < args.rows:
            line = ser.readline()
            if not line:
                raise SystemExit("serial read timed out before capture completed")

            if line.startswith(b"STATE,"):
                decoded = line.decode("utf-8", errors="replace").rstrip("\r\n")
                print(decoded, flush=True)
                saw_state = True
                if decoded != f"STATE,CAPTURE_DONE,{args.rows}":
                    raise SystemExit(f"capture did not complete: {decoded}")
                continue

            if not synced:
                if line == b"index,interval_us\n":
                    out.write(line)
                    synced = True
                continue

            out.write(line)
            received_rows += 1

    if not saw_state:
        raise SystemExit("capture produced no STATE line")

    return 0


if __name__ == "__main__":
    sys.exit(main())
