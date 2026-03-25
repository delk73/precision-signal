# Δ-BD Build Determinism

## Claim
- Release build reproducibility is hardened with pinned release profile and CI dual-build canary.
- Same-machine/toolchain reproducibility is enforced; cross-platform bit identity is not claimed.
- Determinism evidence is process-level and independent of DSP semantic changes.

## Mechanism
- `Cargo.toml` release profile pinning (`codegen-units=1`, `lto=thin`, `panic=abort`, `incremental=false`, `debug=0`, `overflow-checks=false`, `strip=symbols`).
- CI release build + hash step + `verify_release_repro.sh` dual-build binary comparison.
- Contract text captured in `docs/MATH_CONTRACT.md` determinism section.

## Evidence Commands
```bash
cargo build --release -p dpw4 --features cli --locked
bash verify_release_repro.sh
sha256sum target/release/precision
```

## Evidence Output
- Dual-build canary passes on same machine/toolchain.
- Release artifact hash emitted in CI with toolchain metadata.

## Artifacts/Hashes
- Release binary hash records are maintained in CI logs and git-tracked evidence updates.

## Status
- PASS (Δ-BD enforced in CI/process gates).
