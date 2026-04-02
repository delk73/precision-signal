# Reference Invariants Specification
**Document revision:** v1.0.0-rc5  
**Applies to:** release 1.3.1 (content unchanged)

## Versioning Terminology

- Document revision labels editorial history for this specification.
- Release versions identify the shipped software release.
- This document defines mathematical and operational invariants; unchanged content remains applicable to release `1.3.1`.

This document defines the authoritative mathematical and operational invariants for `dpw4`. This is a **Canonical Reference Baseline**.

## Governance & Interpretation

To prevent interpretation drift, all signals produced by this system are classified as either Normative or Advisory.

| Signal | Authority | Role | Failure Consequence |
| --- | --- | --- | --- |
| **SHA-256 Hashes** | **Normative** | The absolute definition of correctness. | **Immediate Rejection** |
| **Bit-Identical Phase** | **Normative** | Shaping-domain truncation locked to `(s_q31 >> 1)` prior to DPW differentiation. | **Non-Conformance** |
| **Integer Model** | **Normative** | Two's Complement bit-exact. | **Non-Conformance** |
| **Egress Width** | **Normative** | 32-bit Signed Integer (S32LE). | **Non-Conformance** |

## 1. Signal Core (Fixed-Point)

| Parameter | Type | Scaling / Format | Invariant |
| :--- | :--- | :--- | :--- |
| **Phase State** | `Scalar` | I64F64 radians | Canonical oscillator phase representation; reduced modulo 2π. |
| **Shaping Phase Projection** | `i32` | signed Q31 bipolar | Bounded phase projection derived from `Scalar` for DPW differentiation. |
| **Polynomial Products** | `i128` | Q124 signed | Intermediates ($x^2, x^4$) use Q124 `i128` math to prevent truncation. |
| **Gain Mantissa** | `u64` | Q63 (0.63) | Unit range $[0.5, 1.0)$. |
| **Egress Sample** | `i32` | S32 Little Endian | Saturated projection of the i128 core. |

### Phase Representation Clarification (Normative)

The oscillator phase state is maintained in `Scalar` (`I64F64`) radians.
For DPW polynomial evaluation and differentiation, this phase is projected into a bounded
Q31 bipolar domain. No `u64` phase accumulator is used in the current
reference implementation.

Earlier descriptions of a `u64` phase accumulator are historical and
descriptive only, and must not be interpreted as a required storage
format.

### Architectural Standard:
* **Fixed 4th-order path**: Absolute spectral consistency across all frequencies.
* **Monomorphized Kernel**: Zero-branching `SignalShape` trait implementation to eliminate runtime dispatch and support LLVM auto-vectorization.

## 2. Transport Protocol: DP32

The system communicates via the **DP32 Protocol**. All signal streams must be preceded by a valid `SignalFrameHeader`.

### Header Structure (64-byte aligned):
* **Magic**: `b"DP32"`
* **Sequence**: `u64` monotonic sequence.
* **Sample Rate**: `u32` observed rate.
* **Bit Depth**: `u32` (Fixed at 32).
* **Padding**: Reserved bytes; total header size is exactly 64 bytes via alignment.

## 3. RAW vs POST Contract

### RAW Output (Reference Standard)
* **Definition:** The authoritative projection of the analytic signal.
* **Composition:** Identity summation of analytic signal paths.
* **Clamping:** Mandatory saturation to 32-bit range `[-2147483648, 2147483647]`.
* **Integrity:** No DC removal or perceptual conditioning is applied in the reference logic.

### POST Output (Listening Policy)
* **Headroom**: Fixed $-3.0$ dBFS default for reference monitoring.
* **DC Centering**: Leaky integration is advisory for listener comfort but forbidden in the forensic reference stream.
* **Reset Invariant**: State buffers (`Dpw4State`) must be zeroed on shape or parameter changes to prevent cross-contamination.

## 4. Operation Semantics
* **Pulse/Square Scaling**: The differential pulse output MUST be scaled by 0.5 prior to gain application to maintain Sawtooth-aligned headroom.
* **TriangleDPW4 (Normative, `shape=2`)**: The normative triangle algorithm is `tick_triangle_dpw4`, a DPW4 integration pipeline.
  Invariant-boundary description of the pipeline:
  1. Two DPW4 sawtooth chains produce a differential band-limited square (`raw_a − raw_b` via `I256::sub`).
  2. The 256-bit difference is narrowed by `DPW_TRUNCATION_BITS` via right shift (`I256::sar`).
  3. The shifted value is scaled by the current phase increment `dphi` (`I256::mul_u32`).
  4. The result is clamped to i128 (`I256::clamp_to_i128`).
  5. The triangle integrator state is advanced via `i128::saturating_add`.
  6. Final egress is through `apply_gain(state.tri.z, gain.m4_q63, gain.e4)` → `i32`.

  A large-step freeze guard (`dphi > DISCONTINUITY_THRESHOLD ⇒ dphi = 0`) suppresses pump artifacts from
  externally driven phase discontinuities; when the guard fires, `delta_i128 == 0` and the integrator state is
  unchanged for that tick.

* **TriangleDPW1 (Forensic-only)**: The naive bitwise-folding path (`tick_triangle_dpw1`, `TriangleDPW1` type) is
  retained **for forensic comparison only** (noise-floor baselining and regression detection). It is **not
  band-limited** and is **not the normative algorithm**. Its output range is asymmetric
  `[-2147483648, 2147483646]` (peaks at `i32::MAX - 1`). `TriangleDPW1` does **not** call `apply_gain` and does
  not participate in the normative gain model.

* **Hardware Baseline**: This crate baselines for 64-bit architectures (ARM64, x86_64).
