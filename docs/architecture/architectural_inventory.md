# Architectural Inventory (Post-2.0)

## Objective and Method

This document inventories the current repository architecture after the 2.0 release.
It is descriptive only.

It records:
- what exists
- how responsibilities are distributed
- where seams appear in current implementation and authority documents

It does not propose refactors, crate splits, repository splits, or roadmap actions.

Primary evidence was taken from:
- workspace manifests and crate entrypoints
- CLI entrypoints and command routers
- replay, witness, authority, and release-bundle tooling
- architecture and release/verification documentation

## Workspace-Level Surfaces

Observed top-level implementation surfaces:
- Rust workspace crates under [crates](../../crates)
- Python replay/evidence tooling under [scripts](../../scripts)
- retained release evidence and authority records under [docs/verification/releases](../verification/releases)
- replay and format contracts under [docs/replay](../replay) and [docs/spec](../spec)
- authority routing under [docs/authority](../authority)
- generated/runtime evidence under [artifacts](../../artifacts)

## Component Classification

Legend:
- Yes: primary or direct responsibility in that domain
- Partial: supporting or bounded responsibility in that domain
- No: not an observed responsibility in current implementation

| Component | Replay | Evidence | Trace | Experimental | Notes |
| --- | --- | --- | --- | --- | --- |
| crates/precision-cli (crate) | Yes | Yes | Yes | No | Publishes authoritative result/trace/meta artifacts; command surface for record/replay/diff/envelope. |
| precision binary (crates/precision-cli/src/bin/precision.rs) | Yes | Yes | Yes | No | Operator-facing replay validation entrypoint with fixed command set. |
| sig-util binary (crates/precision-cli/src/bin/sig_util.rs) | Partial | Yes | Partial | Partial | Validation and artifact utility surface, explicitly marked non-authoritative in help text. |
| substrate_probe binary (crates/precision-cli/src/bin/substrate_probe.rs) | No | Yes | No | Partial | Probe artifact publication path for support/audit flows. |
| crates/precision-math | Partial | Partial | Partial | No | Deterministic DPW/math kernel used by capture/replay trace generation and validation paths. |
| crates/geom-signal | No | No | Partial | Partial | Fixed-point signal math primitives; no direct replay orchestration. |
| crates/geom-spatial | No | No | Partial | Yes | Spatial fixed-point support crate; repository docs describe it as support logic outside canonical gate. |
| crates/replay-core | Yes | No | Yes | Yes | RPL constants and frame/header encoding primitives; crate itself marked experimental in manifest. |
| crates/replay-host (crate) | Yes | Partial | Yes | Yes | Parser/diff/replay-hash engine and interval-csv import path; release docs classify broad Rust replay as experimental except bounded historical diff slice. |
| replay-host binary (crates/replay-host/src/main.rs) | Yes | Partial | Yes | Yes | CLI for diff and interval-csv validation/import workflows. |
| crates/replay-fw-f446 | Yes | Yes | Yes | Partial | STM32 capture firmware producing RPL0 v1 artifacts and witness-observer feature paths. |
| crates/replay-fw-f446-timing | Partial | Yes | Yes | Yes | Timing characterization fixture emitting interval CSV evidence; documented as support/reference, not active release capture path. |
| crates/replay-embed | Partial | No | Partial | Yes | Minimal embedded replay scaffolding around replay-core types. |
| crates/replay-cli | No | No | No | Yes | Placeholder library crate; no operational CLI surface in current tree. |
| crates/audit-float-boundary | No | Yes | No | Partial | Static analysis utility for float-boundary policy in Rust sources. |
| crates/xtask (usb/release/workflow orchestration) | Partial | Yes | Partial | Partial | Orchestrates USB workflows, release checks, parser/replay workflow checks, and evidence pipeline tasks. |
| scripts/artifact_tool.py and script family | Yes | Yes | Yes | Partial | Capture/verify/hash/compare/inspect workflows for RPL artifacts; release docs classify as historical support/reference tooling. |
| scripts/artifact_diff.py | Yes | Partial | Yes | Partial | Divergence localization and shape classification on parsed artifacts. |
| scripts/inspect_artifact.py and parser tests | Yes | Yes | Yes | Partial | Structural RPL parser, strict/hash-mode behavior, adversarial and mutation corpus checks. |
| scripts/rpl0_witness.py + scripts/check_replay_witness.py | Partial | Yes | Yes | Partial | Independent witness digest path and retained-release witness verification workflow. |
| scripts/check_release_bundle.py | No | Yes | Partial | No | Validates retained release bundle coherence, required files, hashes, and authority-chain checks. |
| docs/authority/cli_contract.md | Partial | Yes | Partial | No | Normative contract for authoritative CLI behavior and artifact publication contract. |
| docs/spec/rpl0_format_contract.md | Yes | Yes | Yes | No | Normative RPL container structure, parser dispatch, and deterministic hash rules. |
| docs/replay/FW_F446_CAPTURE_v1.md | Yes | Yes | Yes | No | Normative firmware capture contract for active STM32 RPL0 path. |
| docs/replay/RPL0_WITNESS_v1.md | Partial | Yes | Yes | Partial | Defines witness scope and limits; explicitly support-evidence by default. |
| docs/verification/releases/* (retained records) | No | Yes | Partial | No | Versioned retained evidence bundles and release proof routing. |
| artifacts/* (runtime and retained data trees) | Partial | Yes | Yes | Partial | Contains published CLI artifacts, replay runs, retained demo/support evidence, and hardware characterization outputs. |

## Dependency Direction Observations

| Major component | Primary dependencies | Primary consumers | Observed architectural role |
| --- | --- | --- | --- |
| precision binary | precision-cli common/precision modules, precision-math, clap/serde/sha2 | Operators, make targets, release validation commands | Authoritative replay-validation command surface and artifact publisher. |
| sig-util binary | precision-cli common/sig_util modules, precision-math header/checksum utilities | Operators, gate/support scripts | Utility and validation command surface adjacent to authority path. |
| precision-cli common module | std fs/io, getrandom, result block/staging implementation | precision, sig-util, substrate_probe binaries | Shared publication and exit-code contract enforcement layer. |
| precision-math | geom-signal, optional sha2 for runtime hash features | precision-cli and tests/bench/proof harnesses | Deterministic DSP/reference kernel used by capture/replay-oriented trace synthesis. |
| geom-signal | fixed crate | precision-math, geom-spatial | Core fixed-point signal primitives. |
| geom-spatial | geom-signal, fixed crate | precision-math dev/tests and support workflows | Deterministic spatial support math surface. |
| replay-core | no_std core artifact constants/encoders | replay-host, replay-embed, replay-fw-f446 | Shared trace container primitive layer for replay/capture codepaths. |
| replay-host crate/bin | replay-core, sha2, serde | replay-host CLI users, scripts and tests around replay diff/import | Host parser/replay hash/diff implementation for RPL artifacts. |
| replay-fw-f446 | replay-core, cortex-m ecosystem, STM32 PAC | UART capture tooling, retained release bundles, witness check input | Firmware producer of RPL0 capture artifacts and timing/witness feature evidence. |
| replay-fw-f446-timing | cortex-m ecosystem, STM32 PAC | interval CSV ingestion paths and timing characterization evidence workflows | Firmware timing characterization producer separate from active release capture path. |
| xtask release/workflow modules | std process orchestration, docs/firmware/usb modules, Python/Make commands | CI/local operators invoking cargo xtask | Operational orchestration layer connecting verification/release commands. |
| artifact_tool.py family | inspect_artifact, compare_artifact, read_artifact, lock_baseline modules | fw-gate and replay validation scripts, operator support workflows | Python replay/evidence utility layer for capture, parse, verify, hash, compare. |
| rpl0_witness + check_replay_witness | Python stdlib + witness module | retained release checks (make replay-witness-check) | Independent witness evidence recomputation and retained digest consistency gate. |
| check_release_bundle.py | filesystem/json/hash checks over release directories | make release-bundle-check, release proof workflows | Retained-release bundle structural and hash coherence validator. |
| docs authority/spec/contracts | cross-linked contract docs | CLI implementations, scripts, release evaluations | Normative boundary definitions for CLI, format, and capture semantics. |

## Documentation of Subsystem Responsibilities

Observed responsibility-defining documents:
- [docs/system_architecture_disclosure.md](../system_architecture_disclosure.md): descriptive architecture framing and replay-tooling boundary routing
- [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md): active/supported/experimental release-surface classification
- [docs/VERIFICATION_GUIDE.md](../VERIFICATION_GUIDE.md): verification route and gate authority
- [docs/authority/cli_contract.md](../authority/cli_contract.md): authoritative CLI and published artifact contract
- [docs/spec/rpl0_format_contract.md](../spec/rpl0_format_contract.md): normative RPL0 container contract
- [docs/replay/FW_F446_CAPTURE_v1.md](../replay/FW_F446_CAPTURE_v1.md): active firmware capture contract
- [docs/replay/tooling.md](../replay/tooling.md): replay tooling boundary between active and support/reference paths
- [docs/replay/RPL0_WITNESS_v1.md](../replay/RPL0_WITNESS_v1.md): witness scope and limitation boundary
- [docs/verification/releases/index.md](../verification/releases/index.md): retained release evidence routing and bundle checks
- [docs/architecture/repository_mapping.md](repository_mapping.md): implementation-level mapping of RPL, firmware, replay engine, and CLI surfaces

## Potential Architectural Boundaries

These are observed seams in the current repository. They are documentation of existing boundaries only.

### 1. Authority CLI Surface vs Support CLI/Script Surface

- Side A:
  - precision binary and its contract path
  - docs/authority/cli_contract.md
- Side B:
  - sig-util, substrate_probe, artifact_tool.py family, replay-host auxiliary commands
- Why this seam appears:
  - explicit authoritative command contract for precision
  - explicit non-authoritative/support language for adjacent tooling
  - different publication and release-classification treatment in release documents

### 2. Replay Container/Parser Core vs Firmware Capture Producer

- Side A:
  - replay-core and replay-host parser/replay-hash modules
  - docs/spec/rpl0_format_contract.md
- Side B:
  - replay-fw-f446 firmware emitter and model-specific metadata paths
  - docs/replay/FW_F446_CAPTURE_v1.md
- Why this seam appears:
  - parser/format validation concerns are host-side and contract-driven
  - artifact production concerns are embedded runtime and hardware-capture driven
  - both sides connect through fixed RPL header/frame contracts

### 3. Replay Engine vs Evidence/Witness Validation

- Side A:
  - replay-host diff/hash path and artifact_diff.py replay divergence analysis
- Side B:
  - rpl0_witness.py/check_replay_witness.py and release-bundle validation scripts
- Why this seam appears:
  - witness path is explicitly independent support evidence
  - release-bundle checks validate retained evidence integrity rather than replay equivalence itself
  - separation reduces self-reference between primary replay logic and evidence attestation

### 4. Active Release Path vs Experimental Replay Path

- Side A:
  - precision CLI release path, firmware capture contract, retained release records
- Side B:
  - replay-core/replay-host broader capabilities, replay-embed, replay-cli placeholder, timing fixture extras
- Why this seam appears:
  - release-surface documentation classifies bounded active paths separately from experimental/support capabilities
  - manifests and docs label several replay crates as scaffolding/experimental

### 5. Deterministic Trace Core vs Geometry/Exploratory Math Surfaces

- Side A:
  - precision-math and geom-signal in replay-validation flow
- Side B:
  - geom-spatial and float-boundary auditing utilities
- Why this seam appears:
  - core replay trace paths depend on deterministic fixed-point signal kernel
  - spatial and audit tooling are present as support/exploration and policy-enforcement surfaces with weaker direct coupling to authoritative replay CLI behavior

### 6. Runtime Artifact Trees vs Retained Release Evidence Trees

- Side A:
  - artifacts/<run_id> and replay run directories generated by command execution
- Side B:
  - docs/verification/releases/<version>/ retained release bundles
- Why this seam appears:
  - runtime publication is operational output
  - retained release bundles are curated authority evidence with additional coherence checks and indexing

## Observed Distribution Summary

Across current implementation:
- Replay and Trace concerns are tightly coupled in container parsing, frame processing, and divergence reporting paths.
- Evidence concerns are distributed across CLI publication, retained release records, witness scripts, and bundle validators.
- Experimental concerns are concentrated in replay scaffolding crates, timing/support workflows, and non-authoritative tooling surfaces.
- Geometry-related components are mostly orthogonal support surfaces, with deterministic math reuse but limited direct replay-orchestration ownership.
