# Release Evidence Bundle (1.6.0)

This directory is the retained release record for minor release `1.6.0`.

## Release Summary

Release `1.6.0` is a narrowed retained release record for the primary precision command-line surface, with bounded supporting firmware capture/import evidence retained under `fw_capture/`, `fw_repeat/`, and `supplemental/firmware_evidence.md`.

## Scope of this cut

- narrowed primary precision release claim: `common/mod.rs`, `precision/mod.rs`, and `crates/dpw4/tests/precision_authoritative_surface.rs`
- no replay-host scope expansion
- primary CLI verification path, with bounded supporting firmware capture/import evidence retained
- formal release-bundle coherence checkpoint

## Claim boundaries

This release makes a narrowed Tier-1 claim with the following documented limits:

- Command boundary: primary precision operator surface reviewed and narrowed, but full impossibility proof is not present
- Result block coherence: exercised-path evidence covers seven-line output and result.txt/stdout identity
- Publication integrity: staging collision, rename-failure behavior, and exercised publish paths covered
- Error/exit mapping: exercised invalid invocations and schema failures map correctly; not exhaustive coverage
- Schema/class rejection: invalid meta, trace, comparison, divergence, incompatibility cases exercised

Tier-2 Kani proofs are excluded from this retained bundle and remain optional unless separately retained under `VERIFICATION_GUIDE.md`.

## What is NOT claimed

- Full contract closure for all paths
- Universal path proof or complete exhaustivity
- Full run_id randomness guarantees
- Same-filesystem/durability proof
- Exhaustive mock-mode or trace-edge coverage
- Replay-host guarantees as part of this release
- Firmware release claim

## Retained files in this bundle

### Release-claim evidence

- [cargo_check_dpw4_thumb_locked.txt](cargo_check_dpw4_thumb_locked.txt)
- [kani_evidence.txt](kani_evidence.txt)
- [make_demo_evidence_package.txt](make_demo_evidence_package.txt)
- [make_doc_link_check.txt](make_doc_link_check.txt)
- [make_gate.txt](make_gate.txt)
- [make_replay_tests.txt](make_replay_tests.txt)
- [precision_authoritative_surface_test_evidence.txt](precision_authoritative_surface_test_evidence.txt)
- [release_reproducibility.txt](release_reproducibility.txt)

Reviewers rerunning `precision_authoritative_surface.rs` must enable the `cli` feature; the retained command is `cargo test -p dpw4 --test precision_authoritative_surface --features cli --locked`.

### Supporting / supplemental

- [fw_capture/](fw_capture/)
- [fw_repeat/](fw_repeat/)
- [fw_capture_hash_check.txt](fw_capture_hash_check.txt)
- [fw_repeat_hash_check.txt](fw_repeat_hash_check.txt)
- [supplemental/firmware_evidence.md](supplemental/firmware_evidence.md): bounded supporting firmware capture/import evidence only; not a firmware release claim

## Release decision record

This bundle represents a formal release-readiness checkpoint. The narrowed Tier-1 claim is recorded as release-ready within the stated boundaries and limitations.

No known false Tier-1 claims remain inside the narrowed scope boundary.
