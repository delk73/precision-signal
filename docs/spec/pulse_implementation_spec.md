# Specification: Pulse & Square Implementation
**Version: 1.0.0-rc5 (Reference Lock)**

This document defines the normative implementation for relaying Pulse and Square waveforms.

## 1. Mathematical Model
Pulse synthesis is achieved using the **Differential Sawtooth** method.

$$Pulse(t, w) = \frac{Saw(t) - Saw(t - w)}{2}$$

Phase domain is radians in `[0, 2π)` (I64F64).
Duty cycle is converted via:
    duty_phase = duty * TWO_PI
and wrapped in radians.

### Square Wave Alias
$$Square(t) = Pulse(t, 0.5)$$

**Behavioral Note**: Fixed 50% duty is enforced for the `Square` shape type.

## 2. Bit-Depth Invariants
- **Phase Accumulator**: `Scalar` (`I64F64` radians, reducing modulo $2\pi$).
- **Shaping Projection**: `i32` (Bipolar Q31 for differentiation).
- **Filter State**: `i128` (Matches `Dpw4State` reference).
- **Intermediate Math**: `i128` (Q124 format to prevent polynomial truncation).
- **Egress**: `i32` (S32LE 32-bit reference width).

## 3. High-Precision Mixing
In the Reference Implementation, the subtraction $Saw(t) - Saw(t-w)$ MUST occur in the **Q124 domain** (`i128`) before any scaling or bit-reduction. This ensures that the relativistic phase accuracy is maintained at the machine limit ($10^{-16}$) before quantization to the 32-bit output.

## 4. Normalization
To ensure loudness parity with the Sawtooth waveform, the result of the differential subtraction MUST be scaled by $0.5$ (`raw >> 1`) prior to gain application.

## 5. Reset Behavior
Upon a change to `Shape` or `Duty`, the following must occur:
1.  All `Dpw4State` differentiators are zeroed via `reset()`.
2.  `reset()` clears internal DPW and Triangle state only.
3.  Phase accumulator synchronization is caller-responsibility.
4.  `OscState` does not own an external phase accumulator.
