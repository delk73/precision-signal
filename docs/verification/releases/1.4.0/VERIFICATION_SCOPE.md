# Verification Scope and Limits (1.4.0)

## Released Correctness Claim

Bounded correctness claim for `1.4.0`:

- Domain `D`: the released sine egress path evaluated at the deterministic finite phase set `phase_i = i * 2π / 4096` for `i ∈ {0, ..., 4096}`, with default sine gain routing (`shape=4`, `DpwGain::new(1 << 63, 0, 0, 0)`).
- Observable compared: normalized emitted sample `sample_i / (SINE_EGRESS_SCALE_Q31 >> HEADROOM_BITS)`.
- Reference: `libm::sin(phase_i)` under the pinned Rust `1.91.1` environment.
- Statement: over `D`, the retained evidence in `cargo_test_sine_bounded_correctness.txt` records the observed max absolute residual and requires it to remain at or below `1.0e-6`.

This is an empirical bounded correctness statement. It is not a proof of global sine equivalence outside `D`.

## Composition-Level Invariant

Active triangle-path statement for `1.4.0`:

- If the triangle discontinuity guard freezes the tick (`dphi > DISCONTINUITY_THRESHOLD`, then `dphi := 0`), the released active path preserves the current triangle sample for that tick.
- Composition meaning: phase projection -> discontinuity guard -> I256 delta path -> integrator state `z` -> triangle egress sample, under the retained default release gain routing `DpwGain::new(1 << 63, 0, 0, 0)`.
- Retained evidence:
  - Tier-1 Kani proof `proof_triangle_freeze_egress_invariant` in `kani_evidence.txt`
  - deterministic runtime test `cargo_test_triangle_control_surface.txt`

This is a bounded composition claim for the freeze branch only. It is not a proof of complete triangle-waveform correctness across all phases or frequencies.

## Proof Coverage

Tier-1 release-blocking Kani evidence retained for `1.4.0` covers:

- panic/overflow safety for the core arithmetic surfaces already in the canonical manifest
- fixed-to-`u32` phase projection semantics for the released phase map
- bounded sine egress arithmetic safety and quantizer semantics
- local I256 oracle alignment for `sub`, `sar`, and `clamp`
- triangle freeze arithmetic identity and triangle freeze egress identity
- replay-core wire-layout proofs retained by the canonical runner

Heavy proof disposition for `1.4.0`:

- Not retained as PASS evidence for this release.
- Tier-2 surfaces remain optional and skipped for release gating because they exceed the current release-budget target for routine local verification.
- Skipped heavy proofs do not contribute to the `1.4.0` release claim.
- The skipped set is bounded to the existing Tier-2 manifest entries in `verify_kani.sh`: `proof_i256_mul_u32_matches_spec` and `proof_atan2_q1` through `proof_atan2_q4`.

## Limits and Non-Goals

`1.4.0` does not claim:

- global waveform equivalence for all phases, frequencies, gains, or shapes
- full proof composition from every local harness to every end-to-end system behavior
- heavy Tier-2 proof completion as part of the retained release gate
- new CLI, artifact-format, or release-surface capability

`1.4.0` still relies on empirical evidence for:

- the bounded sine residual claim above
- runtime tests that connect local arithmetic facts to the released signal path

Anything outside the stated domains remains either local-only proof coverage, determinism evidence, or empirical observation rather than a stronger global correctness claim.
