# DPW4 Math Contract — v1.2.1
**Status:** LOCKED normative contract.
**Revision:** Revision 2 (frozen at lock cut)
**Lock Status:** LOCKED

---

## §1 Phase Domain & Wrap

**NORMATIVE**: Phase type `Scalar` (`I64F64`), unit radians. Range `[0, 2π)` via single-branch wrap (`tick_phase` `crates/dpw4/src/lib.rs:393–403`). Inclusive lower, exclusive upper. All u32 arithmetic: wrapping. Valid shaping input: `[-2^30, 2^30]`.

**Conversion Semantics**: `(p / TWO_PI * 2^32).to_num::<u32>()` — fixed→u32 conversion per `fixed` 1.30.0 (Tier-1 proven). Required: deterministic, target-invariant for all `p ∈ [0, 2^32)`. `phase == TWO_PI` cannot occur post-wrap; negative phase impossible post-add-on-underflow. Bipolar: `(phase_u32 as i64).wrapping_sub(1 << 31)` → `i64 ∈ [-2^31, 2^31−1]`.

**Refs**: `crates/dpw4/src/lib.rs:393–403,444–449,577–584,620–627`; `crates/dpw4/src/constants.rs:14,18`; `docs/spec/reference_invariants.md §1`; `docs/spec/oscillator_api.md §Phase`.

**Divergences**: `docs/spec/pulse_implementation_spec.md §1` reconciled to radians `[0, 2π)` domain with `duty * TWO_PI`.

**Verification**:
`to_num::<u32>()` fixed→u32 conversion and overflow bounds are Tier-1 proven:
- `proof_phase_u32_no_overflow`
- `proof_phase_u32_fixed_to_u32_conversion`
See `crates/dpw4/src/verification.rs`; enforced in `verify_kani.sh`.

---

## §2 Gain Model & Headroom

**NORMATIVE**: Gain uses mantissa `m4_q63: u64` (Q63) and exponent `e4: i32`. Two-path: if `|raw| < 2^64` → `prod = raw * m` Q187, shift `= 187 − 16 − exp`; else `prod = (raw >> 64) * m` Q123, shift `= 123 − 16 − exp`. Saturating mul/shift (`crates/dpw4/src/lib.rs:223–264`). Headroom: `res >> HEADROOM_BITS(1)` — truncation. Final: `saturate_i128_to_i32`. `HEADROOM_BITS` compile-time constant; change requires hash regen. Inverse fields not normative.

### Gain Domains (v1.2.1, Normative)

- **DPW Domain** (`shape=0,1,2,3`; Saw/Pulse/Triangle family): egress routes through `apply_gain(raw, m4_q63, e4)` then shared headroom/saturation.
- **Sine Domain** (`shape=4`): egress uses calibrated `SINE_EGRESS_SCALE` (Q31) then `>> HEADROOM_BITS` then `saturate_i128_to_i32`.
- `DpwGain.{m4_q63, e4}` has no effect on Sine in v1.2.1 by design.

**Refs**: `crates/dpw4/src/lib.rs:285–313`; `crates/dpw4/src/constants.rs:54–66`; `docs/spec/dpw_gain.md §Invariants`; `VERIFICATION_GUIDE.md §5.3`.

**Divergences**: [docs/spec/reference_invariants.md](spec/reference_invariants.md) §POST: "Fixed −3 dBFS default"; code sets −3 dBFS only for `generate` subcommand; forensic artifacts use per-scenario exponents.

**Gain Invariance**: Quick-mode mantissa invariance is mechanically enforced in the validate artifact path.
`precision validate --mode quick` dispatches `run_validate -> run_determinism_check -> generate_forensic_artifacts -> generate_artifact` (`crates/dpw4/src/bin/precision.rs:304->467`, `724->738`, `1195->1202`, `1201->1253`).
In that path, `quick_validate_gain_for_scenario` constructs gain with `DpwGain::new(GAIN_M4_Q63_QUICK, scenario.gain_exponent + 15, 0, 0)` (`crates/dpw4/src/bin/precision.rs:1201-1203`), and `generate_artifact` consumes that helper (`crates/dpw4/src/bin/precision.rs:1311`), so arg1 mantissa is fixed singleton `{GAIN_M4_Q63_QUICK}` while only arg2 exponent varies by scenario.
Unit test `quick_validate_gain_mantissa_is_singleton` locks this invariant by reusing the same helper (`crates/dpw4/src/bin/precision.rs:1989-1992`).
Non-validate paths remain unconstrained (e.g., `run_generate` caller-derived mantissa at `crates/dpw4/src/bin/precision.rs:1580`).

---

## §3 Egress Mapping

**NORMATIVE**: Output `i32` S32LE. All shapes via `saturate_i128_to_i32`. Saturation `[i32::MIN, i32::MAX]`, both reachable. Rounding: truncation for right-shifts; Sine fixed-point quantizer uses signed integer-part extraction. SHA-256 over `sample.to_le_bytes()` only. Sine path uses calibrated `SINE_EGRESS_SCALE` (Q31): `(s * SINE_EGRESS_SCALE).to_num::<i32>() as i128 >> HEADROOM_BITS → saturate_i128_to_i32`; does not use `apply_gain`.

**Refs**: `crates/dpw4/src/lib.rs:473–474`; `crates/dpw4/src/lib.rs:212–220`; `crates/dpw4/src/bin/precision.rs:1352–1355,1457`; `VERIFICATION_GUIDE.md §5.1`.

**Divergences**: `docs/spec/dpw_gain.md §Descriptive` says "Triangle bypass apply_gain"; code: Triangle routes through `apply_gain` (`crates/dpw4/src/lib.rs:699`); only Sine bypasses.

---

## §4 DPW Saw (4th-Order)

**NORMATIVE**: `x2_Q124 = ((s_q31 >> 1) as i128)^2 << 64`; `>> 1` normative, immutable. `x4_Q124 = (x2 >> 62)^2`. Differentiator: `wrapping_sub` on `i128` × 3. State `Dpw4State{z1,z2,z3: i128}` zero-init `#[repr(C)]`. Output: `i128` Q124 → `apply_gain` → `i32`.

**Refs**: `crates/dpw4/src/lib.rs:139–189,344–353`; `docs/spec/reference_invariants.md §1`; `VERIFICATION_GUIDE.md §5`. **No divergences.**

---

## §5 Pulse (Two-Saw Differential)

**NORMATIVE**: `Pulse = (SawA − SawB) >> 1` in Q124; `wrapping_sub`, shift truncates. Independent `Dpw4State` per saw. Duty offset: `phase − duty * TWO_PI`, wrapped. Square = `tick_pulse(duty=0.5)`. Single `apply_gain` on mixed output.

**Refs**: `crates/dpw4/src/lib.rs:573–601`; `docs/spec/pulse_implementation_spec.md §1–4`; `docs/spec/reference_invariants.md §4`.

**Divergences**: `docs/spec/pulse_implementation_spec.md §5` reconciled: `reset()` clears internal DPW/Triangle state only; phase sync is caller-responsibility.

---

## §6 Triangle (DPW4 Integration)

**NORMATIVE**: Mechanical overflow prevention via widened I256 arithmetic. Pipeline: (1) `raw_square_diff = I256::from_i128(raw_a).sub(I256::from_i128(raw_b))` (mod 2^256); (2) `shifted = raw_square_diff.sar(DPW_TRUNCATION_BITS)` (I256 arithmetic right-shift); (3) `delta_wide = shifted.mul_u32(dphi)` (I256 × u32, mod 2^256); (4) `delta_i128 = delta_wide.clamp_to_i128()` (single clamp); (5) `z = z.saturating_add(delta_i128)` (defined rail behavior). `dphi > 0x4000_0000` → freeze. First tick returns `0`. Output: `apply_gain(z, m4_q63, e4)`. DPW1 (`tick_triangle_dpw1`) forensic-only.

**Triangle Q/Units Table**:

| Quantity | Type | Q-Format | Notes |
|---|---|---|---|
| `raw_square_diff` | `I256` | Q124 | `I256::from_i128(raw_a).sub(I256::from_i128(raw_b))` — widened mod 2^256 |
| `shifted` | `I256` | Q92 | `raw_square_diff.sar(DPW_TRUNCATION_BITS)` — I256 arithmetic right-shift |
| `dphi` | `u32` | Unitless step (2^32-turn space); Q32 cycles | Wrapping phase delta |
| `delta_wide` | `I256` | Q124 | `shifted.mul_u32(dphi)` — widened mod 2^256 |
| `delta_i128` | `i128` | Q124 | `delta_wide.clamp_to_i128()` — single clamp; saturates to i128::MIN/MAX |
| `z` accumulator | `i128` | Q124 | `z.saturating_add(delta_i128)` — accumulation of Q124 deltas; defined rail |

### Triangle e4 Calibration

For v1.2.1 DPW-based shapes, egress gain is applied with the same call form:
- Saw: `apply_gain(raw4, gain.m4_q63, gain.e4)`
- Pulse: `apply_gain(raw_mix, gain.m4_q63, gain.e4)`, with `raw_diff = raw_a - raw_b`; `raw_mix = raw_diff >> 1`
- Triangle: `apply_gain(state.tri.z, gain.m4_q63, gain.e4)`

Derived egress scaling form (common gain stage):

`effective_scale = m4_q63 * 2^(-exp) * 2^(-HEADROOM_BITS)`, with `exp = gain.e4`.

Shape-specific pre-gain raw-domain factors in v1.2.1:
- Saw: no additional pre-gain shape factor in `tick_dpw4` (`raw4` enters `apply_gain` directly).
- Pulse: fixed extra `2^-1` from `raw_mix = raw_diff >> 1` before `apply_gain`.
- Triangle: delta path applies `sar(DPW_TRUNCATION_BITS)` before `mul_u32(dphi)` and accumulation into `z`; no additional post-integrator output shift before `apply_gain`.

Calibration rationale enforced by code:
- Triangle does not use a distinct `e4` constant; it uses `gain.e4`, identical interface to Saw/Pulse.
- Therefore Triangle/Saw/Pulse share the same gain exponent control and the same headroom policy at egress.
- Code enforces a common bounded egress contract (shared `apply_gain` + `HEADROOM_BITS` + saturation), not cross-shape peak/RMS/energy equality.

### §6.1 Control-Surface Invariant

**NORMATIVE** (single-tick):

Freeze condition: `dphi > DISCONTINUITY_THRESHOLD` (strict `>`).
- `dphi: u32 = phase_u32.wrapping_sub(prev_phase_u32_old)` — wrapping distance (`prev_phase_u32_old` is the value of `state.tri.prev_phase_u32` before tick updates it)
- `DISCONTINUITY_THRESHOLD: u32` — semantic: 1/4 cycle in u32 space; current value `0x4000_0000` is policy, not invariant identity
- `dphi == DISCONTINUITY_THRESHOLD` does **not** freeze

When freeze condition holds, for this tick:
- `delta_i128 := 0` (I256 multiply-by-zero is exact)
- `z_next := z_prev` (`saturating_add(0)` is identity)
- No `clamp_to_i128` side-effect this tick

**Kani Proof**: `proof_triangle_freeze_invariant` (Tier-1) — post-guard arithmetic identity; assumes guard has set `dphi=0`; proves arithmetic consequences only; does not prove guard firing condition.

**Refs**: `tick_triangle_dpw4` in `crates/dpw4/src/lib.rs`; `I256` in `crates/dpw4/src/i256.rs`; `crates/dpw4/src/constants.rs` (`DPW_TRUNCATION_BITS`); Kani harnesses in `crates/dpw4/src/verification.rs` and `crates/dpw4/src/i256.rs`.

**I256 Kani Verification Status**:

| Op | Tier | Status | Harness |
|---|---|---|---|
| `I256::sub` | Tier 1 | ✅ proven vs. byte-level oracle | `proof_i256_sub_matches_spec` |
| `I256::sar` (shift < 256) | Tier 1 | ✅ proven vs. byte-level oracle | `proof_i256_sar_in_range_matches_spec` |
| `I256::sar` (shift ≥ 256) | Tier 1 | ✅ proven vs. byte-level oracle | `proof_i256_sar_out_of_range_matches_spec` |
| `I256::clamp_to_i128` | Tier 1 | ✅ proven vs. byte-level oracle | `proof_i256_clamp_matches_spec` |
| `spec_clamp` in-range contract | Tier 1 | ✅ byte-level spec self-consistency | `proof_spec_clamp_in_range_contract` |
| `spec_clamp` out-of-range contract | Tier 1 | ✅ byte-level spec self-consistency | `proof_spec_clamp_out_of_range_contract` |
| `spec_sar` sanity (known patterns) | Tier 1 | ✅ micro-harness vs. expected bit patterns | `proof_spec_sar_sanity` |
| `I256::mul_u32` | Tier 2 | ⏸ optional — spec correct, proof exceeds CI budget | `proof_i256_mul_u32_matches_spec` (run with `RUN_HEAVY=1`) |

> **Constraint**: "I256 `sub`/`sar`/`clamp` are proven against an independent byte-level oracle; `spec_clamp`/`spec_sar` byte-level specs are self-verified as Tier-1." NOT "I256 arithmetic proven" — `mul_u32` equivalence is tier-2/optional.

**Divergences**: `docs/spec/reference_invariants.md §4` describes Triangle as naive bitwise folding; normative shape=2 is DPW4 integration. Naive form (`TriangleDPW1`) is forensic-only.

**Calibration & Safety**: Triangle `e4` calibration rationale relative to Saw/Pulse is specified in §6 "Triangle e4 Calibration". No code change required.
No uncontrolled i128 wrap; entire pipeline executes in I256 (mod 2^256), delta clamped once via `clamp_to_i128`, z updated via `saturating_add`. Formally verified: `I256::sub`, `I256::sar`, `I256::clamp_to_i128` proven against independent byte-level oracle (tier1); `I256::mul_u32` oracle-specified, proof tier2. Kani harnesses: `proof_triangle_delta_clamp_identity_when_in_range`, `proof_triangle_delta_clamp_saturates_when_out_of_range`, `proof_triangle_z_update_is_saturating`.

---

## §7 CORDIC Sin/Cos

**NORMATIVE**: Shape=4 via `math::sin_cos_fast(phase)` (`geom-signal`). Input: `Scalar` radians `[0, 2π)`. Output `s: Scalar` (`I64F64`) `[-1.0, 1.0]`. Cosine discarded.

**Sine Quantization Closure**:
- `s` type: **`Scalar` (`I64F64`)** — grounded at `crates/dpw4/src/lib.rs:465`.
- Scaling multiply `s * SINE_EGRESS_SCALE`: **fixed-point** (Scalar×Scalar, I64F64 arithmetic), not float.
- `to_num::<i32>()` rounding: **signed conversion** as specified by `fixed` 1.30.0; mechanically verified as equivalent to bit-level integer-part extraction.
- Observable outcome: headroom `>> HEADROOM_BITS` applied; final saturated via `saturate_i128_to_i32`. At `s = 1.0`, Sine max output is bounded by `SINE_EGRESS_SCALE_Q31 >> HEADROOM_BITS`.
- No `apply_gain`; `DpwGain.{m4_q63, e4}` are ignored for Sine.

**Refs**: `crates/dpw4/src/lib.rs:464–475,549–555`; `VERIFICATION_GUIDE.md §5.4`.

**Divergences**: See §7 Gain Semantics subsection below. Behavior unchanged; gain fields remain ineffectual for Sine; now explicitly normative.

**Quantizer Semantics**:
`.to_num::<i32>()` on the Sine egress path follows the `fixed` 1.30.0 signed conversion rule (toolchain pinned). Within the proven domain `|s * SINE_EGRESS_SCALE_Q31| ≤ SINE_EGRESS_SCALE_Q31`, conversion is equivalent to signed integer-part extraction via right shift of the I64F64 representation:
```
let hi: i64 = (scaled.to_bits() >> 64) as i64;
to_num::<i32>() == hi as i32
```
This equivalence is:
- Proven deterministically over 4097 representative phase-derived values (`test_sine_scaling_bit_extraction_equivalence`)
- Mechanically verified by Tier-1 Kani harness `proof_sine_to_i32_in_range`
- Confirmed to preserve sign symmetry and boundedness

The quantizer semantics are therefore fully pinned for v1.2.1.

`sin_cos_fast` output satisfies `|s| ≤ 1.0` over the deterministic 4097-phase sweep (`test_sine_scaling_safety_sweep`).
Given this bound:
```
|s * SINE_EGRESS_SCALE_Q31| ≤ SINE_EGRESS_SCALE_Q31 < 2^31
```
I64F64 has 64 integer bits; therefore no overflow is possible in scaling or conversion.
Overflow absence is mechanically verified by Tier-1 harness `proof_sine_scale_no_overflow`.

### Gain Semantics

**NORMATIVE** (v1.2.1):

- Sine (`shape=4`) does **not** call `apply_gain`.
- `DpwGain.{m4_q63, e4}` are **ignored** in v1.2.1; non-zero values have no effect on Sine amplitude.
- Exact egress pipeline:
  1. `sin_cos_fast` — CORDIC output `s: Scalar` (`I64F64`) ∈ `[-1.0, 1.0]`
  2. `s * SINE_EGRESS_SCALE` — fixed-point scalar scaling
  3. `.to_num::<i32>()` — signed conversion
  4. cast to `i128`
  5. `>> HEADROOM_BITS` — arithmetic right-shift (truncation)
  6. `saturate_i128_to_i32` — saturation to `[i32::MIN, i32::MAX]`
- Sine amplitude is defined by this fixed egress pipeline (**fixed egress scaling semantics**); output level is determined solely by `HEADROOM_BITS` and the CORDIC output range.
- Uniform gain semantics are not defined in v1.2.1.

---

## §8 Determinism Boundaries

**NORMATIVE**: Toolchain `rustc 1.91.1` exact. 64-bit targets only. All wire serialization: explicit LE. `crates/dpw4/src/lib.rs` `#![no_std]`; `std` only under `verification-runtime`. `#![forbid(unsafe_code)]`. Rounding: truncation for right-shifts; fixed→int conversions are specified per-site; no ties-to-even. All structs `#[repr(C)]`; no unsafe transmute in normative path. Feature flags `audit` and `cli` must not change `i32` sample values.

**Normative Dependencies**:
- `rustc`: **1.91.1** — `rust-toolchain.toml`.
- `fixed` crate: **1.30.0** — `Cargo.lock:342`, checksum `c566da967934c6c7ee0458a9773de9b2a685bd2ce26a3b28ddfc740e640182f5`. Normative for `to_num` semantics and I64F64 arithmetic.
- `geom-signal`: **1.2.1**, local workspace crate via path dependency (`{ path = "../geom-signal" }`).

**Build Determinism Contract**: Release builds are pinned with `codegen-units=1`, `lto="thin"` (frozen), `panic=abort`, `incremental=false`, `debug=0`, `overflow-checks=false`, `strip="symbols"`. These fields are frozen workspace-wide in `[profile.release]`; no per-crate overrides. Bit-for-bit identity is enforced on same-machine, same-toolchain builds via dual-build hash comparison (`verify_release_repro.sh`). Cross-platform or cross-linker bit identity is **not** claimed.

**Refs**: `crates/dpw4/src/lib.rs:1–2`; `crates/dpw4/src/bin/precision.rs:1,167`; `crates/dpw4/src/constants.rs:66`; `crates/dpw4/src/verification.rs:210–267`; `VERIFICATION_GUIDE.md §2`.

**Divergences**: `VERIFICATION_GUIDE.md §7.1` + `docs/spec/header_layout_addendum.md §Field Map` show `pad[36]`; code has `HEADER_PAD_SIZE = 32` (`pad[32]`) + `reserved[4]` separate; guide omits `reserved` field entirely.

**Geom-Signal Hermeticity**:
`geom-signal` is a workspace path dependency (`{ path = "../geom-signal" }`).
Hermeticity is enforced via:
- `cargo build --locked`
- pinned `rustc 1.91.1`
- workspace VCS revision control
No external git-source pin is required.

---

## §9 Global Q-Format Table

| Quantity | Type | Q-Format | Range |
|---|---|---|---|
| `phase` | `I64F64` | Radians (Q64.64) | `[0, 2π)` |
| `phase_u32` | `u32` | Unitless | `[0, 2^32)` |
| bipolar `s_q31` | `i64` | Q31 | `[-2^31, 2^31−1]` |
| `x2_q124` / `x4_q124` | `i128` | Q124 | DPW polynomial intermediates |
| `tick_dpw4_raw` output | `i128` | Q124 | Pre-gain |
| Pulse `raw_mix` | `i128` | Q124 | `(A−B) >> 1` |
| Triangle `raw_square_diff` | `i128` | Q124 | Diff of two DPW4 chains |
| Triangle `>> 32` | `i128` | Q92 | Truncated |
| Triangle `dphi` | `u32` | Unitless step | Wrapping delta |
| Triangle `z` | `i128` | Q124 | Accumulator of Q124 deltas; passed to apply_gain |
| `apply_gain` prod (HP) | `i128` | Q187 | Before shift |
| `apply_gain` prod (HA) | `i128` | Q123 | After `>>64` pre-shift |
| Sine `s` | `I64F64` | Fixed `[-1, 1]` | CORDIC output |
| Sine `s * SINE_EGRESS_SCALE` | `Scalar` | Fixed `[-SINE_EGRESS_SCALE_Q31, SINE_EGRESS_SCALE_Q31]` | Pre-headroom calibrated scale |
| Final output (all shapes) | `i32` | S32LE | `[−2^31, 2^31−1]` |

---

## §10 Global Rounding / Saturation Table

| Site | Operation | Mode | Grounded? |
|---|---|---|---|
| `phase_u32` = `(p/TWO_PI*2^32).to_num::<u32>()` | Fixed→u32 | Fixed→u32 conversion (Tier-1 proven) | Kani-proven: `proof_phase_u32_fixed_to_u32_conversion` |
| `s_q31 >> 1` | Arith right-shift | Truncate | `crates/dpw4/src/lib.rs:142` |
| `x2 >> 62` | Arith right-shift | Truncate | `crates/dpw4/src/lib.rs:161` |
| Pulse `raw_diff >> 1` | Arith right-shift | Truncate | `crates/dpw4/src/lib.rs:597` |
| Triangle `raw >> 32` | Arith right-shift | Truncate | `crates/dpw4/src/lib.rs:673` |
| `saturating_shift_i128` right | Arith right-shift | Truncate | `crates/dpw4/src/lib.rs:246` |
| `saturating_shift_i128` left-overflow | `checked_shl` | Saturate → i128::MIN/MAX | `crates/dpw4/src/lib.rs:251–260` |
| `saturating_mul_i128` overflow | `checked_mul` | Saturate → i128::MIN/MAX | `crates/dpw4/src/lib.rs:224–233` |
| `res_i128 >> HEADROOM_BITS` | Arith right-shift | Truncate | `crates/dpw4/src/lib.rs:312` |
| `saturate_i128_to_i32` | `clamp` | Saturate | `crates/dpw4/src/lib.rs:218–220` |
| Sine `s * SINE_EGRESS_SCALE` | Scalar fixed multiply | Panic-free under \|s\|≤1 | Egress arith given `sin_cos_fast` contract \|s\|≤1 |
| Sine `(…).to_num::<i32>()` | Fixed→i32 | Signed conversion | Egress arith given `sin_cos_fast` contract \|s\|≤1 |
| Sine `pre_headroom >> HEADROOM_BITS` | Arith right-shift | Truncate | `crates/dpw4/src/lib.rs:474` |

---

## §11 Conformance Definition

Conformance is defined by byte-identical `S32LE` output under the pinned release configuration. Implementations must match the reference SHA-256 hashes for all normative test scenarios. Logic is derived from the fixed-point kernels and egress policies defined in this contract.

---

**Status**: v1.2.1 (LOCKED)
```
