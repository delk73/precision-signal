## Finding 1

File:
docs/README.md
Section / Line:
Intro, lines 9-11
Category:
[AUDIT:CONSISTENCY]
Current Text:
`The current public v1 release boundary covers the deterministic capture, artifact contract, replay, and divergence-analysis surfaces...`
Issue:
This uses `release boundary` where the repository's canonical descriptive term is `release surface`.
Proposed Fix:
`The current public v1 release surface covers the deterministic capture, artifact contract, replay, and divergence-analysis surfaces...`
Rationale:
This is a direct terminology normalization with no scope change.
Evidence:
`docs/RELEASE_SURFACE.md`:1-5, 44-50

## Finding 2

File:
docs/README.md
Section / Line:
Claim Classes / Start Here, lines 31-37
Category:
[AUDIT:TERM]
Current Text:
`Release-boundary wording ...` and `current release boundary`
Issue:
The document mixes `release-boundary` and `release boundary` with the canonical term `release surface`.
Proposed Fix:
Use `release-surface wording` and `current release surface`.
Rationale:
The meaning stays the same while matching the repository term of art.
Evidence:
`docs/RELEASE_SURFACE.md`:1-5

## Finding 3

File:
docs/replay/README.md
Section / Line:
Intro, lines 6-13
Category:
[AUDIT:CONSISTENCY]
Current Text:
`The public v1 release boundary is narrower...`
Issue:
This document uses `release boundary` for the v1 stability contract instead of the canonical `release surface`.
Proposed Fix:
`The public v1 release surface is narrower...`
Rationale:
This is a local terminology correction only.
Evidence:
`docs/RELEASE_SURFACE.md`:44-50

## Finding 4

File:
docs/architecture/replay_explained.md
Section / Line:
Intro, lines 8-10
Category:
[AUDIT:CONSISTENCY]
Current Text:
`part of the public v1 execution-analysis release boundary`
Issue:
The phrase uses `release boundary` instead of the canonical `release surface`.
Proposed Fix:
`part of the public v1 execution-analysis release surface`
Rationale:
This matches the canonical repository terminology without changing scope.
Evidence:
`docs/RELEASE_SURFACE.md`:1-5, 44-50

## Finding 5

File:
docs/architecture/architecture_whitepaper.md
Section / Line:
Abstract, line 22
Category:
[AUDIT:TERM]
Current Text:
`Python analysis tooling`
Issue:
`analysis tooling` is broader than the canonical host-side term `replay tooling`.
Proposed Fix:
`Python replay tooling`
Rationale:
The affected tooling in this document is replay-facing host tooling.
Evidence:
Canonical term definition for `replay tooling`; `docs/replay/tooling.md`:1-18

## Finding 6

File:
docs/architecture/architecture_whitepaper.md
Section / Line:
Stage 2 / Stage 3 / output table, lines 461-482 and 525
Category:
[AUDIT:CONSISTENCY]
Current Text:
`first divergent frame`
Issue:
This wording drifts from the canonical term `divergence`, defined as the first frame index where two replay hash streams differ, and from the stable field name `first_divergence_frame`.
Proposed Fix:
Use `first divergence frame` in prose.
Rationale:
This aligns prose with the field name and the canonical definition.
Evidence:
`docs/replay/DIVERGENCE_SEMANTICS.md`:39-64; `docs/architecture/architecture_whitepaper.md`:42, 523-526

## Finding 7

File:
docs/demos/demo_v4_region_attribution.md
Section / Line:
Problem statement / Multi-field rule, lines 5-6 and 38-39
Category:
[AUDIT:CONSISTENCY]
Current Text:
`first divergent frame`
Issue:
The demo prose uses `first divergent frame` instead of the canonical `first divergence frame`.
Proposed Fix:
Use `first divergence frame`.
Rationale:
This keeps the demo terminology aligned with `first_divergence_frame`.
Evidence:
`docs/replay/DIVERGENCE_SEMANTICS.md`:39-64

## Finding 8

File:
docs/demos/demo_v5_evolution.md
Section / Line:
Evolution classes, lines 30-32
Category:
[AUDIT:CONSISTENCY]
Current Text:
`first divergent frame includes \`sample_payload\``
Issue:
This prose drifts from the canonical term `first divergence frame`.
Proposed Fix:
`first divergence frame includes \`sample_payload\``
Rationale:
This is a wording-only normalization.
Evidence:
`docs/replay/DIVERGENCE_SEMANTICS.md`:39-64

## Finding 9

File:
docs/demos/demo.md
Section / Line:
Expected output, lines 34-41
Category:
[AUDIT:REF]
Current Text:
`First divergence frame: 8321`
Issue:
This text mirrors tool output rather than repo prose. Normalizing it would require deciding whether to preserve exact CLI output or rewrite the example.
Proposed Fix:
Report only.
Rationale:
This should remain unchanged in a bounded wording pass because it may be intentionally quoting the tool's emitted output.
Evidence:
`docs/demos/demo.md`:34-41; `scripts/artifact_diff.py` usage is referenced at line 14
