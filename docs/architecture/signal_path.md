# Signal Path Overview

This document is descriptive. Arithmetic and narrowing rules are normative only
in [docs/MATH_CONTRACT.md](../MATH_CONTRACT.md).

Signals are generated with fixed-point phase accumulation and deterministic
kernel evaluation. Core signal-path arithmetic is intended to remain free of
floating-point operations.

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
- `dpw4`: reference oscillator, shape selection, and CLI surfaces

All narrowing rules, gain constants, saturation semantics, and egress policy are
defined in [docs/MATH_CONTRACT.md](../MATH_CONTRACT.md).
