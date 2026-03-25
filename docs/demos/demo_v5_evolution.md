# Divergence Evolution Semantics Demo (V5)

## 1. Problem statement

Demo V4 explains where divergence starts and which supported region first differs.
Demo V5 extends the same artifact-first comparison to classify what happens after
that first divergence point.

Demo V5 extends the frozen ladder without redefining it:

```text
perturbation -> localization -> shape -> region -> evolution -> captured evidence
```

## 2. Evolution classes

`scripts/artifact_diff.py` emits exactly these evolution labels:

- `self_healing`
- `bounded_persistent`
- `monotonic_growth`
- `region_transition`

Rule precedence is fixed and deterministic:

1. `region_transition`: a later divergent frame contains any supported region
   that was not present in `all_regions_at_first_divergence`.
2. `self_healing`: no new region appears, and after some later frame all
   remaining compared frames are identical.
3. `monotonic_growth`: no new region appears, divergence does not heal, the
   first divergence frame includes `sample_payload`, and the absolute
   sample-difference magnitude is nondecreasing with at least one strict
   increase through the remaining compared frames.
4. `bounded_persistent`: any remaining supported divergent pair. Divergence
   persists through the final frame without introducing a new region.

These rules use only the compared artifact bytes and supported frame regions:

- `header_schema`
- `timer_delta`
- `irq_state`
- `sample_payload`

## 3. Timeline summary

Demo V5 also emits `timeline_summary` as a compact explanation derived from the
same deterministic rules:

- `self_healing`: reports how many frames it takes to resolve.
- `bounded_persistent`: reports the final bounded region set.
- `monotonic_growth`: reports first and final sample-difference magnitudes and
  the ending frame.
- `region_transition`: reports the first frame where a new supported region
  appears.

## 4. Fixtures

Deterministic fixtures are generated into `artifacts/demo_v5/` by:

```bash
python3 scripts/generate_demo_v5_fixtures.py
```

Produced pairs:

- `self_healing_A.rpl`, `self_healing_B.rpl`
- `bounded_persistent_A.rpl`, `bounded_persistent_B.rpl`
- `monotonic_growth_A.rpl`, `monotonic_growth_B.rpl`
- `region_transition_A.rpl`, `region_transition_B.rpl`

Fixture intent:

- `self_healing`: one-frame `sample_payload` divergence at frame `4096`, then
  exact re-alignment.
- `bounded_persistent`: constant `sample_payload` offset from frame `4096`
  through the end.
- `monotonic_growth`: `sample_payload` difference grows by one additional count
  per frame from frame `4096` onward.
- `region_transition`: divergence begins in `timer_delta` at frame `4096` and
  first appears in `sample_payload` at frame `4098`.

## 5. Workflow

```bash
make demo-v5-verify
make demo-v5-audit-pack
make demo-v5-record
make demo-v5-release
```

## 6. Example output

```text
first_divergence_frame: 4096
shape_class: transient
primary_region: sample_payload
all_regions_at_first_divergence: [sample_payload]
region_summary: sample_payload
evolution_class: self_healing
timeline_summary: divergence resolves within 1 frame
```

Interpretation:

- divergence starts at frame `4096`
- the first differing supported region is `sample_payload`
- the immediate sample-shape label is `transient`
- the full post-divergence trajectory is `self_healing`
