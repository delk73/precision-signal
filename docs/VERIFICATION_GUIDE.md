# Verification Guide

**Version: 1.8.0 (Active Release Baseline)**
**Status: Core Verification Authority**

## Purpose

This guide is the newcomer-safe verification router and compact core authority
for `precision-signal`.

Use it to choose the right verification path first. Detailed retained-release
mechanics live in [docs/verification/releases/index.md](verification/releases/index.md).
This guide may name release commands as routing signposts, but detailed release
command order, retained-bundle inventories, and per-version release procedure
belong in the retained-release index and per-version release records.

## Quick Path Selection

| Intent | Run / Read | Do Not Do Yet |
| --- | --- | --- |
| Validate local host surface | `make gate` | Kani, release bundle work |
| Bring up STM32 board | `make bench-check SERIAL=<device>`, then `make fw-gate SERIAL=<device>` | Kani, release authoring |
| Inspect retained release | `docs/verification/releases/<version>/` and [docs/verification/releases/index.md](verification/releases/index.md) | New capture unless revalidating |
| Prepare retained release | [docs/verification/releases/index.md](verification/releases/index.md) | Ad hoc board/debug flow |

## Core Verification Model

`precision-signal` verification is split by evidence type:

- **Host deterministic validation**: `make gate` is the routine local validation
  entrypoint. It checks the pinned toolchain and hash-locked deterministic signal
  outputs through the `sig-util validate --mode quick` path.
- **Replay artifact authority**: RPL0 artifact structure and replay behavior are
  governed by [docs/spec/rpl0_format_contract.md](spec/rpl0_format_contract.md)
  and the replay contracts under [docs/replay/](replay/).
- **STM32 RPL0 firmware evidence**: `make fw-gate SERIAL=<device>` validates the
  active attached-board capture path for the `replay-fw-f446` firmware contract.
- **Retained release evidence**: release claims are supported by retained files
  under `docs/verification/releases/<version>/`. The retained-release index owns
  the mechanics for preparing and checking those bundles.
- **Kani proof role**: Kani is retained release/proof-boundary evidence. It is
  not a board-bringup step.

Normative behavior still comes from this guide plus the domain contracts it
routes to, especially [docs/MATH_CONTRACT.md](MATH_CONTRACT.md),
[docs/spec/rpl0_format_contract.md](spec/rpl0_format_contract.md), and
[docs/replay/FW_F446_CAPTURE_v1.md](replay/FW_F446_CAPTURE_v1.md).

## Path A: Local Host Validation

Run:

```text
make gate
```

This is the routine local validation path. It is the canonical operator-facing
host gate for the active release baseline.

Expected result:

- command exits `0`
- toolchain pin and workspace version checks pass
- normative deterministic hashes match the current in-code table

Use `make gate-full` only as supplementary validation. It does not replace
`make gate`.

## Path B: First STM32 Board Bring-Up

Run:

```text
make bench-check SERIAL=<serial-device>
make fw-gate SERIAL=<serial-device>
```

For a typical ST-LINK VCP target, `<serial-device>` is usually `/dev/ttyACM0`.

Active board bring-up uses ST-LINK reset by default. The active path must not
ask the operator to press reset manually. If the active board-bringup path
requires manual reset, treat that as a workflow defect.

Kani is not part of board bring-up. Board bring-up proves the attached STM32
capture path can be flashed, read, verified, compared against the retained
baseline, and repeated through the active firmware workflow.

The active firmware contract is
[docs/replay/FW_F446_CAPTURE_v1.md](replay/FW_F446_CAPTURE_v1.md).

## Path C: Retained Release Inspection

Read:

```text
docs/verification/releases/<version>/
docs/verification/releases/index.md
```

At summary level, a retained release record may include:

- retained host gate evidence
- retained firmware evidence for firmware-including releases
- retained Kani evidence where required by that release record
- bundle-check transcript and generated summary files when present

Do not start a new capture just to inspect a retained release. Inspect the
retained files first, then rerun only the checks needed for an explicit
revalidation task.

## Path D: Retained Release Preparation

Start at:

```text
docs/verification/releases/index.md
```

That index owns retained-release mechanics, including release evidence
locations, release-proof routing, bundle checks, retained Kani evidence
requirements, firmware-including release evidence, and historical retained
record policy.

This guide intentionally does not duplicate the detailed release procedure.

## Kani Boundary

Kani is release/proof-boundary evidence:

- Kani is not a board-bringup step.
- Kani is not part of routine local host validation unless explicitly requested.
- If a release path requires retained Kani evidence, produce or verify that
  evidence as part of release-record preparation.
- Do not infer that every release command directly runs Kani. Follow
  [docs/verification/releases/index.md](verification/releases/index.md) and the
  current release tooling behavior.

The canonical runner remains:

```text
bash scripts/verify_kani.sh
```

Optional tiers are explicitly opt-in:

```text
RUN_TIER2=1 bash scripts/verify_kani.sh
RUN_TIER3=1 bash scripts/verify_kani.sh
```

The runner manifest in `scripts/verify_kani.sh` defines the proof surface for
retained formal-verification evidence. Source harnesses omitted from that
manifest are implementation inventory, not retained proof evidence.

## Supporting Checks

Use these only when they match the task:

- `make bench-check SERIAL=<device>`: attached bench preflight before firmware
  validation.
- `make replay-tests`: parser, replay, and artifact-tool regression checks.
- `make doc-link-check`: documentation link integrity.
- `bash scripts/verify_release_repro.sh`: same-machine dual-build identity
  evidence when retained release preparation calls for it.
- `make release-bundle-check VERSION=<version>`: retained bundle coherence check;
  release mechanics live in [docs/verification/releases/index.md](verification/releases/index.md).

## Core Authority Details

The pinned environment is part of verification:

- `rustc 1.91.1`, enforced by `rust-toolchain.toml`
- 64-bit host word-size class for host validation
- `I64F64` fixed-point `Scalar` as the core precision bedrock
- 32-bit signed little-endian egress for hash-locked signal outputs

Core float quarantine is build-surface based. It is satisfied when:

```text
cargo check --workspace --no-default-features
cargo check -p dpw4 --no-default-features --target thumbv7em-none-eabihf
```

both succeed without enabling `float-ingest`. Enforcement is not grep-based;
test-only code and CLI support code are outside the core DSP surface.

For deterministic host validation, the active normative `.det.csv` hashes live
in code, not in this guide. Inspect the current `NORMATIVE_DET_HASHES`
definition in `crates/dpw4/src/bin/precision.rs` when reviewing the active
hash table.

## Red Flags

Treat these as verification defects:

- replay self-diff presented as independent replay equivalence
- unexpected first divergence in replay comparison
- active board-bringup or release path requiring manual reset
- Kani presented as a board-bringup step
- missing retained evidence for a release claim
- non-firmware release proof used for firmware-including release claims
- deterministic hash mismatch in `make gate`
- floating-point math entering the core DSP tick path without an explicit
  `float-ingest` boundary
- retained release evidence copied forward from an older release instead of
  generated or explicitly retained for the current release

## Authority Routing

- Core verification router and compact authority: this guide.
- Release-surface classification:
  [docs/RELEASE_SURFACE.md](RELEASE_SURFACE.md).
- Detailed retained-release mechanics:
  [docs/verification/releases/index.md](verification/releases/index.md).
- Per-version retained release records:
  `docs/verification/releases/<version>/`.
- Active STM32 RPL0 firmware contract:
  [docs/replay/FW_F446_CAPTURE_v1.md](replay/FW_F446_CAPTURE_v1.md).

Historical retained evidence remains valid as historical evidence. It does not
change the active path selected from this guide.
