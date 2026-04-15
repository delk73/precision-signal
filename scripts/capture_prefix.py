#!/usr/bin/env python3

import argparse
import re
import time

import serial

STATE_PATTERN = re.compile(br"STATE,(CAPTURE_DONE|CAPTURE_INCOMPLETE),\d+\r?\n")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Capture raw bytes from serial-port open through first valid STATE line."
    )
    parser.add_argument("--serial", required=True)
    parser.add_argument("--baud", type=int, default=115200)
    parser.add_argument("--deadline", type=float, default=5.0)
    parser.add_argument("--read-size", type=int, default=256)
    parser.add_argument("--read-timeout", type=float, default=0.05)
    parser.add_argument("--settle-delay", type=float, default=0.0)
    parser.add_argument("--flush-after-open", action="store_true")
    parser.add_argument("--label", default="")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    ser = serial.Serial(args.serial, baudrate=args.baud, timeout=args.read_timeout)
    try:
        if args.settle_delay > 0:
            time.sleep(args.settle_delay)
        if args.flush_after_open:
            ser.reset_input_buffer()

        if args.label:
            print(f"LABEL {args.label}")
        print(f"FLUSH_AFTER_OPEN {args.flush_after_open}")
        print(f"SETTLE_DELAY_S {args.settle_delay}")
        print("ARM NOW: press reset")

        buf = bytearray()
        t0 = time.monotonic()

        while time.monotonic() - t0 < args.deadline:
            chunk = ser.read(args.read_size)
            if not chunk:
                continue
            buf += chunk
            match = STATE_PATTERN.search(buf)
            if match is None:
                continue

            prefix = bytes(buf[: match.start()])
            state_line = bytes(match.group(0))
            print(f"PREFIX_LEN {len(prefix)}")
            print(f"PREFIX_HEX {prefix.hex()}")
            print(f"PREFIX_REPR {prefix!r}")
            print(f"STATE_LINE_HEX {state_line.hex()}")
            print(f"STATE_LINE_REPR {state_line!r}")
            return 0

        print("NO_STATE_FOUND")
        return 1
    finally:
        ser.close()


if __name__ == "__main__":
    raise SystemExit(main())
