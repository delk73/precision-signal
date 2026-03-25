# P5 Artifact Boundary

## Claim
- P5A compile-surface checks are authoritative STOP gates.
- P5B AST hygiene enforces float-surface policy with WARN outside core and CORE-LEAK STOP in core paths.
- P5B exits nonzero iff `core_leak > 0`.
- Parent-module gating across files is not accepted; in-file local/file-level gates are recognized.

## P5A — Compile-Surface Boundary
- Commands in CI:
  - `cargo check -p dpw4 --no-default-features --target thumbv7em-none-eabihf --locked`
  - `cargo check --workspace --no-default-features --locked`

## P5B — AST Hygiene
- Command in CI:
  - `cargo run --locked -p audit-float-boundary -- --mode phase5b`
- AST detector scope:
  - type usage (`f32`, `f64`)
  - path usage (`core::f64::...`, `std::f64::...`, `f64::...`)
  - float literal suffixes (`...f32`, `...f64`)

## Evidence Commands
```bash
cargo check -p dpw4 --no-default-features --target thumbv7em-none-eabihf --locked
cargo check --workspace --no-default-features --locked
cargo run --locked -p audit-float-boundary -- --mode phase5b
```

## Evidence Output
- P5A: both checks PASS.
- P5B: deterministic summary includes
  - `total_hits`, `allow_bin`, `allow_test`, `allow_float_ingest`, `warn_other`, `core_leak`.
- CI fails only when `core_leak > 0`.

## Artifacts/Hashes
- P5 is policy/enforcement evidence; artifact hashes remain in release baseline and CI evidence logs.

## Status
- PASS (P5A authoritative; P5B enforced with CORE-LEAK STOP).
