Retained historical audit note only.
This file does not define the release contract, CLI contract, or active replay
capture authority. Use [docs/verification/releases/index.md](releases/index.md)
for historical verification routing.

Δ-03
Operational Domain
Phase is externally drivable via the public API `TriangleDPW4::tick(&mut OscState, phase: Scalar, gain: &DpwGain)`.
Normative domain: Constant frequency increment (`dphi`), continuous phase progression driven by an internal or conforming oscillator where `dphi` <= `DISCONTINUITY_THRESHOLD`.
Adversarial domain: Adaptive or discontinuous arbitrary sequences of `phase` injected strictly to maximize `dphi` variance and drive monotonic combinations.

Normative Drift Result
PASS (exact I256 DC bound; integer-only; no floats; see test output).
Normative Drift Test (N=10,000,000): see test output.

Pump Result
CHARACTERIZATION. Control-surface vulnerability demonstrated. By adaptively injecting a phase discontinuity `> DISCONTINUITY_THRESHOLD` whenever the integration `delta` is negative, the integrator freezes ($d\phi = 0$). Repeating standard increments when `delta` is positive pumps `z` monotonically. Freeze invariant asserted; see test output.
Baseline Envelope for Pump Test (N=1,000,000): see test output.
Pump Test (N=1,000,000): see test output (prints max_abs_z_norm, max_abs_z_pump, ratio).

Arithmetic Safety Result
PASS (mechanical). No uncontrolled wrap in i128 arithmetic; widened arithmetic is defined modulo 2^256 before clamping.
Semantics:
* compute `diff = raw_a - raw_b` in a widened fixed-width model (mod 2^256)
* arithmetic shift right by `DPW_TRUNCATION_BITS` in that widened model
* multiply by `dphi` (u32, non-negative) in that widened model
* clamp the widened result to `i128` once (`delta_i128`)
* update: `state.tri.z = state.tri.z.saturating_add(delta_i128)`
Dimensional Coherence Result
| Quantity | Units | Q | Exact Scaling |
| -------- | ----- | - | ------------- |
| `phase_u32` / `dphi` | Cycles | Q32 | $1.0 \text{ cycle} = 2^{32}$. Integer maps $[0, 2\pi) \rightarrow [0, 2^{32})$. |
| `s_q31` | Half-Cycles | Q31 | `phase_u32 - 2^31`. $1.0 \text{ half-cycle} = 2^{31}$. |
| `x_q30` | Half-Cycles | Q30 | `s_q31 >> 1`. $1.0 \text{ peak} = 2^{30}$. |
| `x^2` | Amplitude | Q124 | $(x_{Q30} \times x_{Q30} \rightarrow Q60) \ll 64 = Q124$. |
| `x^4` | Amplitude | Q124 | $(x^2_{Q124} \gg 62 \rightarrow Q62); (x^2_{Q62} \times x^2_{Q62}) = Q124$. |
| `raw_square_diff` | Amplitude | Q124 | Linear difference function maintains Q124. |
| `shifted` | Amplitude | Q92 | $Q124 \gg 32 = Q92$. |
| `delta` | Amplitude | Q124 | $Q92 \times Q32 (\text{cycles}) = Q124$. |
| `z` | Amplitude | Q124 | Accumulation maintains Q124 natively. |

Algebraic derivation for `(raw >> 32) * dphi`:
1. `raw_square_diff` arrives in Q124: $y_{raw} = \text{val} \times 2^{124}$.
2. Exponent shift applies `>> 32`, transitioning the mantissa to Q92: $y_{shifted} = \text{val} \times 2^{92}$.
3. `dphi` is scaled such that 1.0 cycle = $2^{32}$, establishing it natively in Q32: $y_{dphi} = \Delta \phi \times 2^{32}$.
4. Power-of-two multiplication: $\text{delta} = y_{shifted} \times y_{dphi} = (\text{val} \times 2^{92}) \times (\Delta \phi \times 2^{32}) = (\text{val} \times \Delta \phi) \times 2^{92 + 32} = (\text{val} \times \Delta \phi) \times 2^{124}$.
5. Exact Q124 formatting is fully restored. The precise $Q124$ accumulator state is passed to `apply_gain`.

Artifact Impact (Yes/No)
No impact observed in normative drift run: `did_clamp_delta=false` and `did_saturate_z=false` over N=10,000,000 (see `test_normative_no_clamp_no_saturate` test output).
Operationally, the replacement of `wrapping_add` with `saturating_add` and the addition of `clamp_to_i128` constitutes a runtime semantic change if bounds are hit. However, `sig-util validate --mode quick` passes unchanged, validating that `TriangleDPW4` (which participates in the hashed validation artifacts) remains oracle-equivalent safely within its normative bounds.

Closure Status (PASS / FAIL)
PASS (normative drift passes; pump vulnerability characterized; arithmetic safety PASS).

## Manual Validation

### x86_64
```bash
cargo test -p dpw4 --release --test triangle_normative_drift -- --ignored --nocapture
cargo test -p dpw4 --release --test triangle_phase_pump -- --ignored --nocapture
cargo test -p dpw4 --release --lib test_normative_no_clamp_no_saturate -- --ignored --nocapture
```

### Raspberry Pi (ARMv8)

Same commands; record:

* runtime ms
* max_abs_z
* bound_global hi/lo, abs_sum hi/lo
* bound_window hi/lo, abs_diff hi/lo
