#!/usr/bin/env python3
import argparse
import hashlib
import re
import sys
from pathlib import Path

import compare_artifact
import inspect_artifact
import lock_baseline
import read_artifact

SHA256_HEX_RE = re.compile(r"^[0-9a-f]{64}$")


def run_with_argv(main_fn, argv: list[str]) -> int:
    old_argv = sys.argv
    try:
        sys.argv = argv
        return int(main_fn())
    finally:
        sys.argv = old_argv


def cmd_capture(args: argparse.Namespace) -> int:
    forwarded = ["read_artifact.py"]
    if args.out is not None:
        forwarded.extend(["--out", str(args.out)])
    if args.quick:
        forwarded.append("--quick")
    if args.no_verify:
        forwarded.append("--no-verify")
    if args.signal_model is not None:
        forwarded.extend(["--signal-model", args.signal_model])
    if args.max_sync_bytes is not None:
        forwarded.extend(["--max-sync-bytes", str(args.max_sync_bytes)])
    return run_with_argv(read_artifact.main, forwarded)


def cmd_verify(args: argparse.Namespace) -> int:
    try:
        artifact = inspect_artifact.parse_artifact(args.artifact, allow_trailing=False)
    except ValueError as exc:
        print(f"FAIL: invalid artifact structure ({exc})")
        return 1

    if artifact["frame_count"] != read_artifact.EXPECTED_FRAME_COUNT:
        print(
            "FAIL: unexpected frame_count "
            f"{artifact['frame_count']} (expected {read_artifact.EXPECTED_FRAME_COUNT})"
        )
        return 1

    signal_model = args.signal_model
    for expected_idx, frame in enumerate(artifact["frames"]):
        if frame["frame_idx"] != expected_idx:
            print(
                "FAIL: frame_idx mismatch at frame "
                f"{expected_idx}: got {frame['frame_idx']}"
            )
            return 1
        if frame["irq_id"] != read_artifact.EXPECTED_IRQ_ID:
            print(
                "FAIL: unexpected irq_id at frame "
                f"{expected_idx}: got {frame['irq_id']} "
                f"(expected {read_artifact.EXPECTED_IRQ_ID})"
            )
            return 1
        if frame["timer_delta"] != read_artifact.EXPECTED_TIMER_DELTA:
            print(
                "FAIL: unexpected timer_delta at frame "
                f"{expected_idx}: got {frame['timer_delta']} "
                f"(expected {read_artifact.EXPECTED_TIMER_DELTA})"
            )
            return 1

        expected_sample = read_artifact.expected_sample_for_model(expected_idx, signal_model)
        if expected_sample is not None:
            got_sample = frame["input_sample"] & 0xFFFFFFFF
            if got_sample != expected_sample:
                print(
                    "FAIL: sample mismatch at frame "
                    f"{expected_idx} expected 0x{expected_sample:08X} "
                    f"got 0x{got_sample:08X}"
                )
                return 1

    print("PASS: artifact structure is valid")
    print(f"magic: {artifact['magic'].decode('ascii', errors='replace')}")
    print(f"version: {artifact['version']}")
    print(f"header_len: {artifact['header_len']}")
    print(f"schema_len: {artifact['schema_len']}")
    print(f"frame_count: {artifact['frame_count']}")
    print(f"frame_size: {artifact['frame_size']}")
    print(f"canonical_len: {artifact['canonical_len']}")
    if artifact["version"] == 1:
        print(f"capture_boundary: {artifact['capture_boundary']}")
    print(f"signal_model: {signal_model}")
    return 0


def cmd_hash(args: argparse.Namespace) -> int:
    if args.expect is not None and not SHA256_HEX_RE.fullmatch(args.expect):
        print("FAIL: --expect must be exactly 64 lowercase hex characters")
        return 1

    try:
        artifact = inspect_artifact.parse_artifact(args.artifact, allow_trailing=True)
    except ValueError as exc:
        print(f"FAIL: invalid artifact structure ({exc})")
        return 1

    try:
        data = args.artifact.read_bytes()
    except OSError as exc:
        print(f"FAIL: cannot read artifact ({exc})")
        return 1

    digest = hashlib.sha256(data[: artifact["canonical_len"]]).hexdigest()
    print(f"sha256: {digest}")
    print(f"canonical_len: {artifact['canonical_len']}")
    print(f"trailing_len: {artifact['trailing_len']}")
    print(f"version: {artifact['version']}")
    print(f"frame_count: {artifact['frame_count']}")
    print(f"frame_size: {artifact['frame_size']}")

    if args.expect is not None and digest != args.expect:
        print(f"FAIL: hash mismatch ({digest} != {args.expect})")
        return 1
    if args.expect is not None:
        print("PASS: hash matches expected")
    return 0


def cmd_compare(args: argparse.Namespace) -> int:
    forwarded = ["compare_artifact.py", str(args.baseline), str(args.candidate)]
    return run_with_argv(compare_artifact.main, forwarded)


def cmd_inspect(args: argparse.Namespace) -> int:
    forwarded = ["inspect_artifact.py", str(args.artifact)]
    if args.frames is not None:
        forwarded.extend(["--frames", args.frames])
    if args.frame is not None:
        forwarded.extend(["--frame", str(args.frame)])
    if args.summary:
        forwarded.append("--summary")
    if args.raw:
        forwarded.append("--raw")
    return run_with_argv(inspect_artifact.main, forwarded)


def cmd_lock_baseline(args: argparse.Namespace) -> int:
    forwarded = ["lock_baseline.py"]
    if args.source is not None:
        forwarded.extend(["--source", str(args.source)])
    if args.artifacts_dir is not None:
        forwarded.extend(["--artifacts-dir", str(args.artifacts_dir)])
    if args.serial_port is not None:
        forwarded.extend(["--serial-port", args.serial_port])
    if args.fw_features is not None:
        forwarded.extend(["--fw-features", args.fw_features])
    forwarded.extend(["--signal-model", args.signal_model])
    forwarded.extend(["--manifest-name", args.manifest_name])
    if args.notes is not None:
        forwarded.extend(["--notes", args.notes])
    return run_with_argv(lock_baseline.main, forwarded)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="artifact_tool.py",
        usage="artifact_tool.py <command> ...",
        description="Unified CLI for replay artifact workflows.",
    )
    sub = parser.add_subparsers(dest="command")

    p_capture = sub.add_parser("capture", help="capture artifact from UART")
    p_capture.add_argument("--out", type=Path)
    p_capture.add_argument("--quick", action="store_true")
    p_capture.add_argument("--no-verify", action="store_true")
    p_capture.add_argument(
        "--signal-model",
        choices=read_artifact.SIGNAL_MODELS,
        default="none",
        help="Optional input_sample validator: none (default), ramp, phase8.",
    )
    p_capture.add_argument("--max-sync-bytes", type=int)
    p_capture.set_defaults(func=cmd_capture)

    p_verify = sub.add_parser("verify", help="validate artifact structure")
    p_verify.add_argument("artifact", type=Path)
    p_verify.add_argument(
        "--signal-model",
        choices=read_artifact.SIGNAL_MODELS,
        default="none",
        help="Optional input_sample validator: none (default), ramp, phase8.",
    )
    p_verify.set_defaults(func=cmd_verify)

    p_compare = sub.add_parser("compare", help="compare two artifacts")
    p_compare.add_argument("baseline", type=Path)
    p_compare.add_argument("candidate", type=Path)
    p_compare.set_defaults(func=cmd_compare)

    p_hash = sub.add_parser("hash", help="compute canonical artifact SHA256")
    p_hash.add_argument("artifact", type=Path)
    p_hash.add_argument("--expect", type=str, help="Expected lowercase hex SHA256.")
    p_hash.set_defaults(func=cmd_hash)

    p_inspect = sub.add_parser("inspect", help="human-readable artifact inspection")
    p_inspect.add_argument("artifact", type=Path)
    p_inspect.add_argument("--frames", type=str)
    p_inspect.add_argument("--frame", type=int)
    p_inspect.add_argument("--summary", action="store_true")
    p_inspect.add_argument("--raw", action="store_true")
    p_inspect.set_defaults(func=cmd_inspect)

    p_lock = sub.add_parser("lock-baseline", help="lock baseline artifact")
    p_lock.add_argument("--source", type=Path)
    p_lock.add_argument("--artifacts-dir", type=Path)
    p_lock.add_argument("--serial-port", type=str)
    p_lock.add_argument("--fw-features", type=str)
    p_lock.add_argument(
        "--signal-model",
        required=True,
        help="Signal model for the artifact being promoted (e.g. phase8, ramp). Required.",
    )
    p_lock.add_argument(
        "--manifest-name",
        default="manifest.txt",
        help="Manifest filename inside artifacts-dir (default: manifest.txt).",
    )
    p_lock.add_argument("--notes", type=str)
    p_lock.set_defaults(func=cmd_lock_baseline)

    return parser


def main() -> int:
    parser = build_parser()
    if len(sys.argv) == 1:
        parser.print_help()
        print()
        print("commands:")
        print("  capture         capture artifact from UART")
        print("  verify          validate artifact structure")
        print("  hash            compute canonical artifact SHA256")
        print("  compare         compare two artifacts")
        print("  inspect         human-readable artifact inspection")
        print("  lock-baseline   lock baseline artifact")
        return 1

    args = parser.parse_args()
    func = getattr(args, "func", None)
    if func is None:
        parser.print_help()
        return 1
    return int(func(args))


if __name__ == "__main__":
    raise SystemExit(main())
