# Δ-02 Gain Mantissa Invariance

## Claim
- Quick/full validate artifacts use a mechanically enforced singleton gain mantissa in validate-path generation.
- Scenario variation is exponent-only; quick-mode mantissa does not drift across scenarios.
- Runtime library gain guards were not changed by Δ-02 closure.

## Mechanism
- Enforced in `crates/dpw4/src/bin/precision.rs` via `GAIN_M4_Q63_QUICK` and validate-path helper construction.
- Gate coverage anchored by CI running `cargo test --locked -p dpw4 --features cli --bin precision`.
- Contract wording aligned in `docs/MATH_CONTRACT.md` OQ/Δ mapping.

## Evidence Commands
```bash
cargo test --locked -p dpw4 --features cli --bin precision
cargo run --locked --release -p dpw4 --features cli --bin precision -- validate --mode quick
```

## Evidence Output
- Bin-local tests pass, including quick mantissa invariance checks.
- Quick validate passes without hash mismatch.

## Artifacts/Hashes
- No deterministic artifact hash drift introduced by Δ-02 closure.
- Canonical hashes remain in repo baseline artifacts and git history.

## Status
- PASS (Δ-02 close-out; validate-path mechanical enforcement).
