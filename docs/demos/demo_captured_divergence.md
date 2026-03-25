# Captured Divergence Demonstration

## Purpose

This page is a reference demonstration artifact. Release status is classified
in [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md); verification questions route to
[VERIFICATION_GUIDE.md](../../VERIFICATION_GUIDE.md). Demo pages must not introduce new capability claims
beyond what is already evidenced elsewhere. This page shows that divergence
classification works on artifacts captured from two separate STM32 executions.

## Artifact Pair

The demonstration uses this artifact pair:

- `artifacts/demo_captured/run_A.rpl`
- `artifacts/demo_captured/run_B.rpl`

Both files are hardware captures from separate executions. They are not
synthetic fixtures and they are not post-processed artifacts.

## Verification Workflow

Run the demonstration workflow:

```bash
make demo-captured-release
```

Expected key output fields:

```text
first_divergence_frame: 4096
primary_region: sample_payload
all_regions_at_first_divergence: [sample_payload]
region_summary: sample_payload
shape_class: transient
evolution_class: self_healing
timeline_summary: divergence resolves within 1 frame
```

because `demo-divergence` perturbs exactly one captured sample and canonical
behavior resumes on the next frame.

## Behavioral Summary

The committed captured artifact pair is not identical.

Observed divergence behavior:

- first divergence frame: `4096`
- differing region: `sample_payload`
- divergence class: `transient`
- evolution class: `self_healing`

This explanatory evidence page shows that the released replay-facing Python
tooling classifies a real captured divergence artifact pair from separate
hardware executions.

## Boundaries

This reference demonstration does not change:

- the `RPL0` artifact contract
- frame size
- schema structure
- hashing rules
- `scripts/artifact_diff.py`

This is an evidence-packaging milestone, not a release-surface change.

## Artifact Evidence Hashes

- `run_A.rpl` SHA256: `b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3`
- `run_B.rpl` SHA256: `5957b539722fdd0021b56882c7eb04b9c68ef16484c18b6293cb4ff0d80a5d6d`
