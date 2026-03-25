import argparse
import hashlib
import os
import struct
from pathlib import Path

import serial

DEFAULT_SERIAL_PORT = "/dev/ttyACM0"
BAUD_RATE = 115200
DEFAULT_MAX_SYNC_BYTES = 65536
EXPECTED_FRAME_COUNT = 10_000
EXPECTED_IRQ_ID = 0x02
EXPECTED_TIMER_DELTA = 1000
SIGNAL_MODELS = ("none", "ramp", "phase8")

MAGIC = b"RPL0"
HEADER_FMT = "<4sIII"
FRAME_FMT = "<IBBHIi"
HEADER_SIZE = struct.calcsize(HEADER_FMT)
FRAME_SIZE = struct.calcsize(FRAME_FMT)
V1_MIN_HEADER_SIZE = 0x98
V1_OFF_VERSION = 0x04
V1_OFF_HEADER_LEN = 0x06
V1_OFF_FRAME_COUNT = 0x08
V1_OFF_FRAME_SIZE = 0x0C
V1_OFF_FLAGS = 0x0E
V1_OFF_SCHEMA_LEN = 0x10
V1_OFF_SCHEMA_HASH = 0x14
V1_OFF_CAPTURE_BOUNDARY = 0x94


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Read replay artifact stream over UART.")
    parser.add_argument(
        "--out",
        type=Path,
        help="Write raw stream ([HEADER][SCHEMA][FRAMES]) to this file path.",
    )
    parser.add_argument(
        "--quick",
        action="store_true",
        help="Print only header, frames 0..2, and the last frame.",
    )
    parser.add_argument(
        "--max-sync-bytes",
        type=int,
        default=DEFAULT_MAX_SYNC_BYTES,
        help="Fail if MAGIC is not found within this many bytes (default: 65536).",
    )
    parser.add_argument(
        "--no-verify",
        action="store_true",
        help="Disable structural frame verification checks.",
    )
    parser.add_argument(
        "--signal-model",
        choices=SIGNAL_MODELS,
        default="none",
        help="Optional input_sample validator: none (default), ramp, phase8.",
    )
    return parser.parse_args()


def expected_sample_for_model(frame_idx: int, model: str) -> int | None:
    if model == "none":
        return None
    if model == "ramp":
        return (frame_idx + 1) & 0xFFFFFFFF
    if model == "phase8":
        return frame_idx & 0xFF
    raise ValueError(f"unknown signal model: {model}")


def open_serial_port() -> serial.Serial:
    serial_port = os.environ.get("SERIAL", DEFAULT_SERIAL_PORT)
    print(f"Opening {serial_port} at {BAUD_RATE} baud.", flush=True)
    ser = serial.Serial(serial_port, BAUD_RATE, timeout=None)
    try:
        ser.reset_input_buffer()
    except serial.SerialException:
        pass
    return ser


def write_capture(path: Path | None, data: bytearray) -> None:
    if path is not None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)


def read_exact(ser: serial.Serial, count: int) -> bytes:
    return ser.read(count)


def print_frame(frame_idx: int, irq_id: int, timer_delta: int, input_sample: int) -> None:
    print(
        f"Frame {frame_idx:04d} | IRQ: {irq_id:02X} "
        f"| Delta: {timer_delta} | Sample: 0x{(input_sample & 0xFFFFFFFF):08X}"
    )


def verify_frame(expected_idx: int, irq_id: int, timer_delta: int, input_sample: int, model: str) -> str | None:
    if irq_id != EXPECTED_IRQ_ID:
        return f"unexpected irq_id {irq_id} at frame {expected_idx}"
    if timer_delta != EXPECTED_TIMER_DELTA:
        return f"unexpected timer_delta {timer_delta} at frame {expected_idx}"
    expected_sample = expected_sample_for_model(expected_idx, model)
    if expected_sample is None:
        return None
    got_sample = input_sample & 0xFFFFFFFF
    if got_sample != expected_sample:
        return (
            "sample mismatch at frame "
            f"{expected_idx} expected 0x{expected_sample:08X} got 0x{got_sample:08X}"
        )
    return None


def capture_v0(
    ser: serial.Serial,
    args: argparse.Namespace,
    raw_capture: bytearray,
    header_data: bytes,
) -> int:
    try:
        magic, version, frame_count, reserved = struct.unpack(HEADER_FMT, header_data)
    except struct.error as exc:
        print(f"Failed: bad header unpack: {exc}")
        write_capture(args.out, raw_capture)
        return 4

    if magic != MAGIC:
        print(f"Failed: bad header magic {magic!r}")
        write_capture(args.out, raw_capture)
        return 10
    if frame_count <= 0 or frame_count != EXPECTED_FRAME_COUNT:
        print(f"Failed: unexpected frame_count {frame_count}")
        write_capture(args.out, raw_capture)
        return 11

    print(
        f"Header: Magic={magic.decode('ascii', errors='replace')}, "
        f"Version={version}, Frame Count={frame_count}, Reserved={reserved}"
    )
    print("Reading frames...")

    for expected_idx in range(frame_count):
        frame_data = read_exact(ser, FRAME_SIZE)
        if len(frame_data) != FRAME_SIZE:
            print(
                f"Failed: short read at frame {expected_idx} "
                f"({len(frame_data)}/{FRAME_SIZE})"
            )
            write_capture(args.out, raw_capture)
            return 5
        raw_capture.extend(frame_data)

        try:
            frame_idx, irq_id, _flags, _rsv, timer_delta, input_sample = struct.unpack(
                FRAME_FMT, frame_data
            )
        except struct.error as exc:
            print(f"Failed: bad frame unpack at frame {expected_idx}: {exc}")
            write_capture(args.out, raw_capture)
            return 6

        if not args.no_verify:
            if frame_idx != expected_idx:
                print(f"Failed: frame_idx mismatch expected {expected_idx} got {frame_idx}")
                write_capture(args.out, raw_capture)
                return 20
            failure = verify_frame(
                expected_idx, irq_id, timer_delta, input_sample, args.signal_model
            )
            if failure is not None:
                print(f"Failed: {failure}")
                write_capture(args.out, raw_capture)
                return 21

        if not args.quick or expected_idx < 3 or expected_idx == frame_count - 1:
            print_frame(frame_idx, irq_id, timer_delta, input_sample)

    return 0


def capture_v1(
    ser: serial.Serial,
    args: argparse.Namespace,
    raw_capture: bytearray,
    header_prefix: bytes,
) -> int:
    header_tail = read_exact(ser, V1_MIN_HEADER_SIZE - HEADER_SIZE)
    if len(header_tail) != (V1_MIN_HEADER_SIZE - HEADER_SIZE):
        print(
            "Failed: short read for v1 header tail "
            f"({len(header_tail)}/{V1_MIN_HEADER_SIZE - HEADER_SIZE})"
        )
        write_capture(args.out, raw_capture)
        return 12
    raw_capture.extend(header_tail)
    header_data = bytearray(header_prefix + header_tail)

    version = int.from_bytes(header_data[V1_OFF_VERSION : V1_OFF_VERSION + 2], "little")
    header_len = int.from_bytes(header_data[V1_OFF_HEADER_LEN : V1_OFF_HEADER_LEN + 2], "little")
    frame_count = int.from_bytes(
        header_data[V1_OFF_FRAME_COUNT : V1_OFF_FRAME_COUNT + 4], "little"
    )
    frame_size = int.from_bytes(
        header_data[V1_OFF_FRAME_SIZE : V1_OFF_FRAME_SIZE + 2], "little"
    )
    flags = int.from_bytes(header_data[V1_OFF_FLAGS : V1_OFF_FLAGS + 2], "little")
    schema_len = int.from_bytes(
        header_data[V1_OFF_SCHEMA_LEN : V1_OFF_SCHEMA_LEN + 4], "little"
    )
    schema_hash = bytes(header_data[V1_OFF_SCHEMA_HASH : V1_OFF_SCHEMA_HASH + 32])
    capture_boundary = int.from_bytes(
        header_data[V1_OFF_CAPTURE_BOUNDARY : V1_OFF_CAPTURE_BOUNDARY + 2], "little"
    )

    if version != 1:
        print(f"Failed: unsupported version {version}")
        write_capture(args.out, raw_capture)
        return 13
    if header_len < V1_MIN_HEADER_SIZE:
        print(f"Failed: header_len too small {header_len}")
        write_capture(args.out, raw_capture)
        return 14
    if frame_size != FRAME_SIZE:
        print(f"Failed: invalid frame_size {frame_size}")
        write_capture(args.out, raw_capture)
        return 15
    if frame_count <= 0 or frame_count != EXPECTED_FRAME_COUNT:
        print(f"Failed: unexpected frame_count {frame_count}")
        write_capture(args.out, raw_capture)
        return 11

    if header_len > V1_MIN_HEADER_SIZE:
        extra = read_exact(ser, header_len - V1_MIN_HEADER_SIZE)
        if len(extra) != (header_len - V1_MIN_HEADER_SIZE):
            print(
                "Failed: short read for extended header "
                f"({len(extra)}/{header_len - V1_MIN_HEADER_SIZE})"
            )
            write_capture(args.out, raw_capture)
            return 16
        raw_capture.extend(extra)
        header_data.extend(extra)

    schema_block = read_exact(ser, schema_len)
    if len(schema_block) != schema_len:
        print(f"Failed: short read for schema block ({len(schema_block)}/{schema_len})")
        write_capture(args.out, raw_capture)
        return 17
    raw_capture.extend(schema_block)

    if not args.no_verify:
        computed = hashlib.sha256(schema_block).digest()
        if computed != schema_hash:
            print("Failed: schema_hash mismatch")
            write_capture(args.out, raw_capture)
            return 18

    print(
        f"Header: Magic={MAGIC.decode('ascii')}, Version={version}, HeaderLen={header_len}, "
        f"SchemaLen={schema_len}, Frame Count={frame_count}, FrameSize={frame_size}, "
        f"Flags={flags}, CaptureBoundary={capture_boundary}"
    )
    print("Reading frames...")

    for expected_idx in range(frame_count):
        frame_data = read_exact(ser, FRAME_SIZE)
        if len(frame_data) != FRAME_SIZE:
            print(
                f"Failed: short read at frame {expected_idx} "
                f"({len(frame_data)}/{FRAME_SIZE})"
            )
            write_capture(args.out, raw_capture)
            return 5
        raw_capture.extend(frame_data)

        try:
            frame_idx, irq_id, _flags, _rsv, timer_delta, input_sample = struct.unpack(
                FRAME_FMT, frame_data
            )
        except struct.error as exc:
            print(f"Failed: bad frame unpack at frame {expected_idx}: {exc}")
            write_capture(args.out, raw_capture)
            return 6

        if not args.no_verify:
            if frame_idx != expected_idx:
                print(f"Failed: frame_idx mismatch expected {expected_idx} got {frame_idx}")
                write_capture(args.out, raw_capture)
                return 20
            failure = verify_frame(
                expected_idx, irq_id, timer_delta, input_sample, args.signal_model
            )
            if failure is not None:
                print(f"Failed: {failure}")
                write_capture(args.out, raw_capture)
                return 21

        if not args.quick or expected_idx < 3 or expected_idx == frame_count - 1:
            print_frame(frame_idx, irq_id, timer_delta, input_sample)

    return 0


def main() -> int:
    args = parse_args()
    raw_capture = bytearray()

    try:
        with open_serial_port() as ser:
            print("Listener active; press reset now", flush=True)
            sync_buffer = bytearray()
            total_scanned = 0
            while total_scanned < args.max_sync_bytes:
                b = ser.read(1)
                if len(b) != 1:
                    continue
                total_scanned += 1
                sync_buffer += b
                if len(sync_buffer) > len(MAGIC):
                    del sync_buffer[:-len(MAGIC)]
                if bytes(sync_buffer) == MAGIC:
                    print("Found MAGIC 'RPL0'.")
                    raw_capture.extend(MAGIC)
                    break
            else:
                print(f"Failed: MAGIC not found within {args.max_sync_bytes} bytes")
                return 2

            rest = read_exact(ser, HEADER_SIZE - len(MAGIC))
            if len(rest) != (HEADER_SIZE - len(MAGIC)):
                print(
                    f"Failed: short read for header tail ({len(rest)}/{HEADER_SIZE - len(MAGIC)})"
                )
                write_capture(args.out, raw_capture)
                return 3

            header_prefix = MAGIC + rest
            raw_capture.extend(rest)
            version32 = int.from_bytes(header_prefix[4:8], "little")
            if version32 == 0:
                rc = capture_v0(ser, args, raw_capture, header_prefix)
            else:
                rc = capture_v1(ser, args, raw_capture, header_prefix)
            if rc != 0:
                return rc

            if args.out is not None:
                write_capture(args.out, raw_capture)
                print(f"Wrote {len(raw_capture)} bytes to {args.out}")

            print("Artifact extraction complete.")
            return 0

    except serial.SerialException as exc:
        print(f"Failed: serial open/read error: {exc}")
        return 7
    except OSError as exc:
        print(f"Failed: OS error: {exc}")
        return 8
    except Exception as exc:  # pragma: no cover - manual hardware script
        print(f"Failed: {exc}")
        return 9


if __name__ == "__main__":
    raise SystemExit(main())
