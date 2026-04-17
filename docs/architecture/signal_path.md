# Signal Path Overview

This document is descriptive. Arithmetic and narrowing rules are normative only
in [docs/MATH_CONTRACT.md](../MATH_CONTRACT.md).
Precision Signal is a deterministic execution validation system centered on
replay, operated through the `precision` CLI against an attached STM32 target
over UART.

Signals are generated with fixed-point phase accumulation and deterministic
kernel evaluation. Core signal-path arithmetic is intended to remain free of
floating-point operations.
Within project direction, this signal path is the arithmetic substrate behind
replay artifacts inspected through the `precision` CLI and captured from the
canonical attached STM32-over-UART path.

```text
Phase (Scalar = I64F64)
  -> Shape kernel (DPW4 polynomial or CORDIC sine)
  -> Gain domain
      - DPW shapes (shape=0,1,2,3) -> apply_gain
      - Sine (shape=4) -> SINE_EGRESS_SCALE
  -> Headroom shift
  -> Saturation
  -> S32LE egress
```

## Crate Roles In The Path

- `geom-signal`: fixed-point math bedrock, including `Scalar`, algebraic helpers,
  and CORDIC kernels
- `geom-spatial`: spatial math on `Scalar` for host-side operations
- `dpw4`: reference oscillator and shape-selection surfaces that feed the
  replay-oriented operator path

All narrowing rules, gain constants, saturation semantics, and egress policy are
defined in [docs/MATH_CONTRACT.md](../MATH_CONTRACT.md).
