# Divergence Semantics

**Document revision:** 1.4.0  
**Applies to:** release 1.4.0

## Versioning Terminology

- Document revision labels editorial history for this explanation contract.
- Release versions identify the shipped software release.
- RPL/header versions remain part of the portable replay format contract and do not version this explanation document.

This document defines the normative explanation contract emitted by
`scripts/artifact_diff.py`.

It freezes the deterministic divergence semantics used by the replay demo
ladder. It does not change the portable replay format, fixture generation, capture logic,
or comparison algorithms.

This contract belongs to the released execution-analysis surface only. Other
[docs/wip/](../wip/) material remains outside the current release surface unless
explicitly promoted.

If this document disagrees with implementation, the mismatch is a defect in the
documentation or implementation and must be resolved explicitly. This document
describes the intended stable behavior of the current implementation.

## Scope

The explanation surface is the deterministic ladder:

```text
perturbation
-> localization
-> shape
-> region
-> evolution
-> captured evidence
```

`artifact_diff.py` operates only on compared RPL bytes and supported
RPL fields. It does not infer cause, physical source, or nondeterministic
behavior.

## Stable Output Fields

The stable explanation contract consists of these emitted fields:

- `first_divergence_frame`
- `primary_region`
- `all_regions_at_first_divergence`
- `region_summary`
- `shape_class`
- `evolution_class`
- `timeline_summary`

Field meaning:

- `first_divergence_frame`: the first frame index at which supported divergence
  is observed. Value is `none` only when compared RPL files are identical.
- `primary_region`: one supported region selected by fixed precedence from
  `all_regions_at_first_divergence`, or `none` when compared RPL files are identical.
- `all_regions_at_first_divergence`: ordered list of all supported regions
  present at the first divergent frame.
- `region_summary`: `none` when no divergence exists, the single region name
  when exactly one supported region differs first, or `mixed` when more than
  one supported region differs first.
- `shape_class`: sample-payload divergence shape label, or `none` when the
  first divergence does not include `sample_payload`.
- `evolution_class`: post-divergence trajectory label, or `none` when compared RPL files
  are identical.
- `timeline_summary`: deterministic summary sentence derived from
  `evolution_class`, or `none` when compared RPL files are identical.

`artifact_diff.py` may also emit implementation-detail lines such as
`reconvergence_summary`. Those lines are derivative helpers, not additional
classification classes.

## Supported Regions

Only these regions participate in the explanation contract:

- `header_schema`
- `timer_delta`
- `irq_state`
- `sample_payload`

Region definitions:

- `header_schema`: any difference in RPL header metadata or any byte before
  the frame stream starts. This includes version, frame size, frame count, and
  any non-frame prefix byte mismatch.
- `timer_delta`: difference in the per-frame `timer_delta` field.
- `irq_state`: difference in the per-frame `irq_id` field.
- `sample_payload`: difference in the per-frame `input_sample` field.

Differences in unsupported per-frame fields (`frame_idx`, `flags`, `reserved`,
or an unexplained raw-byte mismatch) are not classified. The tool exits with
`FAIL` instead of inventing an explanation label.

## Localization Rules

Localization is determined in this order:

1. Compare header and non-frame prefix bytes.
2. If header/schema differs, anchor `first_divergence_frame` to `0`.
3. Otherwise scan frames in order and stop at the first frame containing a
   supported region difference.
4. If artifacts are identical through all compared frames and header bytes,
   emit `first_divergence_frame: none`.

When header/schema differs and frame `0` also differs in supported frame
regions, `all_regions_at_first_divergence` includes both `header_schema` and
those frame-`0` regions.

## Region Ordering and Precedence

The region ordering is fixed:

```text
header_schema
timer_delta
irq_state
sample_payload
```

This ordering governs:

- `all_regions_at_first_divergence`
- `primary_region`
- first appearance ordering inside evolution classification

`primary_region` is the first region in the fixed precedence list that appears
in `all_regions_at_first_divergence`.

## Shape Classes

Shape classification is evaluated only when `sample_payload` is present in
`all_regions_at_first_divergence`.

Allowed values are exactly:

- `transient`
- `persistent_offset`
- `rate_divergence`

Let `N = first_divergence_frame`, `diff[i] = sample_A[i] - sample_B[i]` for
`i >= N`, and `K = 8`.

Rule precedence is fixed:

1. `transient`
   - There exists `r` in `N+1 .. N+K` such that `diff[r] == 0`.
   - `diff[j] == 0` for all `j` from `r` through `min(N+K, end)`.
2. `persistent_offset`
   - `diff[i]` is constant for all remaining compared frames.
3. `rate_divergence`
   - Not `transient`
   - Not `persistent_offset`
   - `|diff[i+1]| >= |diff[i]|` for all remaining compared frames
   - At least one strict increase exists

If `sample_payload` participates in the first divergence but none of these
rules match, the tool exits with `FAIL`.

If `sample_payload` is absent from the first divergence, `shape_class` is
`none`.

## Evolution Classes

Evolution classification describes what happens after the first divergence.

Allowed values are exactly:

- `self_healing`
- `bounded_persistent`
- `monotonic_growth`
- `region_transition`

Rule precedence is fixed and must be applied in this exact order:

1. `region_transition`
   - A later divergent frame contains any supported region not present in
     `all_regions_at_first_divergence`.
2. `self_healing`
   - No new supported region appears.
   - After some later frame, all remaining compared frames are identical.
3. `monotonic_growth`
   - No new supported region appears.
   - Divergence does not heal.
   - `sample_payload` is present in `all_regions_at_first_divergence`.
   - The absolute sample-difference magnitude is nondecreasing across the
     remaining compared frames, with at least one strict increase.
4. `bounded_persistent`
   - Any remaining supported divergent pair.
   - Divergence persists through the final compared frame without introducing a
     new supported region.

## Timeline Summary Rules

`timeline_summary` is derived directly from `evolution_class`:

- `region_transition`: report the first frame where a new supported region
  appears and name that region set.
- `self_healing`: report how many frames it takes for divergence to resolve.
- `monotonic_growth`: report first and final sample-difference magnitudes and
  the ending frame.
- `bounded_persistent`: report the final supported region set that remains
  divergent through the final frame.

The summary is explanatory only. It must remain consistent with the fixed
classification precedence above.

## Identical Artifact Case

When no divergence exists:

- `first_divergence_frame: none`
- `primary_region: none`
- `all_regions_at_first_divergence: []`
- `region_summary: none`
- `shape_class: none`
- `evolution_class: none`
- `timeline_summary: none`

## Demo Ladder Mapping

The replay explanation ladder is frozen as:

```text
perturbation -> localization -> shape -> region -> evolution -> captured evidence
```

Current demo mapping:

- Demo capture perturbation: controlled perturbation introduction
- Demo V2: first-divergence localization
- Demo V3: shape classification
- Demo V4: region attribution
- Demo V5: evolution semantics
- Captured divergence demo: hardware capture evidence using the same stable
  explanation contract
