# Divergence Region Attribution Demo (V4)

## 1. Problem statement

Demo V3 localizes the first divergence frame and classifies post-divergence sample
shape. Demo V4 adds region attribution for that same first divergence point:
which supported region differs first, without inferring cause.

Demo V4 extends the frozen ladder without redefining it:

```text
perturbation -> localization -> shape -> region -> evolution -> captured evidence
```

## 2. Supported regions

`scripts/artifact_diff.py` emits exactly these region labels:

- `header_schema`
- `timer_delta`
- `irq_state`
- `sample_payload`

Definitions:

- `header_schema`: any difference in the pre-frame header or schema byte region.
- `timer_delta`: the frame timing delta field differs.
- `irq_state`: the IRQ id field differs.
- `sample_payload`: the sample value field differs.

If header/schema differs, Demo V4 anchors `first_divergence_frame` to `0` and
observes supported frame-`0` regions in the same report when they are also
present. If a divergence is confined to header/schema bytes, the report remains
single-region at frame `0`.

## 3. Multi-field rule

If more than one supported region differs at the first divergence frame,
`all_regions_at_first_divergence` lists each concrete region and
`region_summary` is `mixed`.

`primary_region` is selected by fixed precedence:

```text
header_schema
timer_delta
irq_state
sample_payload
```

`mixed` never appears inside `all_regions_at_first_divergence`.

## 4. Shape classification interaction

Demo V4 preserves Demo V3 sample-shape classification for cases where
`sample_payload` is part of the first divergence:

- `transient`
- `persistent_offset`
- `rate_divergence`

If the first divergence does not include `sample_payload`, Demo V4 reports
`shape_class: none`.

`reconvergence_summary` is emitted only when it is directly derivable from a
`transient` sample divergence.

## 5. Fixtures

Deterministic fixtures are generated into `artifacts/demo_v4/` by:

```bash
python3 scripts/generate_demo_v4_fixtures.py
```

Produced pairs:

- `header_schema_A.rpl`, `header_schema_B.rpl`
- `header_schema_sample_payload_A.rpl`, `header_schema_sample_payload_B.rpl`
- `timer_delta_A.rpl`, `timer_delta_B.rpl`
- `irq_state_A.rpl`, `irq_state_B.rpl`
- `sample_payload_A.rpl`, `sample_payload_B.rpl`
- `mixed_A.rpl`, `mixed_B.rpl`

The `sample_payload` pair is a persistent offset fixture and the `mixed` pair
combines `timer_delta` plus `sample_payload` at frame `4096`. The
`header_schema_sample_payload` pair combines header/schema divergence with a
persistent sample divergence starting at frame `0`.

## 6. Workflow

```bash
make demo-v4-verify
make demo-v4-audit-pack
make demo-v4-record
make demo-v4-release
```
