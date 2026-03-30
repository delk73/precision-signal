# NON-NORMATIVE / EXPERIMENTAL

This template is exploratory only. It does not define current release surface,
verification authority, or normative invariants.

## 2026-03-30 — Pi live-signal bring-up boundary [WIP-PI-BOUNDARY-001]
Status: closed (PASS-constrained)
Owner: signal

Problem
A rapid live-signal source was needed to validate bench scope setup and end-to-end
signal-plumbing before investing in a deterministic witness source.

Hypothesis
Pi userspace GPIO could serve as a quick bring-up stimulus for scope visibility
and pipeline exercise, even if not suitable as a validation-grade witness.

Constraints
- non-normative only
- Pi userspace output is not promoted into trusted witness semantics
- no release claim is derived from this path

Planned Artifacts
- docs/wip/pi_signal_bringup_boundary.md

Evidence Produced
- Tektronix 2235 baseline recovered to known-good trace state
- onboard CAL signal used to confirm basic scope/probe function
- scope baseline recovery succeeded
- probe quality and compensation limits did not block basic bring-up
- Pi live pulse train observed on scope
- Pi output was sufficient for signal presence and plumbing validation
- pulse spacing and period wander observed on Pi output
- observed horizontal pulse wander is treated as timing jitter
- Pi lane accepted for bring-up only and excluded from the trusted witness boundary

Next Decision
Use STM32 timer/peripheral-driven generation as the next witness source for repeatability and diff-valid validation.

Promotion Path
None directly. Boundary conclusion may later be summarized in a higher-level
experimental roadmap note, but not in normative docs unless the witness
architecture is promoted.
