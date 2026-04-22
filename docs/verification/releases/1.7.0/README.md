# Release Evidence Bundle (1.7.0)

This directory is the retained release record for minor release `1.7.0`.

## Release Summary

Release `1.7.0` is a hardening and clarification cut for the replay-authoritative
primary precision CLI surface. This retained bundle records the active
non-firmware release evidence for that narrowed operator path.

## Scope of this cut

- replay-authoritative hardening for the primary `precision replay` path
- retained release evidence aligned to workspace version `1.7.0`
- no replay-host promotion
- no firmware release promotion
- no release-surface expansion beyond the documented primary precision CLI path

## Claim boundaries

This release retains a narrowed, exercised-path claim with the following limits:

- primary precision CLI release surface only
- retained command transcripts and release-check outputs for the active release baseline
- Tier-1 Kani evidence only unless heavier proofs are separately retained
- supporting replay and STM32 material remains bounded support/reference, not a widened release claim

## What is NOT claimed

- full contract closure for all paths
- universal path proof or exhaustive coverage
- replay-host guarantees as part of this release
- firmware release claim
- any release-surface expansion beyond the active documented boundary

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

This bundle represents the retained release-readiness record for `1.7.0` within
the stated boundaries and limitations.
