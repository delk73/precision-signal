#!/usr/bin/env python3
import argparse
import sys
from pathlib import Path
from typing import Optional

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import inspect_artifact

TRANSIENT_WINDOW_FRAMES = 8
HEADER_SCHEMA_REGION = "header_schema"
TIMER_DELTA_REGION = "timer_delta"
IRQ_STATE_REGION = "irq_state"
SAMPLE_PAYLOAD_REGION = "sample_payload"
PRIMARY_REGION_PRECEDENCE = (
    HEADER_SCHEMA_REGION,
    TIMER_DELTA_REGION,
    IRQ_STATE_REGION,
    SAMPLE_PAYLOAD_REGION,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Compare two replay artifacts and report exact first divergence frame."
    )
    parser.add_argument("artifact_a", type=Path, help="Artifact A path.")
    parser.add_argument("artifact_b", type=Path, help="Artifact B path.")
    return parser.parse_args()


def fail(message: str) -> int:
    print(f"FAIL: {message}")
    return 1


def parse_artifact(path: Path) -> tuple[dict | None, bytes | None, str | None]:
    try:
        data = path.read_bytes()
        return inspect_artifact.parse_artifact_bytes(data, allow_trailing=False), data, None
    except ValueError as exc:
        return None, None, str(exc)
    except OSError as exc:
        return None, None, f"read error: {exc}"


def u32_hex(value: int) -> str:
    return f"0x{(value & 0xFFFFFFFF):08X}"


def non_frame_prefix_bytes(parsed: dict, data: bytes) -> bytes:
    return data[: parsed["frames_offset"]]


def sample_diff_series(
    frames_a: list[dict], frames_b: list[dict], start_idx: int
) -> list[int]:
    sample_diffs: list[int] = []
    for idx in range(start_idx, len(frames_a)):
        sample_a = frames_a[idx]["input_sample"]
        sample_b = frames_b[idx]["input_sample"]
        sample_diffs.append(sample_a - sample_b)
    return sample_diffs


def classify_sample_diffs(sample_diffs: list[int]) -> Optional[str]:
    # Transient requires reconvergence within K frames and sustained equality
    # through the remainder of the transient window.
    transient_window_end = min(len(sample_diffs) - 1, TRANSIENT_WINDOW_FRAMES)
    for rel_idx in range(1, transient_window_end + 1):
        if sample_diffs[rel_idx] != 0:
            continue
        if all(sample_diffs[j] == 0 for j in range(rel_idx, transient_window_end + 1)):
            return "transient"

    if sample_diffs and all(diff == sample_diffs[0] for diff in sample_diffs):
        return "persistent_offset"

    abs_diffs = [abs(diff) for diff in sample_diffs]
    nondecreasing = all(
        abs_diffs[idx + 1] >= abs_diffs[idx] for idx in range(len(abs_diffs) - 1)
    )
    strict_increase = any(
        abs_diffs[idx + 1] > abs_diffs[idx] for idx in range(len(abs_diffs) - 1)
    )
    if nondecreasing and strict_increase:
        return "rate_divergence"
    return None


def reconvergence_summary(sample_diffs: list[int], start_idx: int) -> str | None:
    transient_window_end = min(len(sample_diffs) - 1, TRANSIENT_WINDOW_FRAMES)
    for rel_idx in range(1, transient_window_end + 1):
        if sample_diffs[rel_idx] != 0:
            continue
        if all(sample_diffs[j] == 0 for j in range(rel_idx, transient_window_end + 1)):
            return f"reconverged_at_frame={start_idx + rel_idx}"
    return None


def primary_region(regions: list[str]) -> str:
    for region in PRIMARY_REGION_PRECEDENCE:
        if region in regions:
            return region
    raise ValueError(f"no supported primary region in {regions}")


def format_region_list(regions: list[str]) -> str:
    return "[" + ", ".join(regions) + "]"


def format_regions_for_summary(regions: list[str]) -> str:
    if not regions:
        return "none"
    if len(regions) == 1:
        return regions[0]
    return "+".join(regions)


def frame_supported_diffs(frame_a: dict, frame_b: dict) -> tuple[list[str], list[str], list[str]]:
    diffs = []
    regions: list[str] = []
    if frame_a["input_sample"] != frame_b["input_sample"]:
        diffs.append("sample")
        regions.append(SAMPLE_PAYLOAD_REGION)
    if frame_a["irq_id"] != frame_b["irq_id"]:
        diffs.append("irq")
        regions.append(IRQ_STATE_REGION)
    if frame_a["timer_delta"] != frame_b["timer_delta"]:
        diffs.append("delta")
        regions.append(TIMER_DELTA_REGION)

    unsupported_diffs = []
    if frame_a["frame_idx"] != frame_b["frame_idx"]:
        unsupported_diffs.append("frame_idx")
    if frame_a["flags"] != frame_b["flags"]:
        unsupported_diffs.append("flags")
    if frame_a["rsv"] != frame_b["rsv"]:
        unsupported_diffs.append("reserved")
    if not regions and frame_a["raw"] != frame_b["raw"]:
        unsupported_diffs.append("raw")
    elif regions and frame_a["raw"] != frame_b["raw"]:
        diffs.append("raw")
    return diffs, regions, unsupported_diffs


def classify_evolution(
    *,
    first_divergence_frame: int | None,
    first_regions: list[str],
    timeline_regions: list[list[str]],
    sample_diffs: list[int],
) -> tuple[str | None, str | None]:
    if first_divergence_frame is None:
        return None, None

    first_region_set = set(first_regions)
    for rel_idx, regions in enumerate(timeline_regions[1:], start=1):
        extra_regions = sorted(
            set(regions) - first_region_set, key=PRIMARY_REGION_PRECEDENCE.index
        )
        if extra_regions:
            transition_frame = first_divergence_frame + rel_idx
            return (
                "region_transition",
                "divergence reaches "
                f"{format_regions_for_summary(extra_regions)} at frame {transition_frame}",
            )

    resolution_rel_idx: int | None = None
    for rel_idx, regions in enumerate(timeline_regions[1:], start=1):
        if regions:
            continue
        if all(not later_regions for later_regions in timeline_regions[rel_idx:]):
            resolution_rel_idx = rel_idx
            break
    if resolution_rel_idx is not None:
        return (
            "self_healing",
            f"divergence resolves within {resolution_rel_idx} frame"
            f"{'' if resolution_rel_idx == 1 else 's'}",
        )

    if (
        SAMPLE_PAYLOAD_REGION in first_region_set
        and sample_diffs
        and len(sample_diffs) > 1
    ):
        abs_diffs = [abs(diff) for diff in sample_diffs]
        nondecreasing = all(
            abs_diffs[idx + 1] >= abs_diffs[idx] for idx in range(len(abs_diffs) - 1)
        )
        strict_increase = any(
            abs_diffs[idx + 1] > abs_diffs[idx] for idx in range(len(abs_diffs) - 1)
        )
        if nondecreasing and strict_increase:
            return (
                "monotonic_growth",
                "sample_payload magnitude grows from "
                f"{abs_diffs[0]} to {abs_diffs[-1]} by frame "
                f"{first_divergence_frame + len(abs_diffs) - 1}",
            )

    final_regions = timeline_regions[-1]
    return (
        "bounded_persistent",
        "divergence remains in "
        f"{format_regions_for_summary(final_regions)} through final frame",
    )


def print_divergence_summary(
    *,
    first_divergence_frame: int | None,
    shape_class: str | None,
    regions: list[str],
    reconvergence: str | None,
    evolution_class: str | None,
    timeline_summary: str | None,
) -> None:
    primary = "none" if not regions else primary_region(regions)
    region_summary = "none"
    if len(regions) == 1:
        region_summary = regions[0]
    elif len(regions) > 1:
        region_summary = "mixed"

    frame_value = "none" if first_divergence_frame is None else str(first_divergence_frame)
    shape_value = "none" if shape_class is None else shape_class
    print(f"first_divergence_frame: {frame_value}")
    print(f"shape_class: {shape_value}")
    print(f"primary_region: {primary}")
    print(f"all_regions_at_first_divergence: {format_region_list(regions)}")
    print(f"region_summary: {region_summary}")
    print(f"evolution_class: {evolution_class or 'none'}")
    print(f"timeline_summary: {timeline_summary or 'none'}")
    if reconvergence is not None:
        print(f"reconvergence_summary: {reconvergence}")


def main() -> int:
    args = parse_args()

    a, data_a, err_a = parse_artifact(args.artifact_a)
    if err_a is not None:
        return fail(f"artifact A parse error ({err_a})")
    b, data_b, err_b = parse_artifact(args.artifact_b)
    if err_b is not None:
        return fail(f"artifact B parse error ({err_b})")

    assert a is not None and b is not None
    assert data_a is not None and data_b is not None

    header_schema_diff = (
        a["version"] != b["version"]
        or a["frame_size"] != b["frame_size"]
        or a["frame_count"] != b["frame_count"]
        or non_frame_prefix_bytes(a, data_a) != non_frame_prefix_bytes(b, data_b)
    )
    frame_count = min(a["frame_count"], b["frame_count"])
    if header_schema_diff:
        diffs = ["header/schema"]
        regions = [HEADER_SCHEMA_REGION]
        classification = None
        timeline_regions = [[HEADER_SCHEMA_REGION] for _ in range(frame_count)]
        sample_diffs: list[int] = []
        if a["frames"] and b["frames"]:
            frame_a = a["frames"][0]
            frame_b = b["frames"][0]
            frame_diffs, frame_regions, unsupported_diffs = frame_supported_diffs(frame_a, frame_b)
            if unsupported_diffs:
                return fail(
                    "first divergent frame contains unsupported field differences "
                    f"({', '.join(unsupported_diffs)})"
                )
            diffs.extend(field for field in frame_diffs if field != "raw")
            regions.extend(frame_regions)
            if frame_regions and "raw" in frame_diffs:
                diffs.append("raw")
            if SAMPLE_PAYLOAD_REGION in frame_regions:
                sample_diffs = sample_diff_series(a["frames"], b["frames"], 0)
                classification = classify_sample_diffs(sample_diffs)
                if classification is None:
                    return fail(
                        "divergence shape is non-transient and non-constant-offset, "
                        "but does not satisfy rate-divergence monotonic-growth rule"
                    )
        for idx, (frame_a, frame_b) in enumerate(zip(a["frames"], b["frames"])):
            _, frame_regions, unsupported_diffs = frame_supported_diffs(frame_a, frame_b)
            if unsupported_diffs:
                return fail(
                    "evolution classification encountered unsupported field differences "
                    f"at frame {idx} ({', '.join(unsupported_diffs)})"
                )
            timeline_regions[idx] = sorted(
                set(timeline_regions[idx]) | set(frame_regions),
                key=PRIMARY_REGION_PRECEDENCE.index,
            )
        evolution_class, timeline_summary = classify_evolution(
            first_divergence_frame=0,
            first_regions=sorted(set(regions), key=PRIMARY_REGION_PRECEDENCE.index),
            timeline_regions=timeline_regions,
            sample_diffs=sample_diffs,
        )
        print("DIVERGENCE DETECTED")
        print("First divergence frame: 0")
        print(f"Classification: {classification or 'none'}")
        if SAMPLE_PAYLOAD_REGION in regions:
            print(f"Sample A: {u32_hex(a['frames'][0]['input_sample'])}")
            print(f"Sample B: {u32_hex(b['frames'][0]['input_sample'])}")
        if IRQ_STATE_REGION in regions:
            print(f"IRQ A/B: {a['frames'][0]['irq_id']} / {b['frames'][0]['irq_id']}")
        if TIMER_DELTA_REGION in regions:
            print(f"Delta A/B: {a['frames'][0]['timer_delta']} / {b['frames'][0]['timer_delta']}")
        if "raw" in diffs:
            print(f"Raw A: {a['frames'][0]['raw'].hex()}")
            print(f"Raw B: {b['frames'][0]['raw'].hex()}")
        print(f"Differing fields: {', '.join(diffs)}")
        print(f"Total frames: {min(a['frame_count'], b['frame_count'])}")
        print_divergence_summary(
            first_divergence_frame=0,
            shape_class=classification,
            regions=sorted(set(regions), key=PRIMARY_REGION_PRECEDENCE.index),
            reconvergence=(
                reconvergence_summary(sample_diffs, 0) if classification == "transient" else None
            )
            if SAMPLE_PAYLOAD_REGION in regions
            else None,
            evolution_class=evolution_class,
            timeline_summary=timeline_summary,
        )
        return 0

    for idx, (frame_a, frame_b) in enumerate(zip(a["frames"], b["frames"])):
        diffs, regions, unsupported_diffs = frame_supported_diffs(frame_a, frame_b)

        if unsupported_diffs:
            return fail(
                "first divergent frame contains unsupported field differences "
                f"({', '.join(unsupported_diffs)})"
            )

        if diffs:
            classification = None
            sample_diffs = sample_diff_series(a["frames"], b["frames"], idx)
            timeline_regions: list[list[str]] = []
            for timeline_idx, (timeline_a, timeline_b) in enumerate(
                zip(a["frames"][idx:], b["frames"][idx:])
            ):
                _, timeline_frame_regions, timeline_unsupported_diffs = frame_supported_diffs(
                    timeline_a, timeline_b
                )
                if timeline_unsupported_diffs:
                    return fail(
                        "evolution classification encountered unsupported field differences "
                        f"at frame {idx + timeline_idx} "
                        f"({', '.join(timeline_unsupported_diffs)})"
                    )
                timeline_regions.append(
                    sorted(timeline_frame_regions, key=PRIMARY_REGION_PRECEDENCE.index)
                )
            if SAMPLE_PAYLOAD_REGION in regions:
                classification = classify_sample_diffs(sample_diffs)
            if SAMPLE_PAYLOAD_REGION in regions and classification is None:
                return fail(
                    "divergence shape is non-transient and non-constant-offset, "
                    "but does not satisfy rate-divergence monotonic-growth rule"
                )
            evolution_class, timeline_summary = classify_evolution(
                first_divergence_frame=idx,
                first_regions=sorted(regions, key=PRIMARY_REGION_PRECEDENCE.index),
                timeline_regions=timeline_regions,
                sample_diffs=sample_diffs,
            )
            print("DIVERGENCE DETECTED")
            print(f"First divergence frame: {idx}")
            print(f"Classification: {classification or 'none'}")
            if SAMPLE_PAYLOAD_REGION in regions:
                print(f"Sample A: {u32_hex(frame_a['input_sample'])}")
                print(f"Sample B: {u32_hex(frame_b['input_sample'])}")
            if "irq" in diffs:
                print(f"IRQ A/B: {frame_a['irq_id']} / {frame_b['irq_id']}")
            if "delta" in diffs:
                print(f"Delta A/B: {frame_a['timer_delta']} / {frame_b['timer_delta']}")
            if "flags" in diffs:
                print(f"Flags A/B: {frame_a['flags']} / {frame_b['flags']}")
            if "raw" in diffs:
                print(f"Raw A: {frame_a['raw'].hex()}")
                print(f"Raw B: {frame_b['raw'].hex()}")
            print(f"Differing fields: {', '.join(diffs)}")
            print(f"Total frames: {a['frame_count']}")
            print_divergence_summary(
                first_divergence_frame=idx,
                shape_class=classification,
                regions=sorted(regions, key=PRIMARY_REGION_PRECEDENCE.index),
                reconvergence=(
                    reconvergence_summary(sample_diffs, idx)
                    if classification == "transient"
                    else None
                ),
                evolution_class=evolution_class,
                timeline_summary=timeline_summary,
            )
            return 0

    print("NO DIVERGENCE DETECTED")
    print("NO DIVERGENCE: artifacts identical")
    print(f"Total frames: {a['frame_count']}")
    print_divergence_summary(
        first_divergence_frame=None,
        shape_class=None,
        regions=[],
        reconvergence=None,
        evolution_class=None,
        timeline_summary=None,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
