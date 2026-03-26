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
Status: proposed
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
- no validated artifacts yet
- this append-only experiment seed establishing the decision path

Next Decision
Decide whether the board can be isolated enough to justify a bounded bring-up procedure.

Promotion Path
`docs/hardware/`, `docs/operations/`, `docs/replay/`, or release evidence if the path becomes validated.
