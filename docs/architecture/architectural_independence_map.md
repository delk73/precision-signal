# Architectural Independence Map (Post-2.0)

## Objective and Framing

This document maps architectural independence for the repository as implemented today.

Focus:
- foundational components
- dependent components
- orchestration components
- authority components
- which surfaces remain coherent in isolation

This is descriptive only.
It does not propose refactors, extraction, roadmap actions, or repository splits.

Project direction context used for interpretation:
Precision Signal is a deterministic execution validation system centered on replay, operated through the precision CLI against an attached STM32 target over UART.

## Independence Table

| Component | Can Exist Alone | Depends On | Enables | Why |
| --- | --- | --- | --- | --- |
| crates/geom-signal | Yes | fixed-point numeric model and deterministic math assumptions | deterministic signal primitives used by math and trace paths | Coherent as an isolated fixed-point signal math library. |
| crates/geom-spatial | Mostly | geom-signal scalar model and deterministic fixed-point conventions | deterministic spatial representation and distance/magnitude transforms | Useful alone, but strongest meaning comes from shared scalar conventions with signal/math layers. |
| crates/precision-math | Mostly | geom-signal primitives, deterministic DPW contracts, validation assumptions | deterministic trace-supporting math, oscillator/state evolution, checksum helpers | Coherent library, but present responsibilities are tied to repository verification and replay flows. |
| crates/replay-core | Mostly | RPL frame/header representation contract | replay container encoding constants used by firmware and host replay surfaces | Semantically narrow but coherent as a reusable replay artifact representation core. |
| crates/replay-host (library) | Mostly | replay artifact structure, replay frame semantics, parser invariants | host replay diff, replay hash stream generation, interval capture import | Can stand alone as replay parser/diff engine if artifact contract is carried with it. |
| replay-host binary | Mostly | replay-host library and replay artifact corpus expectations | command-line replay diff and interval csv import/validation | Operationally coherent, but current value is bounded by documented support/experimental scope. |
| crates/replay-fw-f446 | No | STM32 target profile, replay-core format definitions, UART capture workflow contracts | active hardware replay capture artifact production | Without board, contract, and capture flow context it loses most practical coherence. |
| crates/replay-fw-f446-timing | Mostly | STM32 timing fixture assumptions and interval capture contract | timing characterization csv evidence generation | Coherent as a dedicated timing fixture, but purpose is support to larger validation system. |
| crates/replay-embed | Mostly | replay-core frame/header types | embedded replay scaffolding | Minimal but coherent scaffold; utility is primarily as a support layer. |
| crates/replay-cli | No | broader replay implementation not present in this crate | experimental namespace placeholder | Placeholder surface without independent capability. |
| crates/precision-cli (crate) | No | authority CLI contract, artifact publication contract, math and trace expectations, filesystem artifact conventions | operator replay commands, result block emission, artifact publication | Strongly repository-coupled to authority and artifact conventions. |
| precision binary | No | precision-cli command contract and publication semantics | authoritative replay execution/validation command surface | Exists to coordinate repository-specific replay and artifact authority behavior. |
| sig-util binary | Mostly | precision-math helpers, artifact parsing/verification expectations | support validation utilities and deterministic checks | Can run in isolation, but semantics rely on repository contracts and test corpus. |
| substrate_probe binary | Mostly | artifact staging/publishing conventions | probe run artifact publication for support checks | Small coherent utility, but meaningful outputs depend on repository artifact conventions. |
| crates/audit-float-boundary | Yes | Rust source tree and float-boundary policy definitions | policy evidence for float boundary compliance | Coherent static-audit utility independent of replay runtime. |
| crates/xtask release/workflow/usb orchestration | No | make targets, scripts, board descriptors, release/verification policy routes | coordinated workflow execution across firmware, docs, and verification paths | Primarily orchestration glue across repository systems. |
| scripts/artifact_tool.py family | Mostly | replay artifact format/parser behavior, capture device path, baseline conventions | capture, verify, hash, compare, inspect replay artifacts | Coherent tool family, but deeply tied to local contracts and artifacts corpus. |
| scripts/artifact_diff.py | Mostly | parsed replay artifacts and divergence semantics rules | divergence localization and evolution classification | Independent script utility with narrow but coherent scope. |
| scripts/inspect_artifact.py and parser tests | Yes | RPL representation contract | trace/container interpretation and parser validity evidence | Coherent parser/validator surface independently useful for artifact correctness checks. |
| scripts/rpl0_witness.py + scripts/check_replay_witness.py | Mostly | witness report contract, retained release artifacts | witness digest generation and retained witness verification | Coherent evidence tooling, but practical use depends on retained release structure. |
| scripts/check_release_bundle.py | Mostly | retained release directory conventions, summary/hash schema | release bundle legitimacy and coherence verification | Independent checker with strong coupling to local release bundle schema. |
| scripts/fw_gate.py | No | make workflow, capture scripts, ST-LINK tooling, release gate expectations | hardware-backed gate orchestration and evidence run coordination | Pure orchestration layer for existing systems; little standalone semantic value. |
| docs/authority/cli_contract.md | Mostly | repository command and artifact publication implementation | normative CLI behavior and publication legitimacy criteria | Coherent authority spec on its own, but authored for this repository implementation. |
| docs/spec/rpl0_format_contract.md | Yes | RPL design assumptions and deterministic hashing rules | normative trace/container contract for parser and replay systems | Strongly standalone as a format contract document. |
| docs/replay/FW_F446_CAPTURE_v1.md | Mostly | board-specific capture implementation and RPL format contract | normative firmware capture behavior for active replay path | Coherent contract, but tightly anchored to specific hardware path. |
| docs/replay/RPL0_WITNESS_v1.md | Mostly | witness implementation and retained evidence usage | witness scope boundaries and report semantics | Coherent scope contract but oriented to this repository workflow. |
| docs/replay/tooling.md | Mostly | release classification and replay tooling inventory | boundary routing between active, support, and experimental replay tooling | Coherent as taxonomy/routing document for current repository. |
| docs/RELEASE_SURFACE.md | Mostly | release policy context and versioned retained records | release classification authority and operator surface boundaries | Coherent governance artifact, but tied to repository release record structure. |
| docs/VERIFICATION_GUIDE.md | Mostly | gate targets, release routing, contract docs | verification authority routing and operator evaluation path | Coherent as process authority, but implementation-specific in command references. |
| docs/verification/releases/* retained records | No | release version directories, summary/hash artifacts, authority chain | proof retention, release legitimacy history, evidence continuity | Inherently relational archive; loses meaning outside release chain context. |
| artifacts/* runtime and retained trees | No | capture/replay producers, authority and verification interpretation rules | runtime evidence, retained artifact corpus, replay and witness inputs | Raw artifacts are not self-explanatory without contracts and validation systems. |

## Foundation Components

Components that appear foundational due to relatively low outgoing dependency load and high enabling effect:

- crates/geom-signal
Reason it appears foundational: deterministic scalar and signal operations are reused by higher-level math and trace-supporting paths.

- crates/precision-math
Reason it appears foundational: provides deterministic state evolution and signal-generation machinery used by replay-adjacent command and validation behavior.

- crates/replay-core
Reason it appears foundational: central replay artifact representation primitives are consumed by firmware and host replay systems.

- scripts/inspect_artifact.py and parser tests
Reason it appears foundational: parser truth surface underpins artifact structural interpretation used by replay/evidence workflows.

- docs/spec/rpl0_format_contract.md
Reason it appears foundational: normative container semantics anchor replay parsing, hashing, and format-validity interpretation.

## Orchestration Components

Components that primarily coordinate systems rather than define core semantics:

- precision binary
Systems coordinated: command parsing, mode handling, artifact publication, result emission routing.

- crates/xtask release/workflow/usb modules
Systems coordinated: docs checks, firmware workflows, parser/replay test workflows, release procedures.

- scripts/fw_gate.py
Systems coordinated: build, flash, capture, verify, compare, repeat-capture gate path.

- scripts/artifact_tool.py family
Systems coordinated: capture input, parser invocation, hashing, comparison, and baseline interactions.

- docs/VERIFICATION_GUIDE.md
Systems coordinated: operator validation routes across software gate, bench gate, and retained release routing.

## Authority Components

Components that establish normative behavior, legitimacy, or evidence requirements:

- docs/authority/cli_contract.md
Authority exercised: canonical command grammar, result block semantics, artifact publication and exit-code contract.

- docs/spec/rpl0_format_contract.md
Authority exercised: normative replay artifact container rules, parser dispatch, canonical hash semantics.

- docs/replay/FW_F446_CAPTURE_v1.md
Authority exercised: active STM32 capture contract for replay artifact production behavior.

- docs/RELEASE_SURFACE.md
Authority exercised: classification of active vs support vs experimental surfaces.

- docs/VERIFICATION_GUIDE.md
Authority exercised: verification routing and gate authority references.

- docs/verification/releases/* retained records
Authority exercised: retained release legitimacy record and evidence continuity for historical versions.

- scripts/check_release_bundle.py
Authority exercised: mechanical enforcement of retained bundle coherence expectations.

- scripts/check_replay_witness.py
Authority exercised: mechanical verification of retained witness consistency for release versions.

## Dependency Gravity

| Component | Incoming Gravity | Outgoing Gravity | Notes |
| --- | --- | --- | --- |
| crates/geom-signal | High | Low | Frequently reused fixed-point signal primitive layer. |
| crates/geom-spatial | Medium | Medium | Depends on signal scalar model; consumed by support math paths. |
| crates/precision-math | High | Medium | Core deterministic math/state surface consumed by CLI and validation/testing paths. |
| crates/replay-core | High | Low | Shared replay representation core consumed by host and firmware replay implementations. |
| crates/replay-host library | Medium | Medium | Depends on replay-core and parser semantics; enables replay diff behavior. |
| replay-host binary | Low | Medium | Thin command wrapper over replay-host capabilities. |
| crates/replay-fw-f446 | Medium | High | Depends on hardware/runtime and contracts; enables active capture path. |
| crates/replay-fw-f446-timing | Low | High | Hardware-bound fixture with narrower consumers. |
| crates/precision-cli / precision binary | High | High | Central operator surface depending on authority, artifact, and deterministic math/trace assumptions. |
| sig-util binary | Medium | Medium | Utility layer with moderate dependencies and moderate consumer reach. |
| crates/xtask | Medium | High | Orchestrates many systems; depends on many external commands and scripts. |
| scripts/artifact_tool.py family | Medium | High | Coordinates multiple artifact operations and parser/capture subsystems. |
| scripts/artifact_diff.py | Low | Medium | Narrow divergence consumer of parsed artifacts. |
| scripts/inspect_artifact.py and parser tests | High | Medium | Widely reused interpretation surface; depends on format contract expectations. |
| scripts/rpl0_witness + check_replay_witness | Medium | Medium | Important evidence check path with retained-release dependency. |
| scripts/check_release_bundle.py | Medium | Medium | Significant release legitimacy gate with structured retained-data dependencies. |
| scripts/fw_gate.py | Medium | High | Aggregates many command and tooling dependencies into one gate path. |
| docs/authority/cli_contract.md | High | Low | Many behaviors are interpreted against this contract; minimal document dependencies. |
| docs/spec/rpl0_format_contract.md | High | Low | Deep normative anchor for parser and replay format behavior. |
| docs/replay/FW_F446_CAPTURE_v1.md | Medium | Medium | Normative for active capture path with format/hardware coupling. |
| docs/RELEASE_SURFACE.md | High | Medium | Organizing classification authority dependent on retained-record context. |
| docs/VERIFICATION_GUIDE.md | High | Medium | High routing influence with references to many verification surfaces. |
| docs/verification/releases/* retained records | High | High | Consumed by authority checks and witness/release verification; depends on generated evidence artifacts. |
| artifacts/* runtime and retained trees | High | High | Central runtime/evidence inputs dependent on producers and interpretation contracts. |

## Removal Thought Experiments

### Remove Replay

What survives:
- authority documentation surfaces
- retained release bundles and historical evidence records
- parser-level and witness-level evidence tooling in form
- deterministic trace/math libraries as standalone computation surfaces

What becomes unusable:
- active replay execution/comparison command paths
- replay-host diff-centric workflows
- replay-centered operator flow through precision commands

What remains coherent:
- evidence governance and release verification logic still has structure, though it references reduced operational replay capability.

### Remove Evidence

What survives:
- deterministic math and trace representation components
- replay encoding/parsing execution mechanics at code level
- some host/firmware implementation surfaces

What becomes unusable:
- retained release legitimacy checks
- witness verification against retained records
- authority-based publication and release evaluation routes

What remains coherent:
- replay and trace technical machinery remain coherent as implementation artifacts, but legitimacy and verification-chain interpretation are significantly reduced.

### Remove Trace

What survives:
- orchestration and authority documents as process artifacts
- some release/evidence shell workflows and retained file structures

What becomes unusable:
- deterministic replay interpretation correctness boundaries
- parser/format semantic integrity checks
- math-grounded trace generation and transformation behavior

What remains coherent:
- evidence process scaffolding can persist structurally, but technical correctness foundations are heavily degraded.

### Remove Experimental

What survives:
- active replay path through precision and fw capture contract
- core evidence authority and retained release infrastructure
- core trace/math and parser foundations

What becomes unusable:
- placeholder/scaffolding replay surfaces
- support characterization fixtures and exploratory orchestration portions

What remains coherent:
- core replay, evidence, and trace structure remains coherent with reduced auxiliary breadth.

## Architectural Shapes

Best match based on current dependency observations: Pattern E (None of the above).

Observed structure:
- Trace appears as a deep semantic foundation for deterministic representation and interpretation.
- Replay appears as the central operational behavior surface.
- Evidence appears as the dominant organizing and legitimacy surface for release routing, retained records, and authority interpretation.

Why Pattern E:
- Pattern A under-describes authority and evidence centrality.
- Pattern B under-describes trace depth.
- Pattern C captures trace depth but under-describes evidence as organizing governance center.
- Pattern D captures evidence organization but under-describes replay operational centrality and trace semantic foundation.

Current implementation looks tri-axial rather than single-center:
- operational center: Replay
- semantic foundation: Trace
- governance/legitimacy organizer: Evidence

## Summary View

This map indicates:
- foundational layers exist in trace/math and replay representation contracts
- orchestration layers are substantial and repository-coupled
- authority layers exert strong dependency gravity across runtime and retained evidence paths
- several components are coherent alone, but the highest-value flows are cross-component and contract-coupled
- the repository does not collapse into a single simple dependency shape

