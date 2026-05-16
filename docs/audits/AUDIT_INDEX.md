# Audits Landing

Audit material is support, retained evidence, or historical close-out material.
It does not override the active authority path in
[START_HERE.md](../START_HERE.md). Release decisions route through
[VERIFICATION_GUIDE.md](../VERIFICATION_GUIDE.md) and
[RELEASE_SURFACE.md](../RELEASE_SURFACE.md).

## Audit Routes

| Route | Class | Scope |
| --- | --- | --- |
| [AUDIT_CONVENTIONS.md](AUDIT_CONVENTIONS.md) | active support | P-axis, backlog, and state-token naming convention. |
| [PRE_RELEASE_AUDIT.md](PRE_RELEASE_AUDIT.md) | active support | P5 pre-release policy and execution guidance. |
| [P5_artifact_boundary.md](P5_artifact_boundary.md) | active support | P5A/P5B artifact boundary policy and evidence commands. |
| [ci_pins.md](ci_pins.md) | active support | CI pin support notes. |
| [delta-02_gain-mantissa-invariance.md](delta-02_gain-mantissa-invariance.md) | retained release evidence | Delta close-out ledger. |
| [delta-03_triangle-dpw4-integrity.md](delta-03_triangle-dpw4-integrity.md) | retained release evidence | Triangle integrity evidence summary. |
| [delta-04_sine-quantizer-semantics.md](delta-04_sine-quantizer-semantics.md) | retained release evidence | Sine quantizer/scaling close-out ledger. |
| [delta-bd_build-determinism.md](delta-bd_build-determinism.md) | retained release evidence | Build reproducibility hardening and evidence. |
| [repository_health_baseline.md](repository_health_baseline.md) | historical/demo | Historical repository-health baseline. |
| [repository_health_1.2.2.md](repository_health_1.2.2.md) | historical/demo | Historical repository-health snapshot. |
| [repository_health_1.4.0.md](repository_health_1.4.0.md) | historical/demo | Historical repository-health snapshot. |
| [repository_health_1.5.0.md](repository_health_1.5.0.md) | historical/demo | Historical repository-health snapshot. |
| [repository_health_1.6.0.md](repository_health_1.6.0.md) | historical/demo | Historical repository-health snapshot. |

Public historical audit snapshots are intentionally curated in this repository
artifact. Removed internal or superseded ledgers are not part of the shipped
public surface.

## Historical Demo Lifecycle Protocol

The historical demo operator lifecycle used these command shapes:
- `make <demo>-capture`
- `make <demo>-verify`
- `make <demo>-audit-pack`
- `make <demo>-record`

## Historical Release Closeout Sequence

Tagged release closeout order was recorded as:

1. tag
2. verify
3. audit-pack
4. record
5. publish

## Runlog Retention

`docs/audits/runlogs/` is primarily an ignored payload directory. The committed
exceptions are limited non-release samples such as `ci_local_sample.txt`;
operational runlogs must still be retained through the selected release/archive
process outside normal git tracking.
