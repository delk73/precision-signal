# Audits Index

- `AUDIT_CONVENTIONS.md`: Canonical P-axis / backlog / state-token naming convention.
- `PRE_RELEASE_AUDIT.md`: Canonical P5 pre-release policy and execution guidance.
- `P5_artifact_boundary.md`: P5A/P5B artifact boundary policy, mechanism, and evidence commands.
- `delta-02_gain-mantissa-invariance.md`: Δ-02 validate-path gain mantissa invariance close-out ledger.
- `delta-03_triangle-dpw4-integrity.md`: Δ-03 triangle integrity and control-surface evidence summary.
- `delta-04_sine-quantizer-semantics.md`: Δ-04 sine quantizer/scaling semantics close-out ledger.
- `delta-bd_build-determinism.md`: Δ-BD build reproducibility hardening and evidence.

Public historical audit snapshots are intentionally curated in this repository
artifact. Removed internal or superseded ledgers are not part of the shipped
public surface.

## Demo Lifecycle Protocol

Canonical demo operator lifecycle should remain first-class make targets:
- `make <demo>-capture`
- `make <demo>-verify`
- `make <demo>-audit-pack`
- `make <demo>-record`

## Release Closeout Sequence

Tagged release closeout order is:

1. tag
2. verify
3. audit-pack
4. record
5. publish

## Runlog Retention

`docs/audits/runlogs/` is an ignored payload directory (`.gitkeep` only).
Runlogs must be retained through the selected release/archive process outside
normal git tracking.
