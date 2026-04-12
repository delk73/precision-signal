#!/usr/bin/env python3

import argparse
import re

import serial

STATE_PATTERN = re.compile(br"STATE,(CAPTURE_DONE|CAPTURE_INCOMPLETE),\d+\r?\n")
DEFAULT_BAUD = 115200
DEFAULT_WINDOW = 16
DEFAULT_READ_SIZE = 256
DEFAULT_READ_TIMEOUT = 0.05


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Continuously monitor raw UART bytes and report pre-STATE context."
    )
    parser.add_argument("--serial", required=True)
    parser.add_argument("--baud", type=int, default=DEFAULT_BAUD)
    parser.add_argument("--window", type=int, default=DEFAULT_WINDOW)
    parser.add_argument("--max-events", type=int, default=0)
    parser.add_argument("--read-size", type=int, default=DEFAULT_READ_SIZE)
    parser.add_argument("--read-timeout", type=float, default=DEFAULT_READ_TIMEOUT)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.window < 0:
        raise SystemExit("--window must be >= 0")
    if args.max_events < 0:
        raise SystemExit("--max-events must be >= 0")

    with serial.Serial(args.serial, baudrate=args.baud, timeout=args.read_timeout) as ser:
        buf = bytearray()
        event_count = 0

        print("MONITOR_READY (press reset anytime, Ctrl-C to stop)")
        print()

        try:
            while True:
                chunk = ser.read(args.read_size)
                if not chunk:
                    continue

                buf += chunk

                while True:
                    match = STATE_PATTERN.search(buf)
                    if match is None:
                        if len(buf) > args.window:
                            del buf[:-args.window]
                        break

                    start = match.start()
                    end = match.end()
                    state_line = bytes(match.group(0))
                    prefix = bytes(buf[max(0, start - args.window) : start])
                    prev_byte = bytes(buf[start - 1 : start]) if start > 0 else b""

                    event_count += 1
                    print(f"EVENT {event_count}")
                    if prev_byte:
                        print(f"PREV_BYTE_HEX {prev_byte.hex()}")
                        print(f"PREV_BYTE_REPR {prev_byte!r}")
                    else:
                        print("PREV_BYTE_HEX none")
                        print("PREV_BYTE_REPR none")
                    print(f"PRE_STATE_WINDOW_HEX {prefix.hex()}")
                    print(f"STATE_LINE_HEX {state_line.hex()}")
                    print(f"STATE_LINE_REPR {state_line!r}")
                    print("---")

                    del buf[:end]

                    if args.max_events and event_count >= args.max_events:
                        return 0
        except KeyboardInterrupt:
            return 0


if __name__ == "__main__":
    raise SystemExit(main())
