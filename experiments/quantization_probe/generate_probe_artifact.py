#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import struct
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
INSPECT_ARTIFACT_PATH = REPO_ROOT / "scripts" / "inspect_artifact.py"
_INSPECT_ARTIFACT_SPEC = importlib.util.spec_from_file_location(
    "wip004_inspect_artifact", INSPECT_ARTIFACT_PATH
)
assert _INSPECT_ARTIFACT_SPEC is not None
assert _INSPECT_ARTIFACT_SPEC.loader is not None
inspect_artifact = importlib.util.module_from_spec(_INSPECT_ARTIFACT_SPEC)
_INSPECT_ARTIFACT_SPEC.loader.exec_module(inspect_artifact)


MAGIC = b"RPL0"
VERSION = 1
HEADER_LEN = 0x98
FRAME_FMT = "<IBBHIi"
FRAME_SIZE = struct.calcsize(FRAME_FMT)

V1_OFF_VERSION = 0x04
V1_OFF_HEADER_LEN = 0x06
V1_OFF_FRAME_COUNT = 0x08
V1_OFF_FRAME_SIZE = 0x0C
V1_OFF_FLAGS = 0x0E
V1_OFF_SCHEMA_LEN = 0x10
V1_OFF_SCHEMA_HASH = 0x14
V1_OFF_BUILD_HASH = 0x34
V1_OFF_CONFIG_HASH = 0x54
V1_OFF_BOARD_ID = 0x74
V1_OFF_CLOCK_PROFILE = 0x84
V1_OFF_CAPTURE_BOUNDARY = 0x94
V1_OFF_RESERVED = 0x96

EXPECTED_IRQ_ID = 0x02
EXPECTED_TIMER_DELTA = 1000
CORPUS_PATH = Path(__file__).with_name("corpus.txt")
CORPUS_PATHS = {
    "C1": CORPUS_PATH,
    "C2": Path(__file__).with_name("corpus_c2.txt"),
}
VALID_QUANT_SHIFTS = (2, 3, 4)
SCHEMA = (
    b'{"experiment":"quantization_probe","frame":"EventFrame0",'
    b'"pipeline":["affine_transform","accumulate","clamp","threshold"],'
    b'"output":"bounded_state_plus_decision"}'
)
BUILD_HASH = hashlib.sha256(b"quantization_probe_host_v1").digest()
CONFIG_HASH = hashlib.sha256(b"shared_corpus_fixed_v1").digest()
BOARD_ID = b"HOST-LINUX-BBB\0\0"
CLOCK_PROFILE = b"host-fixed-step\0"


# WIP-004 layout guard: import canonical parser constants directly without
# introducing new runtime/package architecture for the probe.
assert MAGIC == inspect_artifact.MAGIC
assert HEADER_LEN == inspect_artifact.V1_MIN_HEADER_SIZE
assert FRAME_FMT == inspect_artifact.FRAME_FMT
assert FRAME_SIZE == inspect_artifact.FRAME_SIZE
assert V1_OFF_VERSION == inspect_artifact.V1_OFF_VERSION
assert V1_OFF_HEADER_LEN == inspect_artifact.V1_OFF_HEADER_LEN
assert V1_OFF_FRAME_COUNT == inspect_artifact.V1_OFF_FRAME_COUNT
assert V1_OFF_FRAME_SIZE == inspect_artifact.V1_OFF_FRAME_SIZE
assert V1_OFF_FLAGS == inspect_artifact.V1_OFF_FLAGS
assert V1_OFF_SCHEMA_LEN == inspect_artifact.V1_OFF_SCHEMA_LEN
assert V1_OFF_SCHEMA_HASH == inspect_artifact.V1_OFF_SCHEMA_HASH
assert V1_OFF_BUILD_HASH == inspect_artifact.V1_OFF_BUILD_HASH
assert V1_OFF_CONFIG_HASH == inspect_artifact.V1_OFF_CONFIG_HASH
assert V1_OFF_BOARD_ID == inspect_artifact.V1_OFF_BOARD_ID
assert V1_OFF_CLOCK_PROFILE == inspect_artifact.V1_OFF_CLOCK_PROFILE
assert V1_OFF_CAPTURE_BOUNDARY == inspect_artifact.V1_OFF_CAPTURE_BOUNDARY
assert V1_OFF_RESERVED == inspect_artifact.V1_OFF_RESERVED


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Emit a minimal deterministic quantization witness artifact."
    )
    parser.add_argument(
        "--mode",
        required=True,
        choices=("baseline", "quantized"),
        help="Precision path to execute.",
    )
    parser.add_argument(
        "--corpus",
        choices=tuple(CORPUS_PATHS),
        default="C1",
        help="Fixed corpus selector (default: C1).",
    )
    parser.add_argument(
        "--quant-shift",
        type=int,
        choices=VALID_QUANT_SHIFTS,
        default=3,
        help="Quantization shift for quantized mode (default: 3).",
    )
    parser.add_argument("--out", type=Path, required=True, help="Artifact output path.")
    return parser.parse_args()


def load_corpus(path: Path) -> list[int]:
    values: list[int] = []
    for raw_line in path.read_text(encoding="ascii").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        values.append(int(line, 10))
    if not values:
        raise ValueError("corpus is empty")
    return values


def clamp(value: int, lo: int, hi: int) -> int:
    return max(lo, min(hi, value))


def quantize_step(value: int, shift: int) -> int:
    return (value >> shift) << shift


def run_pipeline(corpus: list[int], mode: str, quant_shift: int) -> list[int]:
    if mode == "baseline":
        quantized = False
    elif mode == "quantized":
        quantized = True
    else:
        raise ValueError(f"unsupported mode: {mode}")
    if quant_shift not in VALID_QUANT_SHIFTS:
        raise ValueError(f"unsupported quant_shift: {quant_shift}")

    acc = 0
    outputs: list[int] = []
    for sample in corpus:
        transformed = sample * 5 + 3
        if quantized:
            transformed = quantize_step(transformed, quant_shift)
        acc += transformed
        bounded = clamp(acc, -1024, 1024)
        decision = 1 if bounded >= 256 else 0
        outputs.append(bounded + decision)
    return outputs


def encode_header(frame_count: int) -> bytes:
    header = bytearray(HEADER_LEN)
    header[0:4] = MAGIC
    struct.pack_into("<H", header, V1_OFF_VERSION, VERSION)
    struct.pack_into("<H", header, V1_OFF_HEADER_LEN, HEADER_LEN)
    struct.pack_into("<I", header, V1_OFF_FRAME_COUNT, frame_count)
    struct.pack_into("<H", header, V1_OFF_FRAME_SIZE, FRAME_SIZE)
    struct.pack_into("<H", header, V1_OFF_FLAGS, 0)
    struct.pack_into("<I", header, V1_OFF_SCHEMA_LEN, len(SCHEMA))
    header[V1_OFF_SCHEMA_HASH : V1_OFF_SCHEMA_HASH + 32] = hashlib.sha256(SCHEMA).digest()
    header[V1_OFF_BUILD_HASH : V1_OFF_BUILD_HASH + 32] = BUILD_HASH
    header[V1_OFF_CONFIG_HASH : V1_OFF_CONFIG_HASH + 32] = CONFIG_HASH
    header[V1_OFF_BOARD_ID : V1_OFF_BOARD_ID + 16] = BOARD_ID
    header[V1_OFF_CLOCK_PROFILE : V1_OFF_CLOCK_PROFILE + 16] = CLOCK_PROFILE
    struct.pack_into("<H", header, V1_OFF_CAPTURE_BOUNDARY, 0)
    struct.pack_into("<H", header, V1_OFF_RESERVED, 0)
    return bytes(header)


def encode_frames(outputs: list[int]) -> bytes:
    frames = bytearray()
    for frame_idx, sample in enumerate(outputs):
        frames.extend(
            struct.pack(
                FRAME_FMT,
                frame_idx,
                EXPECTED_IRQ_ID,
                0,
                0,
                EXPECTED_TIMER_DELTA,
                sample,
            )
        )
    return bytes(frames)


def main() -> int:
    args = parse_args()
    corpus = load_corpus(CORPUS_PATHS[args.corpus])
    outputs = run_pipeline(corpus, args.mode, args.quant_shift)
    artifact = encode_header(len(outputs)) + SCHEMA + encode_frames(outputs)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_bytes(artifact)
    print(f"PASS: wrote {args.mode} artifact to {args.out}")
    print(f"corpus: {args.corpus}")
    print(f"quant_shift: {args.quant_shift}")
    print(f"frame_count: {len(outputs)}")
    print(f"first_output: {outputs[0]}")
    print(f"last_output: {outputs[-1]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
