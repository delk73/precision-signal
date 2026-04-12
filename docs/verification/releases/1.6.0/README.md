# Release Evidence Bundle (1.6.0)

This directory is the retained release record for minor release `1.6.0`.

## Release Summary

Release `1.6.0` is a narrowed non-firmware retained bundle covering the primary precision command-line surface verification under Tier-1 claim boundaries.

## Scope of this cut

- narrowed primary precision release claim: `common/mod.rs`, `precision/mod.rs`, and `precision_authoritative_surface.rs`
- no replay-host scope expansion
- non-firmware verification path only
- formal release-bundle coherence checkpoint

## Claim boundaries

This release makes a narrowed Tier-1 claim with the following documented limits:

- Command boundary: primary precision operator surface reviewed and narrowed, but full impossibility proof is not present
- Result block coherence: exercised-path evidence covers seven-line output and result.txt/stdout identity
- Publication integrity: staging collision, rename-failure behavior, and exercised publish paths covered
- Error/exit mapping: exercised invalid invocations and schema failures map correctly; not exhaustive coverage
- Schema/class rejection: invalid meta, trace, comparison, divergence, incompatibility cases exercised

## What is NOT claimed

- Full contract closure for all paths
- Universal path proof or complete exhaustivity
- Full run_id randomness guarantees
- Same-filesystem/durability proof
- Exhaustive mock-mode or trace-edge coverage
- Replay-host guarantees as part of this release
- Firmware release claim

## Retained files in this bundle

- README.md
- index.md
- cargo_check_dpw4_thumb_locked.txt
- kani_evidence.txt
- make_demo_evidence_package.txt
- make_doc_link_check.txt
- make_gate.txt
- make_replay_tests.txt
- release_reproducibility.txt

## Release decision record

This bundle represents a formal release-readiness checkpoint. The narrowed Tier-1 claim is recorded as release-ready within the stated boundaries and limitations.

No known false Tier-1 claims remain inside the narrowed scope boundary.
