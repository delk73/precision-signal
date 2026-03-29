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

CORPUS_LENGTH = 12
LOW_VALUE = 1
HIGH_VALUE = 6
TARGET_QUANT_SHIFTS = (2, 3)
SEARCH_KIND = "exhaustive"


@dataclass(frozen=True)
class CorpusCase:
    corpus_id: str
    bitmask: int
    corpus: list[int]
    high_positions: list[int]
    stressor_tags: list[str]


@dataclass(frozen=True)
class SearchResult:
    corpus_id: str
    bitmask: int
    corpus: list[int]
    high_positions: list[int]
    stressor_tags: list[str]
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


def bitmask_to_corpus(bitmask: int) -> list[int]:
    corpus: list[int] = []
    for idx in range(CORPUS_LENGTH):
        corpus.append(HIGH_VALUE if bitmask & (1 << idx) else LOW_VALUE)
    return corpus


def stressor_tags_for(bitmask: int) -> list[str]:
    high_positions = [idx for idx in range(CORPUS_LENGTH) if bitmask & (1 << idx)]
    if not high_positions:
        return ["zero_residue_everywhere"]

    tags: list[str] = []
    first_high = high_positions[0]
    if first_high <= 2:
        tags.append("early_residue_placement")
    if first_high >= CORPUS_LENGTH - 3:
        tags.append("delayed_residue_placement")
    if first_high >= 8:
        tags.append("long_zero_residue_prefix_late_transition")

    alternating_a = sum(1 << idx for idx in range(0, CORPUS_LENGTH, 2))
    alternating_b = sum(1 << idx for idx in range(1, CORPUS_LENGTH, 2))
    if bitmask in (alternating_a, alternating_b):
        tags.append("alternating_residue_non_residue")

    run_length = 0
    best_run = 0
    for idx in range(CORPUS_LENGTH):
        if bitmask & (1 << idx):
            run_length += 1
            best_run = max(best_run, run_length)
        else:
            run_length = 0
    if best_run >= 2:
        tags.append("clustered_residue_bursts")

    if len(high_positions) == 1 and 1 <= first_high <= CORPUS_LENGTH - 2:
        tags.append("local_reorderings_around_predicted_boundary")

    return tags


def generate_cases() -> list[CorpusCase]:
    return [
        CorpusCase(
            corpus_id=f"B{bitmask:03X}",
            bitmask=bitmask,
            corpus=bitmask_to_corpus(bitmask),
            high_positions=[idx for idx in range(CORPUS_LENGTH) if bitmask & (1 << idx)],
            stressor_tags=stressor_tags_for(bitmask),
        )
        for bitmask in range(1 << CORPUS_LENGTH)
    ]


def evaluate_case(case: CorpusCase, quant_shift: int) -> SearchResult:
    baseline_outputs_1 = run_pipeline(case.corpus, "baseline", quant_shift)
    baseline_outputs_2 = run_pipeline(case.corpus, "baseline", quant_shift)
    quantized_outputs_1 = run_pipeline(case.corpus, "quantized", quant_shift)
    quantized_outputs_2 = run_pipeline(case.corpus, "quantized", quant_shift)

    predicted = predicted_first_divergence_frame(case.corpus, quant_shift)
    observed, classification, evolution_class = compare_outputs(
        baseline_outputs_1, quantized_outputs_1
    )
    return SearchResult(
        corpus_id=case.corpus_id,
        bitmask=case.bitmask,
        corpus=case.corpus,
        high_positions=case.high_positions,
        stressor_tags=case.stressor_tags,
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


def representative_cases(cases: list[CorpusCase]) -> dict[str, dict[str, object]]:
    representatives: dict[str, dict[str, object]] = {}
    required_tags = [
        "early_residue_placement",
        "delayed_residue_placement",
        "alternating_residue_non_residue",
        "clustered_residue_bursts",
        "long_zero_residue_prefix_late_transition",
        "local_reorderings_around_predicted_boundary",
    ]
    for tag in required_tags:
        match = next(case for case in cases if tag in case.stressor_tags)
        representatives[tag] = {
            "corpus_id": match.corpus_id,
            "bitmask": match.bitmask,
            "corpus": match.corpus,
            "high_positions": match.high_positions,
        }
    return representatives


def family_coverage(cases: list[CorpusCase]) -> dict[str, int]:
    coverage: dict[str, int] = {}
    for case in cases:
        for tag in case.stressor_tags:
            coverage[tag] = coverage.get(tag, 0) + 1
    return dict(sorted(coverage.items()))


def summary(results: list[SearchResult]) -> dict[str, object]:
    mismatches = [result for result in results if not result.match]
    by_quant_shift: dict[str, dict[str, int]] = {}
    for quant_shift in TARGET_QUANT_SHIFTS:
        q_results = [result for result in results if result.quant_shift == quant_shift]
        by_quant_shift[str(quant_shift)] = {
            "cases": len(q_results),
            "matches": sum(1 for result in q_results if result.match),
            "mismatches": sum(1 for result in q_results if not result.match),
            "baseline_repeat_failures": sum(
                1 for result in q_results if result.baseline_repeatability != "PASS"
            ),
            "quantized_repeat_failures": sum(
                1 for result in q_results if result.quantized_repeatability != "PASS"
            ),
            "no_divergence_matches": sum(
                1
                for result in q_results
                if result.match
                and result.predicted_first_divergence_frame is None
                and result.observed_first_divergence_frame is None
            ),
        }
    return {
        "decision": "FAIL" if mismatches else "PASS-constrained",
        "total_corpora": len({result.corpus_id for result in results}),
        "total_cases": len(results),
        "mismatches": len(mismatches),
        "by_quant_shift": by_quant_shift,
        "counterexamples": [asdict(result) for result in mismatches[:8]],
    }


def main() -> int:
    cases = generate_cases()
    results = [
        evaluate_case(case, quant_shift)
        for case in cases
        for quant_shift in TARGET_QUANT_SHIFTS
    ]
    output = {
        "search_space": {
            "kind": SEARCH_KIND,
            "corpus_length": CORPUS_LENGTH,
            "allowed_sample_values": [LOW_VALUE, HIGH_VALUE],
            "quant_shifts": list(TARGET_QUANT_SHIFTS),
            "generation_family": (
                "all 12-frame corpora over {1, 6}, enumerated by 12-bit mask where bit i=1 maps "
                "frame i to value 6 and bit i=0 maps frame i to value 1"
            ),
            "family_size": len(cases),
            "required_stressors": [
                "early_residue_placement",
                "delayed_residue_placement",
                "alternating_residue_non_residue",
                "clustered_residue_bursts",
                "long_zero_residue_prefix_late_transition",
                "local_reorderings_around_predicted_boundary",
            ],
            "stressor_coverage": family_coverage(cases),
            "stressor_representatives": representative_cases(cases),
        },
        "summary": summary(results),
        "results": [asdict(result) for result in results],
    }
    print(json.dumps(output, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
