# Cross-Context Comparison Summary

**Executed at:** `2026-03-26T01:21:22Z`
**Result:** `PASS`

## Setup

- External target: existing synthetic RPL0 v1 fixture generator `scripts/generate_demo_v5_fixtures.py`
- Here, `v1` refers to RPL0 format version 1 container framing, not a replay-semantics revision.
- Contexts:
  - `context_a`
  - `context_b`
- Canonical replay/classification context: Python tooling
  - `scripts/artifact_tool.py`
  - `scripts/artifact_diff.py`
- Comparison-only replay context: experimental Rust host
  - `cargo run -p replay-host -- diff ...`

The generator already emits RPL0 v1 container artifacts with legacy
`EventFrame0` replay semantics, so no schema or adapter changes were required.
All encoded fields remain integer-only and deterministic.

## Scenarios Compared

- `self_healing`
- `bounded_persistent`
- `monotonic_growth`
- `region_transition`

## Results

- Byte-identical artifacts matched across `context_a` and `context_b` for:
  - `self_healing_A.rpl`
  - `self_healing_B.rpl`
  - `bounded_persistent_A.rpl`
  - `bounded_persistent_B.rpl`
  - `monotonic_growth_A.rpl`
  - `monotonic_growth_B.rpl`
  - `region_transition_A.rpl`
  - `region_transition_B.rpl`
- Canonical artifact hashes matched across both contexts for the same artifact names.
- Replay hash progression matched across both contexts for the same artifact names.
- Python divergence output matched across both contexts for all four scenarios.
- Rust `replay-host` reported the same first-divergence frame (`4096`) across both contexts for all four scenarios.
- `artifact_tool verify` reports FAIL for perturbed artifacts by design,
  because verification enforces canonical baseline expectations.
- These FAIL results are expected and confirm divergence injection, not structural invalidity.

## Classification Checks

### `self_healing`

- Python output in both contexts:
  - `first_divergence_frame: 4096`
  - `shape_class: transient`
  - `primary_region: sample_payload`
  - `region_summary: sample_payload`
  - `evolution_class: self_healing`
  - `timeline_summary: divergence resolves within 1 frame`
- Rust output in both contexts:
  - `first divergence at frame 4096`

### `bounded_persistent`

- Python output in both contexts:
  - `first_divergence_frame: 4096`
  - `shape_class: persistent_offset`
  - `primary_region: sample_payload`
  - `region_summary: sample_payload`
  - `evolution_class: bounded_persistent`
  - `timeline_summary: divergence remains in sample_payload through final frame`
- Rust output in both contexts:
  - `first divergence at frame 4096`

### `monotonic_growth`

- Python output in both contexts:
  - `first_divergence_frame: 4096`
  - `shape_class: rate_divergence`
  - `primary_region: sample_payload`
  - `region_summary: sample_payload`
  - `evolution_class: monotonic_growth`
  - `timeline_summary: sample_payload magnitude grows from 1 to 5904 by frame 9999`
- Rust output in both contexts:
  - `first divergence at frame 4096`

### `region_transition`

- Python output in both contexts:
  - `first_divergence_frame: 4096`
  - `shape_class: none`
  - `primary_region: timer_delta`
  - `region_summary: timer_delta`
  - `evolution_class: region_transition`
  - `timeline_summary: divergence reaches sample_payload at frame 4098`
- Rust output in both contexts:
  - `first divergence at frame 4096`

## Bounded Differences

- None observed.
- `replay-host` was used for first-divergence confirmation only. It remains experimental and was not promoted or relied on for classification labels.
