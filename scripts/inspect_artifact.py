#!/usr/bin/env python3
import argparse
import hashlib
import struct
import sys
from pathlib import Path

from tools.rpl0_constants import (
    FRAME_FMT,
    FRAME_SIZE,
    HEADER_LEN,
    MAGIC,
    V1_OFF_BOARD_ID,
    V1_OFF_BUILD_HASH,
    V1_OFF_CAPTURE_BOUNDARY,
    V1_OFF_CLOCK_PROFILE,
    V1_OFF_CONFIG_HASH,
    V1_OFF_FLAGS,
    V1_OFF_FRAME_COUNT,
    V1_OFF_FRAME_SIZE,
    V1_OFF_HEADER_LEN,
    V1_OFF_RESERVED,
    V1_OFF_SCHEMA_HASH,
    V1_OFF_SCHEMA_LEN,
    V1_OFF_VERSION,
)

HEADER_FMT = "<4sIII"
HEADER_SIZE = struct.calcsize(HEADER_FMT)
U64_MAX = (1 << 64) - 1

V1_MIN_HEADER_SIZE = HEADER_LEN

# Current contract policy: these fields are reserved for future use and must be zero.
REQUIRED_RESERVED_V0 = 0
REQUIRED_RESERVED_V1 = 0
REQUIRED_FLAGS_V1 = 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Inspect replay artifact contents.")
    parser.add_argument("artifact", type=Path, help="Artifact file path.")

    group = parser.add_mutually_exclusive_group()
    group.add_argument("--frame", type=int, help="Inspect a single frame index.")
    group.add_argument(
        "--frames",
        type=str,
        help="Inspect frame range as start:end (inclusive). Example: 0:5",
    )
    group.add_argument(
        "--summary",
        action="store_true",
        help="Print header plus first and last frame.",
    )

    parser.add_argument(
        "--raw",
        action="store_true",
        help="Print per-frame packed bytes in hex before decoded fields.",
    )
    return parser.parse_args()


def fail_invalid(reason: str) -> int:
    print(f"FAIL: invalid artifact structure ({reason})")
    return 1


def _u16_le(data: bytes, offset: int) -> int:
    return int.from_bytes(data[offset : offset + 2], "little", signed=False)


def _u32_le(data: bytes, offset: int) -> int:
    return int.from_bytes(data[offset : offset + 4], "little", signed=False)


def _checked_add_u64(a: int, b: int, label: str) -> int:
    total = a + b
    if total > U64_MAX:
        raise ValueError(f"{label} overflows u64")
    return total


def _checked_mul_u64(a: int, b: int, label: str) -> int:
    product = a * b
    if product > U64_MAX:
        raise ValueError(f"{label} overflows u64")
    return product


def _decode_frames(data: bytes, frame_count: int, frames_offset: int) -> list[dict]:
    frames = []
    offset = frames_offset
    for i in range(frame_count):
        chunk = data[offset : offset + FRAME_SIZE]
        try:
            frame_idx, irq_id, flags, rsv, timer_delta, input_sample = struct.unpack(
                FRAME_FMT, chunk
            )
        except struct.error as exc:
            raise ValueError(f"frame {i} unpack error: {exc}")
        frames.append(
            {
                "frame_idx": frame_idx,
                "irq_id": irq_id,
                "flags": flags,
                "rsv": rsv,
                "timer_delta": timer_delta,
                "input_sample": input_sample,
                "raw": chunk,
            }
        )
        offset += FRAME_SIZE
    return frames


def parse_artifact_bytes(data: bytes, allow_trailing: bool = False) -> dict:
    if len(data) < HEADER_SIZE:
        raise ValueError(f"file too short: {len(data)} < header {HEADER_SIZE}")

    magic = data[:4]
    if magic != MAGIC:
        raise ValueError(f"bad magic: {magic!r}")

    # Version dispatch permanence boundary:
    # - v0 and v1 overlap at offset 0x04 but use different field widths.
    # - v0 layout stores version as u32 at offset 0x04 with required value 0.
    # - v1 layout stores version as u16 at offset 0x04 with required value 1.
    # We intentionally read u32 first so legacy v0 artifacts dispatch deterministically.
    # Non-zero u32 enters v1 parsing; this is frozen compatibility logic.
    # Changing this rule requires a format-version break and test updates.
    version32 = _u32_le(data, 4)
    if version32 == 0:
        version = 0
        frame_count = _u32_le(data, 8)
        reserved = _u32_le(data, 12)
        if reserved != REQUIRED_RESERVED_V0:
            raise ValueError(
                f"reserved must be {REQUIRED_RESERVED_V0} for v0 (got {reserved})"
            )
        frame_bytes = _checked_mul_u64(frame_count, FRAME_SIZE, "frame region length")
        expected_size = _checked_add_u64(HEADER_SIZE, frame_bytes, "artifact length")
        if expected_size > len(data):
            raise ValueError(
                "truncated frame region: "
                f"need {expected_size} bytes from frame_count={frame_count}, got {len(data)}"
            )
        if not allow_trailing and len(data) != expected_size:
            raise ValueError(
                f"size mismatch: expected {expected_size} from frame_count={frame_count}, got {len(data)}"
            )

        frames = _decode_frames(data, frame_count, HEADER_SIZE)
        canonical = data[:expected_size]
        return {
            "magic": magic,
            "version": version,
            "header_len": HEADER_SIZE,
            "schema_len": 0,
            "frame_count": frame_count,
            "frame_size": FRAME_SIZE,
            "reserved": reserved,
            "frames_offset": HEADER_SIZE,
            "canonical_len": expected_size,
            "trailing_len": len(data) - expected_size,
            "schema_hash": None,
            "schema_hash_ok": None,
            "canonical_hash": hashlib.sha256(canonical).hexdigest(),
            "frames": frames,
        }

    if len(data) < V1_MIN_HEADER_SIZE:
        raise ValueError(f"file too short: {len(data)} < v1 min header {V1_MIN_HEADER_SIZE}")

    version = _u16_le(data, V1_OFF_VERSION)
    if version != 1:
        raise ValueError(f"unsupported version: {version}")

    header_len = _u16_le(data, V1_OFF_HEADER_LEN)
    frame_count = _u32_le(data, V1_OFF_FRAME_COUNT)
    frame_size = _u16_le(data, V1_OFF_FRAME_SIZE)
    flags = _u16_le(data, V1_OFF_FLAGS)
    schema_len = _u32_le(data, V1_OFF_SCHEMA_LEN)
    schema_hash = data[V1_OFF_SCHEMA_HASH : V1_OFF_SCHEMA_HASH + 32]
    build_hash = data[V1_OFF_BUILD_HASH : V1_OFF_BUILD_HASH + 32]
    config_hash = data[V1_OFF_CONFIG_HASH : V1_OFF_CONFIG_HASH + 32]
    board_id = data[V1_OFF_BOARD_ID : V1_OFF_BOARD_ID + 16]
    clock_profile = data[V1_OFF_CLOCK_PROFILE : V1_OFF_CLOCK_PROFILE + 16]
    capture_boundary = _u16_le(data, V1_OFF_CAPTURE_BOUNDARY)
    reserved = _u16_le(data, V1_OFF_RESERVED)

    if header_len < V1_MIN_HEADER_SIZE:
        raise ValueError(f"header_len too small: {header_len} < {V1_MIN_HEADER_SIZE}")
    if header_len > len(data):
        raise ValueError(f"header_len exceeds file length: {header_len} > {len(data)}")
    if frame_size != FRAME_SIZE:
        raise ValueError(f"invalid frame_size {frame_size} (expected {FRAME_SIZE})")
    if flags != REQUIRED_FLAGS_V1:
        raise ValueError(f"unsupported flags value {flags} (expected {REQUIRED_FLAGS_V1})")
    if reserved != REQUIRED_RESERVED_V1:
        raise ValueError(
            f"reserved must be {REQUIRED_RESERVED_V1} for v1 (got {reserved})"
        )

    schema_offset = header_len
    schema_end = _checked_add_u64(schema_offset, schema_len, "schema end")
    if schema_end > len(data):
        raise ValueError(
            f"schema region exceeds file length: schema_end={schema_end} > file_len={len(data)}"
        )

    frame_bytes = _checked_mul_u64(frame_count, frame_size, "frame region length")
    expected_size = _checked_add_u64(schema_end, frame_bytes, "artifact length")
    if expected_size > len(data):
        raise ValueError(
            f"truncated frame region: need {expected_size} bytes, got {len(data)}"
        )
    if not allow_trailing and expected_size != len(data):
        raise ValueError(f"size mismatch: expected {expected_size}, got {len(data)}")

    schema_block = data[schema_offset:schema_end]
    computed_schema_hash = hashlib.sha256(schema_block).digest()
    if computed_schema_hash != schema_hash:
        raise ValueError("schema_hash mismatch")

    frames = _decode_frames(data, frame_count, int(schema_end))
    canonical = data[:expected_size]
    return {
        "magic": magic,
        "version": version,
        "header_len": header_len,
        "frame_count": frame_count,
        "frame_size": frame_size,
        "flags": flags,
        "schema_len": schema_len,
        "schema_hash": schema_hash,
        "build_hash": build_hash,
        "config_hash": config_hash,
        "board_id": board_id,
        "clock_profile": clock_profile,
        "capture_boundary": capture_boundary,
        "reserved": reserved,
        "frames_offset": int(schema_end),
        "canonical_len": expected_size,
        "trailing_len": len(data) - expected_size,
        "schema_hash_ok": True,
        "canonical_hash": hashlib.sha256(canonical).hexdigest(),
        "frames": frames,
    }


def parse_artifact(path: Path, allow_trailing: bool = False):
    try:
        data = path.read_bytes()
    except OSError as exc:
        raise ValueError(f"read error: {exc}")
    return parse_artifact_bytes(data, allow_trailing=allow_trailing)


def parse_range(spec: str, max_idx: int) -> tuple[int, int]:
    if ":" not in spec:
        raise ValueError("range must be start:end")
    lhs, rhs = spec.split(":", 1)
    if lhs == "" or rhs == "":
        raise ValueError("range must include both start and end")
    start = int(lhs)
    end = int(rhs)
    if start < 0 or end < 0:
        raise ValueError("range bounds must be non-negative")
    if start > end:
        raise ValueError("range start must be <= end")
    if end > max_idx:
        raise ValueError(f"range end {end} out of bounds (max {max_idx})")
    return start, end


def print_header(path: Path, artifact: dict) -> None:
    print(f"Artifact: {path}")
    print()
    print("Header")
    print("------")
    print(f"magic: {artifact['magic'].decode('ascii', errors='replace')}")
    print(f"version: {artifact['version']}")
    print(f"header_len: {artifact['header_len']}")
    print(f"frame_count: {artifact['frame_count']}")
    print(f"frame_size: {artifact['frame_size']}")
    print(f"schema_len: {artifact['schema_len']}")
    print(f"frames_offset: {artifact['frames_offset']}")
    print(f"canonical_len: {artifact['canonical_len']}")
    if artifact["trailing_len"] > 0:
        print(f"trailing_len: {artifact['trailing_len']}")
    print(f"reserved: {artifact['reserved']}")
    if artifact["version"] == 1:
        print(f"flags: {artifact['flags']}")
        print(f"capture_boundary: {artifact['capture_boundary']}")
        print(f"schema_hash_ok: {artifact['schema_hash_ok']}")


def raw_hex(frame: dict) -> str:
    b = frame["raw"]
    return " ".join(
        [
            b[0:4].hex(),
            b[4:5].hex(),
            b[5:6].hex(),
            b[6:8].hex(),
            b[8:12].hex(),
            b[12:16].hex(),
        ]
    )


def print_frame_line(frame: dict, show_raw: bool) -> None:
    sample = frame["input_sample"] & 0xFFFFFFFF
    if show_raw:
        print(
            f"{frame['frame_idx']}  {raw_hex(frame)}  "
            f"irq={frame['irq_id']} flags={frame['flags']} "
            f"delta={frame['timer_delta']} sample=0x{sample:08X}"
        )
    else:
        print(
            f"{frame['frame_idx']}  irq={frame['irq_id']} flags={frame['flags']} "
            f"delta={frame['timer_delta']} sample=0x{sample:08X}"
        )


def print_single_frame(frame_no: int, frame: dict, show_raw: bool) -> None:
    print(f"Frame {frame_no}")
    print("---------")
    print(f"frame_idx={frame['frame_idx']}")
    print(f"irq_id={frame['irq_id']}")
    print(f"flags={frame['flags']}")
    print(f"rsv={frame['rsv']}")
    print(f"timer_delta={frame['timer_delta']}")
    print(f"input_sample=0x{(frame['input_sample'] & 0xFFFFFFFF):08X}")
    if show_raw:
        print(f"raw={raw_hex(frame)}")


def main() -> int:
    args = parse_args()

    try:
        artifact = parse_artifact(args.artifact)
    except ValueError as exc:
        return fail_invalid(str(exc))

    frame_count = artifact["frame_count"]
    print_header(args.artifact, artifact)
    print()

    frames = artifact["frames"]
    if frame_count == 0:
        print("No frame data.")
        return 0

    if args.summary:
        print("Summary")
        print("-------")
        print("first:")
        print_frame_line(frames[0], args.raw)
        print("last:")
        print_frame_line(frames[-1], args.raw)
        return 0

    if args.frame is not None:
        idx = args.frame
        if idx < 0 or idx >= frame_count:
            return fail_invalid(f"frame {idx} out of bounds (0..{frame_count - 1})")
        print_single_frame(idx, frames[idx], args.raw)
        return 0

    if args.frames is not None:
        try:
            start, end = parse_range(args.frames, frame_count - 1)
        except ValueError as exc:
            return fail_invalid(str(exc))
    else:
        start, end = 0, min(9, frame_count - 1)

    print(f"Frames ({start}..{end})")
    print("-------------")
    for i in range(start, end + 1):
        print_frame_line(frames[i], args.raw)
    return 0


if __name__ == "__main__":
    sys.exit(main())
