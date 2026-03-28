# NON-NORMATIVE / EXPERIMENTAL

This append-only log is exploratory only. It does not define current release surface,
verification authority, or normative invariants. If an item matures, promote it to
its target document; do not treat this log as authoritative.

## 2026-03-26 — Witness-model v2 direction exploration [WIP-002]
Status: proposed
Owner: architecture

Problem
The current witness-model direction is not yet concrete enough to decide what artifact shape, proof boundary, or integration path should replace or extend the existing model.

Hypothesis
A narrower witness-model v2 can improve replay evidence clarity if it is scoped as an explicit artifact contract candidate before any normative migration.

Constraints
- Must not redefine release surface or replay invariants from `docs/spec/` or `docs/MATH_CONTRACT.md`
- Must remain compatible with existing verification authority in `VERIFICATION_GUIDE.md`

Planned Artifacts
- `docs/roadmap/witness_model_direction.md`
- candidate notes or prototypes referenced from future WIP entries

Evidence Produced
- existing direction note at `docs/roadmap/witness_model_direction.md`
- this append-only experiment seed for follow-on exploration

Next Decision
Decide whether witness-model v2 merits a concrete candidate artifact contract draft or should remain roadmap-only.

Promotion Path
`docs/spec/`, `docs/architecture/`, or retained release evidence, depending on what is validated.

## 2026-03-26 — BeagleBone Black hostile-board isolation bring-up [WIP-001]
Status: closed (PASS-constrained)
Owner: hardware

Problem
Board bring-up on a hostile or weakly controlled BeagleBone Black path needs a concise place to track isolation assumptions, failure modes, and whether the board is useful for capture work.

Hypothesis
Treating BeagleBone Black bring-up as a hostile-board experiment will clarify which isolation controls are required before any capture evidence from that path can be trusted.

Constraints
- Must not imply supported release hardware or verification authority
- Must not redefine current capture contracts outside the existing replay and verification docs

Planned Artifacts
- bring-up notes under `docs/wip/`
- retained command logs or observations linked from future WIP entries

Evidence Produced
- `docs/wip/bbb_bringup_evidence_BBB-001.md` (retained evidence record)
- `make gate`: VERIFICATION PASSED (on-target bit-exact validation)
- `make replay-tests`: PASS (after `python3-serial` dependency correction)

Next Decision
Onboarding complete for BBB-001. Proceed to experiment scaffold / workload definition.

Promotion Path
`docs/hardware/`, `docs/operations/`, `docs/replay/`, or release evidence if the path becomes validated.

## 2026-03-26 — BeagleBone Black prudent hostile-board bring-up [WIP-003]
Status: proposed
Owner: hardware

Problem
An unknown-source BeagleBone Black requires an auditable bring-up path that avoids trusting onboard state, avoids early networking, and produces retained evidence before any limited experimental use.

Hypothesis
Offline reimage to known-good microSD media, followed by isolated first boot and explicit surface reduction, yields an acceptable constrained experimental platform without implying trusted hardware.

Constraints
- Must not trust onboard eMMC state
- Must not use early network connectivity during initial isolation
- Must not create release-surface, specification, or verification authority claims

Planned Artifacts
- `docs/wip/bbb_prudent_bringup.md`
- `docs/wip/templates/bringup_evidence_checklist.md`

Evidence Produced
- WIP hostile-board bring-up procedure with T0-T5 gates
- compact retained-evidence checklist for operator use

Next Decision
Decide whether a specific board instance passes T5 for constrained experimental use, should remain on HOLD, or must be quarantined.

Promotion Path
`docs/hardware/` only after successful gated bring-up, retained evidence, and repeatability across later validation.

## 2026-03-27 — Quantization divergence witness experiment [WIP-004]
Status: closed (PASS-constrained)
Owner: signal
Phase 2 (matrix extension): appended host-only evidence; does not reopen phase 1.

Problem
We need a minimal, deterministic experiment that demonstrates divergence localization under quantization using the Precision signal/replay pipeline.

Hypothesis
A small fixed pipeline with baseline vs quantized paths and per-stage artifact emission will produce a reproducible first-divergence witness and classification.

Constraints
- Must be fully deterministic (no FP nondeterminism, no threading)
- Must use existing artifact + replay + diff tooling
- Must not modify release surface
- Must remain host-executable (BBB) without PRU

Planned Artifacts
- `experiments/quantization_probe/`
- baseline and quantized pipeline implementations
- fixed input corpus
- artifact outputs A/B
- diff output demonstrating first divergence

Evidence Produced
- BBB host Linux execution confirmed for the phase-1 witness path
- `make gate`: PASS on BBB host
- baseline artifact rerun hash: `67e309b08d7bf8db286869b2b81a23da297b7ccfd2ecd9e322830729e69a9e69`
- quantized artifact rerun hash: `fe992bec716077dc20eb94550d007022439fef871a1bf101a30727b2d18a8abf`
- first divergence: `frame_idx=4` (reported as `First divergence frame: 4`)
- `shape_class=persistent_offset`
- `primary_region=sample_payload`
- `evolution_class=bounded_persistent`
- controlled host matrix retained for `C1-Q2`, `C1-Q3`, `C1-Q4`, and `C2-Q3`
- matrix result summary:
  `C1-Q2` and `C1-Q3` stay at `frame_idx=4` with `shape_class=persistent_offset`
  `C1-Q4` moves first divergence to `frame_idx=0` with `shape_class=rate_divergence`
  `C2-Q3` moves first divergence to `frame_idx=7` with `shape_class=persistent_offset`
- retained experiment note, exact commands, and case hashes/classifications:
  `experiments/quantization_probe/README.md`

Next Decision
Rerun the same controlled matrix on BBB host if BBB-specific parity is required; otherwise keep the result experiment-local and avoid framework expansion.

Promotion Path
experiment-local retention only; reconsider `docs/replay/` or `docs/architecture/` only if a later phase produces broader validated evidence
