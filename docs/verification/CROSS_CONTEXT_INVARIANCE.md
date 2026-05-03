# Cross-Context Invariance

This document records an external validation check showing that identical RPL0
files produce identical replay outputs and divergence classification across
multiple execution contexts.

## Scope

- Portable replay format: existing RPL0 v1 container files with legacy
  `EventFrame0` replay semantics
- Here, `v1` refers to RPL0 format version 1 container framing, not a replay-semantics revision.
- External target: existing synthetic fixture generator
  `scripts/generate_demo_v5_fixtures.py`
- Canonical context: Python replay tooling
  `scripts/artifact_tool.py`, `scripts/artifact_diff.py`
- Comparison-only context: experimental Rust `replay-host`

No DSP math, RPL schema interpretation, or release-surface behavior changed.

## Test Setup

Two independent context directories were generated:

- [docs/verification/cross_context/context_a](cross_context/context_a)
- [docs/verification/cross_context/context_b](cross_context/context_b)

Each context contains deterministic baseline/perturbed pairs for four scenarios:

- `self_healing`
- `bounded_persistent`
- `monotonic_growth`
- `region_transition`

Commands used are retained in
[docs/verification/cross_context/commands.txt](cross_context/commands.txt).
Per-context replay outputs are retained under
[docs/verification/cross_context/outputs](cross_context/outputs).

Note: Fixtures use legacy EventFrame0 semantics (version: 0) within RPL0 container framing.

## Result

`PASS`

- Same RPL bytes were produced in both contexts.
- Same RPL bytes produced the same canonical hash output in both contexts.
- Same RPL bytes produced the same replay hash progression in both contexts.
- Python divergence classification output matched exactly across both contexts.
- Experimental Rust `replay-host` reported the same first-divergence frame for
  the same pairs in both contexts.
- No context-dependent replay or classification behavior was observed.

Note: Verification failures on perturbed RPL files are expected due to intentional divergence.
They do not indicate RPL corruption and are part of the test design.

## Bounded Note

`replay-host` remains experimental and comparison-only. This check does not
promote it to the release surface. The canonical classification authority
remains the Python tooling and the normative divergence semantics documented in
[docs/replay/DIVERGENCE_SEMANTICS.md](../replay/DIVERGENCE_SEMANTICS.md).

## Retained Evidence

- [docs/verification/cross_context/comparison_summary.md](cross_context/comparison_summary.md)
- [docs/verification/cross_context/replay_hash_progression_summary.txt](cross_context/replay_hash_progression_summary.txt)
