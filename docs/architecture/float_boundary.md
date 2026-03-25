# Float Boundary

This document is descriptive. The normative arithmetic contract lives in
[docs/MATH_CONTRACT.md](../MATH_CONTRACT.md).

## Boundary

Default core crate builds are float-free.

Floating-point is permitted only in:

- `float-ingest` feature adapters
- non-core binaries such as the `dpw4` CLI
- `#[cfg(test)]` code

Floating-point must not influence:

- canonical DSP state evolution
- fixed-point arithmetic paths
- deterministic hash artifacts

This is a governance and audit boundary, not a performance claim.

## Build-Surface Enforcement

Quarantine is evaluated at the build surface rather than by token scanning.

Reference checks:

```bash
cargo check --workspace --no-default-features
cargo check -p dpw4 --no-default-features --target thumbv7em-none-eabihf
```

These checks must succeed without enabling `float-ingest`.
