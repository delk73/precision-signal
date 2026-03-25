#!/usr/bin/env python3
import argparse
import hashlib
import sys
from pathlib import Path

import inspect_artifact

HEADER_SIZE = inspect_artifact.HEADER_SIZE
FRAME_SIZE = inspect_artifact.FRAME_SIZE


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Compare two replay artifacts and report first semantic divergence."
    )
    parser.add_argument("baseline", type=Path, help="Baseline artifact path.")
    parser.add_argument("candidate", type=Path, help="Candidate artifact path.")
    return parser.parse_args()


def sha256_bytes(data: bytes) -> str:
    h = hashlib.sha256()
    h.update(data)
    return h.hexdigest()


def parse_artifact(path: Path) -> tuple[dict, list[dict], str | None]:
    try:
        parsed = inspect_artifact.parse_artifact(path, allow_trailing=False)
    except ValueError as exc:
        return {}, [], f"parse error: {exc}"

    header = {
        "magic": parsed["magic"],
        "version": parsed["version"],
        "frame_count": parsed["frame_count"],
        "reserved": parsed["reserved"],
        "frame_size": parsed["frame_size"],
        "schema_len": parsed["schema_len"],
        "header_len": parsed["header_len"],
    }
    return header, parsed["frames"], None


def format_magic(value: bytes) -> str:
    return value.decode("ascii", errors="replace")


def format_frame_line(frame: dict) -> str:
    sample_u32 = frame["input_sample"] & 0xFFFFFFFF
    return (
        f"frame_idx={frame['frame_idx']} irq_id={frame['irq_id']} "
        f"flags={frame['flags']} timer_delta={frame['timer_delta']} "
        f"input_sample=0x{sample_u32:08X}"
    )


def main() -> int:
    args = parse_args()

    try:
        baseline_bytes = args.baseline.read_bytes()
    except OSError as exc:
        print(f"FAIL: cannot read baseline: {exc}")
        return 2

    try:
        candidate_bytes = args.candidate.read_bytes()
    except OSError as exc:
        print(f"FAIL: cannot read candidate: {exc}")
        return 2

    if baseline_bytes == candidate_bytes:
        digest = sha256_bytes(baseline_bytes)
        print("PASS: artifacts are identical")
        print(f"sha256: {digest}")
        print(f"frames: {(len(baseline_bytes) - HEADER_SIZE) // FRAME_SIZE}")
        return 0

    baseline_sha = sha256_bytes(baseline_bytes)
    candidate_sha = sha256_bytes(candidate_bytes)
    print("INFO: artifacts differ at byte level")
    print(f"baseline_sha256:  {baseline_sha}")
    print(f"candidate_sha256: {candidate_sha}")

    b_header, b_frames, b_err = parse_artifact(args.baseline)
    if b_err:
        print(f"FAIL: baseline parse failed: {b_err}")
        return 3

    c_header, c_frames, c_err = parse_artifact(args.candidate)
    if c_err:
        print(f"FAIL: candidate parse failed: {c_err}")
        return 3

    header_fields = ["magic", "version", "header_len", "schema_len", "frame_count", "frame_size", "reserved"]
    header_diffs = [field for field in header_fields if b_header[field] != c_header[field]]
    if header_diffs:
        print("FAIL: header mismatch")
        for field in header_diffs:
            b_val = b_header[field]
            c_val = c_header[field]
            if field == "magic":
                b_val = format_magic(b_val)
                c_val = format_magic(c_val)
            print(f"  {field}: baseline={b_val} candidate={c_val}")
        return 4

    frame_fields = ["frame_idx", "irq_id", "flags", "timer_delta", "input_sample"]
    for idx, (b_frame, c_frame) in enumerate(zip(b_frames, c_frames)):
        diffs = [field for field in frame_fields if b_frame[field] != c_frame[field]]
        if diffs:
            print(f"FAIL: first divergence at frame {idx}")
            print()
            print("baseline:")
            print(f"  {format_frame_line(b_frame)}")
            print()
            print("candidate:")
            print(f"  {format_frame_line(c_frame)}")
            print()
            print(f"differing_fields: {', '.join(diffs)}")
            return 5

    # Should be unreachable because byte-equality was checked first.
    print("FAIL: artifacts differ by bytes but no semantic divergence found")
    return 6


if __name__ == "__main__":
    sys.exit(main())
