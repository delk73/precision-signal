# End-to-End Evidence Packaging

## Purpose

This page packages the completed Phase 1 through Phase 5 replay work into one
proof path and one retained evidence bundle.

It does not change the replay contract, the capture contract, firmware
behavior, or divergence semantics. It packages already-retained inputs and
expected outputs into one reproducibility path.

This is the canonical packaged proof route for the completed replay pipeline.
It does not expand the release surface. Released operator tooling remains the
Python replay toolchain and the `precision` validation CLI surface classified in
[docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md).

## Canonical locations

- inputs: `artifacts/demo_evidence/inputs/`
- retained bundle: `artifacts/demo_evidence/retained/`
- generated verification output: `artifacts/demo_evidence/_generated/`

The generated directory is scratch space recreated by the operator command.
The retained directory is the canonical evidence bundle checked into the repo.
That retained bundle is the canonical proof bundle for this packaged replay
path.

## Operator path

From a fresh clone, run exactly:

```bash
make demo-evidence-package
```

That command:

1. validates the retained baseline and perturbed CSV fixtures
2. imports both fixtures into canonical `.rpl` artifacts
3. records artifact hashes
4. records `replay-host diff` outputs for identical and perturbed comparisons as
   experimental reference output
5. records `artifact_diff.py` outputs for identical and perturbed comparisons as
   the released replay-facing operator result
6. records the exact command list and transcript
7. verifies that every regenerated file matches the retained bundle byte-for-byte

## Expected replay results

The canonical path must reproduce:

- identical comparison: `no divergence`
- perturbed comparison: `first divergence at frame 17`

The artifact-level summary must also preserve:

- `first_divergence_frame: none` for the identical pair
- `first_divergence_frame: 17` for the perturbed pair
- `shape_class: transient`
- `primary_region: sample_payload`
- `evolution_class: self_healing`

## Retained evidence

The retained bundle contains:

- canonical input fixtures
- imported artifact hashes
- replay diff outputs
- artifact diff outputs
- exact command list
- exact transcript
- manifest with pinned input and output hashes

## Scope boundary

This packaging step is evidence-only. It does not add new experiments, new demo
claims, firmware changes, replay-tool redesign, or release-surface promotion of
experimental components.
