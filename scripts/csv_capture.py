#!/usr/bin/env python3

import argparse
import sys

import serial


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--serial", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--rows", type=int, default=138)
    parser.add_argument("--baud", type=int, default=115200)
    parser.add_argument("--timeout", type=float, default=10.0)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    received_rows = 0
    synced = False

    with serial.Serial(args.serial, args.baud, timeout=args.timeout) as ser, open(
        args.out, "wb"
    ) as out:
        while received_rows < args.rows:
            line = ser.readline()
            if not line:
                raise SystemExit("serial read timed out before capture completed")

            if not synced:
                if line == b"index,interval_us\n":
                    out.write(line)
                    synced = True
                continue

            out.write(line)
            received_rows += 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
