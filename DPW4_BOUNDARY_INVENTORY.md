DPW4 CURRENT:
- crates/dpw4/src/lib.rs - no-std deterministic oscillator and reference signal-processing API
- crates/dpw4/src/constants.rs - deterministic math and artifact/header constants
- crates/dpw4/src/i256.rs - no-std 256-bit helper for triangle integration and Kani harnesses
- crates/dpw4/src/header.rs - artifact origin header identity layout
- crates/dpw4/src/checksum.rs - Fletcher-32 and optional stream/payload hashing helpers
- crates/dpw4/src/goldens.rs - locked reference output hashes
- crates/dpw4/src/verification.rs - optional verification helpers and header verifier
- crates/dpw4/src/bin/common - shared CLI result, staging, audit, and argument helpers
- crates/dpw4/src/bin/precision* - public precision operator CLI
- crates/dpw4/src/bin/sig_util* - support CLI for generation, validation, inspection, and verification
- crates/dpw4/src/bin/substrate_probe.rs - audit/probe CLI wrapper
- crates/dpw4/build.rs - CLI build metadata generator
- crates/dpw4/tests - mixed math/reference tests and CLI surface tests
- crates/dpw4/examples - math/reference examples and hardware-oriented probes

MOVE:
- src/lib.rs - precision-math - no-std deterministic oscillator/reference API
- src/constants.rs - precision-math - no-std constants used by the reference API
- src/i256.rs - precision-math - deterministic helper used by the reference API and proofs
- src/header.rs - precision-math - no-std artifact identity layout
- src/checksum.rs - precision-math - reference-safe checksum and optional named hash feature support
- src/goldens.rs - precision-math - reference hashes belong with the deterministic math surface
- src/verification.rs - precision-math - verification helper surface is tied to the reference library
- math/reference tests - precision-math - exercise the reference library API
- math/reference examples - precision-math - exercise the reference library API
- src/bin/common - precision-cli - CLI-only result block, staging, audit, and argument utilities
- src/bin/precision* - precision-cli - public precision operator CLI
- src/bin/sig_util* - precision-cli - CLI-only generation, validation, inspection, and reporting wrappers
- src/bin/substrate_probe.rs - precision-cli - CLI-only audit/probe wrapper
- build.rs - precision-cli - generates CLI audit/version metadata only
- CLI surface tests - precision-cli - require CLI binaries and CARGO_BIN_EXE_* environment

RETAIN/DEFER:
- crates/replay-core - retained; existing replay-domain crate remains owner of replay artifact code unless a mechanical import update is required
- crates/replay-host - retained; no consolidation into precision-cli
- crates/replay-cli - retained; no consolidation unless a future inspection proves duplicate boundary ownership
- crates/replay-embed - retained; no dependency churn required by this split
- crates/replay-fw-f446 - retained; firmware behavior and target support unchanged
- crates/replay-fw-f446-timing - retained; timing firmware behavior unchanged
- precision-core - deferred; no clearly shared non-math domain type currently requires a new crate

DOCUMENTATION BLAST RADIUS:
- Update active current-state references that describe the crate boundary, validation commands, scripts, Makefile variables, CI/check paths, or package-name-sensitive checks.
- Do not rewrite historical evidence logs, archived release evidence, retained artifacts, old provenance records, or release outputs merely because they mention dpw4.
- Remaining dpw4 references after the split must be limited to this inventory, historical evidence/release/provenance material, or comments explicitly describing legacy history.
