# System Surfaces

This document explains repository surfaces and build targets.
Exact operator-surface classification lives in [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md).
Normative verification governance remains in [VERIFICATION_GUIDE.md](../../VERIFICATION_GUIDE.md).
This file is descriptive only and does not define release classification.

## Repository Surfaces

### Core Logic

- `crates/geom-signal`
- `crates/geom-spatial`
- `crates/dpw4/src`

These surfaces hold deterministic fixed-point math and reference DSP logic.
`geom-signal` underpins the canonical gate. `geom-spatial` is support logic
outside that gate.

### Hardware Harness

- `crates/dpw4/examples/rpi_verify_logic.rs`

This surface is Raspberry Pi specific and is used for physical-time observation,
not core arithmetic definition.

### Current Implementation Path

- Python replay/parser tooling under [docs/replay/](../replay/)
- STM32F446 firmware capture workflow for the active RPL0 format version 1 path

These surfaces describe the current implementation path.
Release classification is defined only in [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md).

The active STM32 self-stimulus capture contract is
[docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md](../replay/INTERVAL_CAPTURE_CONTRACT_v1.md).
The historical retained board-capture note remains
[docs/replay/FW_F446_CAPTURE_v1.md](../replay/FW_F446_CAPTURE_v1.md). The
active workspace/package version is `1.4.0`, and the latest retained patch
release evidence currently present in-tree lives under
[docs/verification/releases/1.4.0/](../verification/releases/1.4.0/). The
older hardware-backed retained release record remains explicit under
[docs/verification/releases/1.2.2/](../verification/releases/1.2.2/).

### Present In Workspace But Not Promoted By The Canonical Release Classifier

- `replay-core`, `replay-host`, `replay-embed`, and `replay-cli` (present in
  the workspace; `replay-cli` is a placeholder library and not a currently
  exposed CLI surface. Current Rust replay path support is RPL0 format
  version 1 container parsing plus legacy 16-byte `EventFrame0` replay
  interpretation)
- [docs/wip/](../wip/)
- demo-only mutation substrates not used by the released replay baseline

## Reference Build Details

- toolchain is pinned by `rust-toolchain.toml` (`rustc 1.91.1`)
- release CLI build: `cargo build --release -p dpw4 --features cli`
- supported reference targets: `x86_64` and `aarch64` little-endian
- egress format is `S32LE`

## Workspace Routing

- [docs/architecture/workspace.md](workspace.md): high-level workspace framing
- [docs/replay/README.md](../replay/README.md): replay subsystem scope and document index
- [docs/verification/build_reproducibility.md](../verification/build_reproducibility.md): reproducibility and release checks
