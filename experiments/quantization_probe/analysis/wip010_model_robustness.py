#!/usr/bin/env python3
from __future__ import annotations

import json
from dataclasses import asdict, dataclass

from experiments.quantization_probe.generate_probe_artifact import (
    SCHEMA,
    encode_frames,
    encode_header,
    run_pipeline,
)
from scripts import artifact_diff, inspect_artifact


CORPORA: dict[str, list[int]] = {
    "P1_front_loaded_six": [6, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    "M1_monotonic_plateau_then_rise": [1, 1, 1, 1, 5, 5, 6, 6, 6, 6, 6, 6],
    "O1_alternating_low_high": [1, 6, 1, 6, 1, 6, 1, 6, 1, 6, 1, 6],
    "K1_plateau_then_jump": [1, 1, 1, 1, 1, 1, 6, 6, 6, 6, 6, 6],
}
TARGET_QUANT_SHIFTS = (2, 3)


@dataclass
class SliceResult:
    corpus_name: str
    corpus: list[int]
    quant_shift: int
    predicted_first_divergence_frame: int | None
    observed_first_divergence_frame: int | None
    match: bool
    baseline_repeatability: str
    quantized_repeatability: str
    classification: str | None
    evolution_class: str | None
    likely_failure_mode: str | None


def artifact_bytes(outputs: list[int]) -> bytes:
    return encode_header(len(outputs)) + SCHEMA + encode_frames(outputs)


def predicted_first_divergence_frame(corpus: list[int], quant_shift: int) -> int | None:
    # This residue-index rule is validated here only for the tested probe pipeline
    # and corpus family; this helper does not claim it as a universal law.
    mask = (1 << quant_shift) - 1
    for idx, sample in enumerate(corpus):
        if ((5 * sample + 3) & mask) != 0:
            return idx
    return None


def compare_outputs(
    baseline_outputs: list[int],
    quantized_outputs: list[int],
) -> tuple[int | None, str | None, str | None]:
    baseline_artifact = inspect_artifact.parse_artifact_bytes(
        artifact_bytes(baseline_outputs), allow_trailing=False
    )
    quantized_artifact = inspect_artifact.parse_artifact_bytes(
        artifact_bytes(quantized_outputs), allow_trailing=False
    )

    frames_a = baseline_artifact["frames"]
    frames_b = quantized_artifact["frames"]
    for idx, (frame_a, frame_b) in enumerate(zip(frames_a, frames_b)):
        _, regions, unsupported = artifact_diff.frame_supported_diffs(frame_a, frame_b)
        if unsupported:
            raise ValueError(
                f"unsupported field differences at frame {idx}: {', '.join(unsupported)}"
            )
        if not regions:
            continue
        sample_diffs = artifact_diff.sample_diff_series(frames_a, frames_b, idx)
        classification = None
        if artifact_diff.SAMPLE_PAYLOAD_REGION in regions:
            classification = artifact_diff.classify_sample_diffs(sample_diffs)
        timeline_regions: list[list[str]] = []
        for timeline_a, timeline_b in zip(frames_a[idx:], frames_b[idx:]):
            _, timeline_frame_regions, timeline_unsupported = artifact_diff.frame_supported_diffs(
                timeline_a, timeline_b
            )
            if timeline_unsupported:
                raise ValueError(
                    "unsupported evolution field differences: "
                    + ", ".join(timeline_unsupported)
                )
            timeline_regions.append(
                sorted(timeline_frame_regions, key=artifact_diff.PRIMARY_REGION_PRECEDENCE.index)
            )
        evolution_class, _ = artifact_diff.classify_evolution(
            first_divergence_frame=idx,
            first_regions=sorted(regions, key=artifact_diff.PRIMARY_REGION_PRECEDENCE.index),
            timeline_regions=timeline_regions,
            sample_diffs=sample_diffs,
        )
        return idx, classification, evolution_class
    return None, None, None


def evaluate_corpus(name: str, corpus: list[int], quant_shift: int) -> SliceResult:
    baseline_outputs_1 = run_pipeline(corpus, "baseline", quant_shift)
    baseline_outputs_2 = run_pipeline(corpus, "baseline", quant_shift)
    quantized_outputs_1 = run_pipeline(corpus, "quantized", quant_shift)
    quantized_outputs_2 = run_pipeline(corpus, "quantized", quant_shift)

    predicted = predicted_first_divergence_frame(corpus, quant_shift)
    observed, classification, evolution_class = compare_outputs(
        baseline_outputs_1, quantized_outputs_1
    )
    return SliceResult(
        corpus_name=name,
        corpus=corpus,
        quant_shift=quant_shift,
        predicted_first_divergence_frame=predicted,
        observed_first_divergence_frame=observed,
        match=predicted == observed,
        baseline_repeatability=(
            "PASS"
            if artifact_bytes(baseline_outputs_1) == artifact_bytes(baseline_outputs_2)
            else "FAIL"
        ),
        quantized_repeatability=(
            "PASS"
            if artifact_bytes(quantized_outputs_1) == artifact_bytes(quantized_outputs_2)
            else "FAIL"
        ),
        classification=classification,
        evolution_class=evolution_class,
        likely_failure_mode=None if predicted == observed else "invalid assumption in rule",
    )


def main() -> int:
    rows = [
        asdict(evaluate_corpus(name, corpus, quant_shift))
        for name, corpus in CORPORA.items()
        for quant_shift in TARGET_QUANT_SHIFTS
    ]
    print(json.dumps(rows, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
