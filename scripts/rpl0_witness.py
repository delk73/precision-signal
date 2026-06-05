#!/usr/bin/env python3
"""Independent RPL0 witness for retained RPL0 v1 artifacts.

Parses an RPL0 v1 artifact through a standalone implementation path
separate from the main replay/comparison engine and emits a deterministic
witness report.

This script does not shell out to precision, replay-host, cargo, or make.
It does not import the existing replay/comparison engine.

Usage:
    python3 scripts/rpl0_witness.py path/to/artifact.rpl0
    python3 scripts/rpl0_witness.py path/to/artifact.rpl0 --out witness.txt

Exit codes:
    0  PASS
    1  FAIL (structural or validation error)
    2  Usage error
"""

import argparse
import hashlib
import struct
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# RPL0 v1 layout constants (independent local definitions)
# ---------------------------------------------------------------------------

MAGIC = b"RPL0"

# Minimum v1 header size (0x98 = 152 bytes)
V1_MIN_HEADER_LEN = 0x98

# v1 header field offsets
_OFF_MAGIC = 0x00          # [u8; 4]
_OFF_VERSION = 0x04        # u16 little-endian
_OFF_HEADER_LEN = 0x06     # u16 little-endian
_OFF_FRAME_COUNT = 0x08    # u32 little-endian
_OFF_FRAME_SIZE = 0x0C     # u16 little-endian
_OFF_FLAGS = 0x0E          # u16 little-endian
_OFF_SCHEMA_LEN = 0x10     # u32 little-endian
_OFF_SCHEMA_HASH = 0x14    # [u8; 32]
# Safe after v1 minimum header_len check.
_OFF_RESERVED = 0x96       # u16 little-endian

# Frame layout (16 bytes, EventFrame0)
FRAME_SIZE = 16
_FRAME_FMT = "<IBBHIi"  # frame_idx, irq_id, flags, rsv, timer_delta, input_sample

# Sanity caps (not format invariants, just witness acceptance limits)
_MAX_FRAME_COUNT = 1_000_000
_MAX_SCHEMA_LEN = 65_536
_MAX_HEADER_LEN = 65_536

# ---------------------------------------------------------------------------
# Deterministic witness digest
# ---------------------------------------------------------------------------
# Independent fold using local constants. Must not reuse the canonical
# replay hash (SHA-256 over [HEADER][SCHEMA][FRAMES]).

_DIGEST_INIT = 0x517A_F3C9_B20E_4D81
_DIGEST_MASK = (1 << 64) - 1
_DIGEST_PRIME = 0x9E37_79B9_7F4A_7C15


def _fold(state: int, value: int, bits: int) -> int:
    mask = (1 << bits) - 1
    state = ((state ^ (value & mask)) * _DIGEST_PRIME) & _DIGEST_MASK
    return state


def _witness_digest(frames: list[tuple]) -> str:
    """Deterministic fold over parsed frame tuples.

    Each tuple is (frame_idx, irq_id, flags, rsv, timer_delta, input_sample).
    input_sample is i32; fold as unsigned 32-bit.
    """
    state = _DIGEST_INIT
    for frame_idx, irq_id, flags, rsv, timer_delta, input_sample in frames:
        state = _fold(state, frame_idx, 32)
        state = _fold(state, irq_id, 8)
        state = _fold(state, flags, 8)
        state = _fold(state, rsv, 16)
        state = _fold(state, timer_delta, 32)
        # treat i32 as u32 for fold
        state = _fold(state, input_sample & 0xFFFF_FFFF, 32)
    return f"{state:016x}"


# ---------------------------------------------------------------------------
# Parse
# ---------------------------------------------------------------------------

WITNESS_ID = "rpl0-independent-witness-v1"
CLAIM = "independent parse and deterministic fold of retained RPL0 artifact"


def _format_optional(value: int | None) -> str:
    if value is None:
        return "none"
    return str(value)


def _fail_report(
    path: str,
    error: str,
    frame_size: int | None = None,
    frame_count: int | None = None,
    first_invalid_frame: str = "none",
) -> str:
    return (
        f"RESULT: FAIL\n"
        f"ARTIFACT_COUNT: 1\n"
        f"FRAME_SIZE: {_format_optional(frame_size)}\n"
        f"FRAME_COUNT: {_format_optional(frame_count)}\n"
        f"WITNESS_DIGEST: none\n"
        f"FIRST_INVALID_FRAME: {first_invalid_frame}\n"
        f"ERROR: {error}\n"
        f"WITNESS: {WITNESS_ID}\n"
        f"INPUT: {path}\n"
        f"CLAIM: {CLAIM}\n"
    )


def _pass_report(
    path: str,
    header_len: int,
    schema_len: int,
    frame_count: int,
    digest: str,
) -> str:
    return (
        f"RESULT: PASS\n"
        f"ARTIFACT_COUNT: 1\n"
        f"FRAME_SIZE: {FRAME_SIZE}\n"
        f"FRAME_COUNT: {frame_count}\n"
        f"WITNESS_DIGEST: {digest}\n"
        f"FIRST_INVALID_FRAME: none\n"
        f"WITNESS: {WITNESS_ID}\n"
        f"INPUT: {path}\n"
        f"FORMAT: RPL0/v1\n"
        f"HEADER_LEN: {header_len}\n"
        f"SCHEMA_LEN: {schema_len}\n"
        f"CLAIM: {CLAIM}\n"
    )


def witness(data: bytes, path: str) -> str:
    """Parse data as an RPL0 v1 artifact and return a witness report string."""

    # --- magic ---
    if len(data) < 4:
        return _fail_report(path, "truncated: file too short to contain magic")
    if data[0:4] != MAGIC:
        return _fail_report(path, f"bad magic: {data[0:4]!r}")

    # --- version dispatch ---
    if len(data) < 6:
        return _fail_report(path, "truncated: file too short to read version")
    version = struct.unpack_from("<H", data, _OFF_VERSION)[0]
    if version != 1:
        return _fail_report(path, f"unsupported version: {version} (witness accepts v1 only)")

    # --- header_len ---
    if len(data) < 8:
        return _fail_report(path, "truncated: file too short to read header_len")
    header_len = struct.unpack_from("<H", data, _OFF_HEADER_LEN)[0]
    if header_len < V1_MIN_HEADER_LEN:
        return _fail_report(
            path,
            f"header_len {header_len} below minimum {V1_MIN_HEADER_LEN}",
        )
    if header_len > _MAX_HEADER_LEN:
        return _fail_report(path, f"header_len {header_len} exceeds sanity cap {_MAX_HEADER_LEN}")
    if len(data) < header_len:
        return _fail_report(path, "truncated: file shorter than declared header_len")

    # --- frame_count / frame_size ---
    frame_count = struct.unpack_from("<I", data, _OFF_FRAME_COUNT)[0]
    frame_size = struct.unpack_from("<H", data, _OFF_FRAME_SIZE)[0]
    if frame_size != FRAME_SIZE:
        return _fail_report(
            path,
            f"unexpected frame_size {frame_size} (witness expects {FRAME_SIZE})",
            frame_size=frame_size,
            frame_count=frame_count,
        )
    if frame_count > _MAX_FRAME_COUNT:
        return _fail_report(
            path,
            f"frame_count {frame_count} exceeds sanity cap {_MAX_FRAME_COUNT}",
            frame_size=frame_size,
            frame_count=frame_count,
        )

    # --- flags / reserved ---
    flags = struct.unpack_from("<H", data, _OFF_FLAGS)[0]
    if flags != 0:
        return _fail_report(
            path,
            f"unsupported flags value {flags} (expected 0)",
            frame_size=frame_size,
            frame_count=frame_count,
        )
    reserved = struct.unpack_from("<H", data, _OFF_RESERVED)[0]
    if reserved != 0:
        return _fail_report(
            path,
            f"reserved must be 0 for v1 (got {reserved})",
            frame_size=frame_size,
            frame_count=frame_count,
        )

    # --- schema_len / schema region ---
    schema_len = struct.unpack_from("<I", data, _OFF_SCHEMA_LEN)[0]
    if schema_len > _MAX_SCHEMA_LEN:
        return _fail_report(
            path,
            f"schema_len {schema_len} exceeds sanity cap {_MAX_SCHEMA_LEN}",
            frame_size=frame_size,
            frame_count=frame_count,
        )
    schema_end = header_len + schema_len
    if len(data) < schema_end:
        return _fail_report(
            path,
            f"truncated: file too short to contain schema (need {schema_end}, have {len(data)})",
            frame_size=frame_size,
            frame_count=frame_count,
        )
    schema_block = data[header_len:schema_end]
    schema_hash = data[_OFF_SCHEMA_HASH : _OFF_SCHEMA_HASH + 32]
    computed_schema_hash = hashlib.sha256(schema_block).digest()
    if computed_schema_hash != schema_hash:
        return _fail_report(
            path,
            "schema_hash mismatch",
            frame_size=frame_size,
            frame_count=frame_count,
        )

    # --- frame region ---
    frame_region_start = schema_end
    frame_region_len = frame_count * FRAME_SIZE
    frame_region_end = frame_region_start + frame_region_len
    if len(data) < frame_region_end:
        return _fail_report(
            path,
            f"truncated: file too short to contain frame region "
            f"(need {frame_region_end}, have {len(data)})",
            frame_size=frame_size,
            frame_count=frame_count,
        )
    if len(data) != frame_region_end:
        return _fail_report(
            path,
            f"size mismatch: expected {frame_region_end}, got {len(data)}",
            frame_size=frame_size,
            frame_count=frame_count,
        )

    # --- parse frames ---
    frames: list[tuple] = []
    for i in range(frame_count):
        off = frame_region_start + i * FRAME_SIZE
        frame = struct.unpack_from(_FRAME_FMT, data, off)
        frame_idx = frame[0]
        if frame_idx != i:
            return _fail_report(
                path,
                f"non-monotonic frame_idx at position {i}: expected {i}, got {frame_idx}",
                frame_size=frame_size,
                frame_count=frame_count,
                first_invalid_frame=str(i),
            )
        frames.append(frame)

    digest = _witness_digest(frames)
    return _pass_report(path, header_len, schema_len, frame_count, digest)


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main() -> int:
    parser = argparse.ArgumentParser(
        description="Independent RPL0 v1 witness.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("artifact", help="Path to RPL0 artifact file.")
    parser.add_argument("--out", metavar="FILE", help="Write report to FILE (also printed to stdout).")
    args = parser.parse_args()

    artifact_path = Path(args.artifact)
    try:
        data = artifact_path.read_bytes()
    except OSError as exc:
        print(f"ERROR: cannot read {artifact_path}: {exc}", file=sys.stderr)
        return 2

    report = witness(data, str(artifact_path))
    print(report, end="")

    if args.out:
        try:
            Path(args.out).write_text(report, encoding="utf-8")
        except OSError as exc:
            print(f"ERROR: cannot write output file {args.out}: {exc}", file=sys.stderr)
            return 2

    return 0 if report.startswith("RESULT: PASS") else 1


if __name__ == "__main__":
    sys.exit(main())
