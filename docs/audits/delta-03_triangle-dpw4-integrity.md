# Δ-03 Triangle DPW4 Integrity

## Claim
- Triangle DPW4 integrity and calibration rationale are documented and mechanically defended.
- Control-surface freeze invariant is formalized and verified for discontinuity conditions.
- I256 byte-spec and saturation semantics are hardened for deterministic behavior.

## Mechanism
- Documentation closure in `docs/MATH_CONTRACT.md` (triangle calibration, accumulator semantics, Δ mappings).
- Runtime/test enforcement in triangle control-surface tests and Kani harnesses.
- Tiered verification in `verify_kani.sh` (Tier-1 normative, Tier-2 heavy optional).

## Evidence Commands
```bash
cargo test --locked -p dpw4 --test triangle_control_surface
cargo test --locked -p dpw4 --features cli --bin precision
./verify_kani.sh
```

## Evidence Output
- Triangle control-surface invariant tests pass.
- Validate path and forensic checks remain stable.
- Tier-1 proof set passes with hardened I256 micro-harness coverage.

## Artifacts/Hashes
- No new hash drift attributed to documentation-only Δ-03 closure items.
- Any normative baseline deltas are captured by validate/forensic artifacts in git history.

## Status
- PASS (Δ-03 documentation + integrity hardening closed).
