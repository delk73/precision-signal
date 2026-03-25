# RC5 Debt and Planning Ledger (Non-Normative)
**Status:** NON-NORMATIVE project management artifact.
**Scope:** Planning, debt tracking, and future delta proposals. This file is not part of the locked math contract.

## OQ/DIV/Δ Closure Maps (Non-Normative)

**Historical tracing; not part of the normative contract.**

### OQ Tracking

| OQ-ID | Closure Delta(s) | State | Notes |
|---|---|---|---|
| OQ-1 | Δ-01 | CLOSED | `to_num::<u32>()` fixed→u32 conversion |
| OQ-2 | Δ-02 | CLOSED | Quick-mode mantissa singleton |
| OQ-3 | Δ-03 | CLOSED | Triangle `e4` calibration rationale |
| OQ-4 | Δ-03 | CLOSED | Triangle I256 overflow safety |
| OQ-5 | Δ-04 | CLOSED | Sine bit-extraction equivalence |
| OQ-6 | Δ-04 | CLOSED | Sine amplitude boundedness |
| OQ-7 | Δ-05 | CLOSED | `geom-signal` hermeticity |

### DIV Tracking

| DIV-ID | Closure Delta(s) | State | Notes |
|---|---|---|---|
| DIV-01 | Δ-06 | CLOSED | Radians domain reconciliation |
| DIV-02 | Δ-11 | DEFERRED | Gain default clarification |
| DIV-03 | Δ-07 | CLOSED | Triangle egress path reconciliation |
| DIV-04 | Δ-08 | CLOSED | Pulse reset semantics |
| DIV-05 | Δ-07 | CLOSED | Triangle algorithm reconciliation |
| DIV-06 | Δ-09a | CLOSED | Sine gain-bypass semantics |
| DIV-07 | Δ-10 | CLOSED | Header pad discrepancy |

### Delta Ledger

| Δ-ID | Title | Scope | State | Acceptance |
|---|---|---|---|---|
| Δ-01 | fixed->u32 conversion semantics | rc5 | CLOSED | Tier-1 Kani proofs pass on pinned toolchain |
| Δ-02 | quick validate mantissa singleton | rc5 | CLOSED | `quick_validate_gain_mantissa_is_singleton` passes |
| Δ-03 | triangle integrator safety/calibration docs | rc5 | CLOSED | proofs/tests/docs aligned |
| Δ-04 | sine scaling + quantizer proof closure | rc5 | CLOSED | sine proofs/tests pass |
| Δ-05 | dependency pinning / lock discipline | rc5 | CLOSED | `--locked` gates pass |
| Δ-06 | radians domain reconciliation | rc5 | CLOSED | docs reconciled |
| Δ-07 | triangle/dpw_gain doc reconciliation | rc5 | CLOSED | docs match code paths |
| Δ-08 | pulse reset semantics reconciliation | rc5 | CLOSED | docs match code behavior |
| Δ-09a | sine gain-bypass semantics normative | rc5 | CLOSED | DIV-06 closed in rc5 |
| Δ-09B | sine local calibration (post Δ-02 lock) | rc5 | CLOSED | validate quick + pinned hashes pass |
| Δ-09C | cross-domain gain unification | post-LOCK major | OPEN (future) | normative raw-Q definition + controlled rebaseline plan + oracle pass |
| Δ-10 | header pad discrepancy reconciliation | rc5 | CLOSED | docs/tests aligned |
| Δ-11 | gain-default scope clarification | post-v1 | DEFERRED | doc-only clarification retained |
| Δ-BD | release reproducibility hardening | rc5 | CLOSED | repro canary passes |
| Δ-06-CS | triangle freeze control-surface invariant | rc5 | CLOSED | tier-1 proof + deterministic test pass |

## Debt Register

- `DEBT-002`: Gain-stage implementation hardening follow-up.
  - Current state: closed in rc5 implementation path.
  - Notes: kept for historical traceability.
- `DEBT-003: Cross-Domain Gain Unification (Post-LOCK Major)`
  - Current state: open, explicitly out of rc5 scope.
  - Intent: if pursued, define a normative raw-Q mapping for CORDIC output and migration/rebaseline plan.
