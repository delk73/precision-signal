#!/usr/bin/env python3

import argparse
import re
import time

import serial

STATE_PATTERN = re.compile(br"STATE,(CAPTURE_DONE|CAPTURE_INCOMPLETE),\d+\r?\n")
DEFAULT_READ_SIZE = 256
DEFAULT_READ_TIMEOUT = 0.05
DEFAULT_DEADLINE = 5.0
DEFAULT_SETTLE = 0.25
DEFAULT_COUNT = 5
INTER_TRIAL_DELAY = 0.25


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Persistently capture raw pre-STATE bytes across repeated manual resets."
    )
    parser.add_argument("--serial", required=True)
    parser.add_argument("--baud", type=int, default=115200)
    parser.add_argument("--deadline", type=float, default=DEFAULT_DEADLINE)
    parser.add_argument("--flush-after-open", action="store_true")
    parser.add_argument("--settle", type=float, default=DEFAULT_SETTLE)
    parser.add_argument("--count", type=int, default=DEFAULT_COUNT)
    parser.add_argument("--label", default="loop")
    parser.add_argument("--read-size", type=int, default=DEFAULT_READ_SIZE)
    parser.add_argument("--read-timeout", type=float, default=DEFAULT_READ_TIMEOUT)
    return parser.parse_args()


def capture_once(ser: serial.Serial, deadline_s: float, read_size: int, label: str) -> None:
    buf = bytearray()
    deadline = time.monotonic() + deadline_s

    while time.monotonic() < deadline:
        chunk = ser.read(read_size)
        if not chunk:
            continue
        buf += chunk
        match = STATE_PATTERN.search(buf)
        if match is None:
            continue

        prefix = bytes(buf[: match.start()])
        state_line = bytes(match.group(0))
        print(f"LABEL {label}")
        print(f"PREFIX_LEN {len(prefix)}")
        print(f"PREFIX_HEX {prefix.hex()}")
        print(f"PREFIX_REPR {prefix!r}")
        print(f"STATE_LINE_HEX {state_line.hex()}")
        print(f"STATE_LINE_REPR {state_line!r}")
        print("---")
        return

    print(f"LABEL {label}")
    print("NO_STATE_FOUND")
    print("---")


def main() -> int:
    args = parse_args()
    if args.count <= 0:
        raise SystemExit("--count must be > 0")

    with serial.Serial(args.serial, baudrate=args.baud, timeout=args.read_timeout) as ser:
        if args.settle > 0:
            time.sleep(args.settle)

        print("LISTENER_READY")
        print(f"FLUSH_AFTER_OPEN {args.flush_after_open}")
        print(f"SETTLE_DELAY_S {args.settle}")
        print(f"COUNT {args.count}")
        print()

        for index in range(1, args.count + 1):
            if args.flush_after_open:
                ser.reset_input_buffer()

            print(f"ARM NOW: trial {index}/{args.count}")
            capture_once(ser, args.deadline, args.read_size, f"{args.label}_{index}")
            if index != args.count:
                time.sleep(INTER_TRIAL_DELAY)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
