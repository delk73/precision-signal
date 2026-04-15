#!/usr/bin/env python3

import argparse
import re

import serial

STATE_PATTERN = re.compile(br"STATE,(CAPTURE_DONE|CAPTURE_INCOMPLETE),\d+\r?\n")
DEFAULT_BAUD = 115200
DEFAULT_WINDOW = 16
DEFAULT_READ_SIZE = 256
DEFAULT_READ_TIMEOUT = 0.05
DEFAULT_MAX_EVENTS = 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Continuously monitor raw UART bytes and report pre-STATE context."
    )
    parser.add_argument("--serial", required=True)
    parser.add_argument("--baud", type=int, default=DEFAULT_BAUD)
    parser.add_argument("--window", type=int, default=DEFAULT_WINDOW)
    parser.add_argument("--max-events", type=int, default=DEFAULT_MAX_EVENTS)
    parser.add_argument("--read-size", type=int, default=DEFAULT_READ_SIZE)
    parser.add_argument("--read-timeout", type=float, default=DEFAULT_READ_TIMEOUT)
    return parser.parse_args()


def validate_args(args: argparse.Namespace) -> None:
    if args.window < 0:
        raise SystemExit("--window must be >= 0")
    if args.max_events < 0:
        raise SystemExit("--max-events must be >= 0")
    if args.read_size <= 0:
        raise SystemExit("--read-size must be > 0")
    if args.read_timeout <= 0:
        raise SystemExit("--read-timeout must be > 0")


def maybe_trim_buffer(buf: bytearray, window: int) -> None:
    max_len = max(window, 1)
    if len(buf) > max_len:
        del buf[:-max_len]


def print_event(event_count: int, prev_byte: bytes, prefix: bytes, state_line: bytes) -> None:
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


def read_next_event(
    ser: serial.Serial, buf: bytearray, *, window: int, read_size: int
) -> tuple[bytes, bytes, bytes]:
    while True:
        match = STATE_PATTERN.search(buf)
        if match is not None:
            start = match.start()
            end = match.end()
            state_line = bytes(match.group(0))
            prefix = bytes(buf[max(0, start - window) : start])
            prev_byte = bytes(buf[start - 1 : start]) if start > 0 else b""
            del buf[:end]
            return prev_byte, prefix, state_line

        chunk = ser.read(read_size)
        if not chunk:
            maybe_trim_buffer(buf, window)
            continue

        buf += chunk


def main() -> int:
    args = parse_args()
    validate_args(args)

    with serial.Serial(args.serial, baudrate=args.baud, timeout=args.read_timeout) as ser:
        buf = bytearray()
        event_count = 0

        print("MONITOR_READY (press reset anytime, Ctrl-C to stop)")
        print()

        try:
            while True:
                event = read_next_event(
                    ser,
                    buf,
                    window=args.window,
                    read_size=args.read_size,
                )
                event_count += 1
                prev_byte, prefix, state_line = event
                print_event(event_count, prev_byte, prefix, state_line)

                if args.max_events and event_count >= args.max_events:
                    return 0
        except KeyboardInterrupt:
            return 0


if __name__ == "__main__":
    raise SystemExit(main())
