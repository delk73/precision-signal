# Release Evidence Bundle (1.8.0)

This directory is the retained release record for minor release `1.8.0`.

## Release Summary

Release `1.8.0` is a firmware-inclusive release adding the `irregular-timing`
feature to `replay-fw-f446` for deterministic varying-interval stimulus, along
with `dpw4` crate cleanup and new design documentation.

## Scope of this cut

- Added `irregular-timing` feature to `replay-fw-f446` for deterministic varying-interval stimulus
- Implemented `irregular_arr_for_interval` pseudo-random timing modulation for improved drift characterization
- Added `docs/notes/power_characterization.md` design note
- Cleaned up `dpw4` crate: removed redundant casts, optimized phase-generation loops
- Enforced compile-time isolation between `irregular-timing` and demo-divergence features

## Claim boundaries

This release retains a narrowed, exercised-path claim with the following limits:

- primary precision CLI release surface only
- retained command transcripts and release-check outputs for the active release baseline
- Tier-1 Kani evidence only unless heavier proofs are separately retained
- supporting replay and STM32 material remains bounded support/reference, not a widened release claim
- `irregular-timing` is feature-gated and not default; `replay-fw-f446` remains experimental

## What is NOT claimed

- full contract closure for all paths
- universal path proof or exhaustive coverage
- replay-host guarantees as part of this release
- firmware release claim beyond the experimental classification
- any release-surface expansion beyond the active documented boundary

## Retained files in this bundle

- [cargo_check_dpw4_thumb_locked.txt](cargo_check_dpw4_thumb_locked.txt)
- [kani_evidence.txt](kani_evidence.txt)
- [make_demo_evidence_package.txt](make_demo_evidence_package.txt)
- [make_doc_link_check.txt](make_doc_link_check.txt)
- [make_gate.txt](make_gate.txt)
- [make_release_bundle_check.txt](make_release_bundle_check.txt)
- [make_replay_tests.txt](make_replay_tests.txt)
- [release_reproducibility.txt](release_reproducibility.txt)

## Release decision record

This bundle represents the retained release-readiness record for `1.8.0` within
the stated boundaries and limitations.
