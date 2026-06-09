# Architectural Ownership Map (Post-2.0)

## Objective and Classification Basis

This document classifies repository components by ownership as implemented today.

Classification rule applied for Canonical Owner Domain:
- What would be most damaged if this component disappeared?

Each component has:
- exactly one Canonical Owner Domain
- zero or one Secondary Domain

Domains:
- Replay
- Evidence
- Trace
- Experimental

## Component Ownership Table

| Component | Canonical Owner Domain | Secondary Domain | Why |
| --- | --- | --- | --- |
| crates/precision-cli (crate) | Replay | Evidence | Loss primarily removes authoritative replay command behavior and replay result generation. |
| precision binary (crates/precision-cli/src/bin/precision.rs) | Replay | Evidence | Loss removes the operator replay execution and replay comparison entrypoint. |
| sig-util binary (crates/precision-cli/src/bin/sig_util.rs) | Evidence | Replay | Loss primarily removes validation and support evidence tooling rather than authoritative replay entrypoint. |
| substrate_probe binary (crates/precision-cli/src/bin/substrate_probe.rs) | Evidence | Experimental | Loss primarily removes probe artifact publication for support/audit evidence paths. |
| crates/precision-math | Trace | Replay | Loss primarily removes deterministic math/state machinery used to generate and transform trace-aligned signal output. |
| crates/geom-signal | Trace | None | Loss primarily removes fixed-point signal primitives used for deterministic representation/transformation. |
| crates/geom-spatial | Trace | Experimental | Loss primarily removes deterministic spatial state representation utilities. |
| crates/replay-core | Replay | Trace | Loss primarily removes replay container primitives required by replay producer/consumer paths. |
| crates/replay-host (crate) | Replay | Trace | Loss primarily removes host replay execution and diff behavior over artifacts. |
| replay-host binary (crates/replay-host/src/main.rs) | Replay | Experimental | Loss primarily removes replay-host command surface; release docs keep broad surface as support/experimental outside bounded slice. |
| crates/replay-fw-f446 | Replay | Evidence | Loss primarily removes active firmware replay capture producer for deterministic replay validation flow. |
| crates/replay-fw-f446-timing | Experimental | Evidence | Loss primarily removes timing characterization fixture, which is support/reference rather than active authoritative replay capture route. |
| crates/replay-embed | Experimental | Trace | Loss primarily removes exploratory embedded replay scaffolding. |
| crates/replay-cli | Experimental | None | Loss primarily removes placeholder experimental replay CLI scaffolding surface. |
| crates/audit-float-boundary | Evidence | Experimental | Loss primarily removes verification-policy evidence for float-boundary compliance in source. |
| crates/xtask (usb/release/workflow orchestration) | Evidence | Experimental | Loss primarily removes release and verification orchestration infrastructure. |
| scripts/artifact_tool.py and script family | Replay | Evidence | Loss primarily removes replay capture/verify/compare operational path used around artifact replay workflows. |
| scripts/artifact_diff.py | Replay | Trace | Loss primarily removes replay divergence localization behavior. |
| scripts/inspect_artifact.py and parser tests | Trace | Evidence | Loss primarily removes structural trace/container interpretation and parser truth surface. |
| scripts/rpl0_witness.py + scripts/check_replay_witness.py | Evidence | Trace | Loss primarily removes independent witness evidence verification and retained digest checks. |
| scripts/check_release_bundle.py | Evidence | None | Loss primarily removes retained release-bundle integrity verification. |
| scripts/fw_gate.py | Evidence | Replay | Loss primarily removes hardware-backed release evidence gate orchestration. |
| docs/authority/cli_contract.md | Evidence | Replay | Loss primarily removes authoritative evidence contract for CLI result and artifact publication semantics. |
| docs/RELEASE_SURFACE.md | Evidence | Experimental | Loss primarily removes release classification authority and support/experimental routing definitions. |
| docs/VERIFICATION_GUIDE.md | Evidence | Replay | Loss primarily removes authoritative verification routing for release/bench paths. |
| docs/spec/rpl0_format_contract.md | Trace | Replay | Loss primarily removes normative trace/container representation contract for RPL parsing/hashing boundaries. |
| docs/replay/FW_F446_CAPTURE_v1.md | Replay | Evidence | Loss primarily removes normative definition of active replay capture producer behavior. |
| docs/replay/tooling.md | Evidence | Experimental | Loss primarily removes documented boundary between retained support replay tooling and active/operator authority paths. |
| docs/replay/RPL0_WITNESS_v1.md | Evidence | Trace | Loss primarily removes witness evidence scope and report contract boundary. |
| docs/verification/releases/* retained records | Evidence | None | Loss primarily removes retained proof/evidence history and release authority record continuity. |
| artifacts/* runtime and retained trees | Evidence | Trace | Loss primarily removes preserved evidence artifacts and retained trace-bearing outputs. |

## Ownership Conflicts

#### replay-host binary (crates/replay-host/src/main.rs)
Chosen Canonical Domain: Replay

Alternative Candidate: Experimental

Reason For Tension:
Implementation behavior is replay diff/import oriented, but release classification constrains most of its command surface to support/experimental usage.

#### crates/replay-core
Chosen Canonical Domain: Replay

Alternative Candidate: Trace

Reason For Tension:
It defines container/frame constants and encoding that also behave as trace representation primitives, but direct consumers are replay producer/consumer paths.

#### crates/replay-fw-f446
Chosen Canonical Domain: Replay

Alternative Candidate: Evidence

Reason For Tension:
As implemented it is the active replay capture producer, while operationally it is also core to release-evidence generation and retention.

#### scripts/artifact_tool.py and script family
Chosen Canonical Domain: Replay

Alternative Candidate: Evidence

Reason For Tension:
Command surface performs replay-facing verify/compare/capture behavior, yet release documents classify the Python layer as retained support/reference rather than canonical operator authority.

#### scripts/inspect_artifact.py and parser tests
Chosen Canonical Domain: Trace

Alternative Candidate: Evidence

Reason For Tension:
It is a structural parser and interpretation layer for RPL representation, but in practice it is also used as evidence validation infrastructure in replay checks.

#### docs/spec/rpl0_format_contract.md
Chosen Canonical Domain: Trace

Alternative Candidate: Replay

Reason For Tension:
The contract is fundamentally representation-level, but replay behavior and replay-hash semantics rely directly on it, making replay ownership plausible.

#### docs/replay/tooling.md
Chosen Canonical Domain: Evidence

Alternative Candidate: Experimental

Reason For Tension:
The document acts as governance/routing evidence for release boundaries while simultaneously cataloging support/reference and experimental replay surfaces.

#### crates/xtask (usb/release/workflow orchestration)
Chosen Canonical Domain: Evidence

Alternative Candidate: Experimental

Reason For Tension:
It is operational infrastructure for verification and release checks, but much of the orchestration wraps support workflows and non-authoritative command paths.

## Domain Gravity

### Component Counts

- Replay-owned components: 9
- Evidence-owned components: 14
- Trace-owned components: 5
- Experimental-owned components: 3

### Gravity Findings

1. Which domain appears to own the repository center of gravity?
Evidence appears to own the center of gravity by governance/routing load and retained-release infrastructure coverage, even while replay remains the core product behavior.

2. Which domain appears largest by component count?
Evidence is largest by count.

3. Which domain appears most dependent on the others?
Evidence appears most dependent, because evidence generation and validation require replay producers/executors plus trace/container definitions.

4. Which domain appears most independent?
Trace appears most independent, with math and representation contracts that remain useful even when replay/evidence orchestration is reduced.

## What The Map Reveals Under Domain Removal

- What remains if Replay is removed?
Trace math/representation surfaces, evidence governance records, release-bundle validation infrastructure, and retained artifacts remain; active replay execution/comparison flow collapses.

- What remains if Evidence is removed?
Replay and trace implementations remain, but authority routing, retained release records, witness checks, and bundle coherence paths are largely absent.

- What remains if Trace is removed?
Evidence and some orchestration remain, but deterministic container/math semantics and parser/representation foundations are severely degraded, breaking replay correctness boundaries.

- Which components resist clean ownership assignment?
Primary tension clusters around replay-core, replay-host binary, replay-fw-f446, artifact_tool.py family, inspect_artifact parser surfaces, replay tooling boundary docs, and xtask orchestration.
