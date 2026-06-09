# Architectural Findings Synthesis (Post-2.0)

## Purpose

This is a findings document synthesizing:
- [architectural_inventory.md](architectural_inventory.md)
- [architectural_ownership_map.md](architectural_ownership_map.md)
- [architectural_independence_map.md](architectural_independence_map.md)

Focus:
- agreement and disagreement across passes
- architectural anchors (immovable components)
- identity-crisis components (classification tension)
- descriptive boundary stress test for a consumable Replay subsystem

This document is descriptive only.
No refactor, extraction, split, or roadmap recommendations are made.

## Cross-Pass Comparison Matrix

Legend:
- Ownership: canonical owner domain from ownership map
- Independence: condensed signal from independence map (Foundation / Orchestration / Authority / Standalone status)
- Observation: synthesis note

| Component | Ownership | Independence | Observation |
| --- | --- | --- | --- |
| crates/replay-core | Replay | Foundation (Mostly standalone) | Interesting: ownership says Replay, inventory/independence show deep Trace-like foundational role. Persistent bridge signal. |
| crates/replay-host (lib/bin) | Replay | Mostly standalone replay engine; binary bounded by support scope | Interesting: operational Replay core with release-surface ambiguity for broad host commands. |
| crates/replay-fw-f446 | Replay | Not standalone in practice; hardware/workflow-coupled | Consistent but tensioned: canonically Replay, operationally also Evidence producer for release legitimacy. |
| crates/precision-cli + precision binary | Replay | Orchestration-heavy and repository-coupled | Interesting: clear replay ownership, but low isolation score due to authority/publication coupling. |
| sig-util binary | Evidence | Mostly standalone utility | Interesting: utility lives near replay CLI crate but maps to Evidence due to support/verification role. |
| crates/precision-math | Trace | Foundation (Mostly standalone) | Consistent: repeatedly appears as deep deterministic base for replay correctness. |
| crates/geom-signal | Trace | Foundation (Standalone Yes) | Consistent and strong: deepest reusable primitive layer across passes. |
| crates/geom-spatial | Trace | Mostly standalone support math | Mild tension: treated as support/experimental-adjacent in inventory notes but ownership still Trace. |
| scripts/inspect_artifact.py + parser tests | Trace | Foundation (Standalone Yes) | Interesting: parser surface is both Trace foundation and Evidence enforcement substrate. |
| scripts/artifact_tool.py family | Replay | Orchestration-heavy, mostly standalone | Identity-crisis pattern: replay-facing behavior with evidence/support classification pressure. |
| scripts/artifact_diff.py | Replay | Mostly standalone narrow tool | Consistent: replay divergence specialization with moderate coupling. |
| scripts/rpl0_witness.py + check_replay_witness.py | Evidence | Mostly standalone evidence tooling | Consistent: strong Evidence ownership with Trace dependency. |
| scripts/check_release_bundle.py | Evidence | Mostly standalone validator; authority enforcement | Consistent: retained-release legitimacy guardrail component. |
| scripts/fw_gate.py | Evidence | Orchestration (not standalone) | Consistent: gate coordinator with high outgoing dependency gravity. |
| crates/xtask | Evidence | Orchestration (not standalone) | Identity-crisis pattern: Evidence ownership but behavior is coordination-heavy infrastructure. |
| docs/authority/cli_contract.md | Evidence | Authority anchor; low outgoing gravity | Consistent: hard authority surface across all analyses. |
| docs/spec/rpl0_format_contract.md | Trace | Foundation + Authority anchor | Interesting: Trace-owned but functionally central to Replay and Evidence legitimacy. |
| docs/replay/FW_F446_CAPTURE_v1.md | Replay | Authority component, hardware-context-coupled | Tension: Replay ownership with Evidence authority effects. |
| docs/replay/RPL0_WITNESS_v1.md | Evidence | Authority component | Consistent: witness scope authority and evidence semantics center. |
| docs/RELEASE_SURFACE.md | Evidence | Authority/organization layer | Consistent: organizing classification surface that drives interpretation across components. |
| docs/VERIFICATION_GUIDE.md | Evidence | Authority + orchestration router | Consistent: process authority rather than deep technical foundation. |
| docs/verification/releases/* | Evidence | Not standalone corpus; high gravity | Consistent: legitimacy archive, structurally dependent but central to evidence chain. |
| artifacts/* | Evidence | Not standalone corpus; high gravity | Consistent: high-consumption evidence substrate requiring contracts/producers for meaning. |

## Disagreement-Focused Findings

Most consistent rows are less informative.
The informative rows are where role and independence signals diverge.

### 1) Replay-Owned But Not Independently Coherent

Components:
- crates/precision-cli + precision binary
- crates/replay-fw-f446

Finding:
- Replay ownership is clear.
- Independence score is constrained by authority contracts, publication semantics, hardware profile, and gate workflows.
- Replay behavior exists, but as part of a contract-driven system rather than isolated replay logic.

### 2) Trace-Owned Components Acting As Cross-Domain Pillars

Components:
- docs/spec/rpl0_format_contract.md
- scripts/inspect_artifact.py + parser tests

Finding:
- These are Trace-class by definition.
- They are simultaneously Evidence-critical (legitimacy checks) and Replay-critical (parser/replay semantics).
- Trace is not merely a helper layer; it is a shared semantic anchor.

### 3) Evidence-Owned Components That Behave Like Infrastructure

Components:
- crates/xtask
- scripts/fw_gate.py

Finding:
- Canonical ownership is Evidence because failure damages release legitimacy and verification flow.
- Their implementation role is orchestration/integration rather than evidence semantics definition.
- They are coordination chokepoints, not semantic authorities.

### 4) Replay vs Evidence Pressure On Support Tooling

Component:
- scripts/artifact_tool.py family

Finding:
- Ownership map assigns Replay due to capture/verify/compare replay impact.
- Inventory and release-surface framing push it toward support/reference Evidence-adjacent interpretation.
- This is a bridge between operational replay behavior and evidence workflow execution.

## Architectural Anchors (Immovable Components)

These repeatedly appear as important across all three passes regardless of method.

- crates/replay-core
Reason: common replay representation primitive consumed by firmware and host surfaces.

- crates/replay-host (library and binary)
Reason: primary replay execution/diff path and recurring replay boundary reference.

- crates/replay-fw-f446
Reason: active hardware replay capture producer and evidence input source.

- crates/precision-cli + precision binary
Reason: canonical operator surface for replay result generation and publication.

- docs/spec/rpl0_format_contract.md
Reason: deep format/trace authority used by parser, replay, and evidence checks.

- docs/authority/cli_contract.md
Reason: normative publication/CLI authority that shapes operational legitimacy.

- scripts/inspect_artifact.py + parser tests
Reason: structural interpretation truth surface reused in replay and evidence validation flows.

- scripts/rpl0_witness.py + scripts/check_replay_witness.py
Reason: independent witness evidence path and retained verification mechanism.

- docs/verification/releases/* and scripts/check_release_bundle.py
Reason: retained-release legitimacy center and executable coherence enforcement.

## Identity-Crisis Components

These are components where classification friction remains meaningful.

### crates/replay-core
Why difficult:
- Owned as Replay.
- Behaves like a foundational Trace contract implementation surface.

Interpretation:
- Bridge component: replay execution depends on representation semantics encoded here.

### crates/replay-host (especially binary surface)
Why difficult:
- Clearly replay-centric behavior.
- Release-surface documentation constrains broader host commands to support/experimental scope.

Interpretation:
- Boundary component between released replay slice and exploratory replay capability.

### scripts/artifact_tool.py family
Why difficult:
- Performs replay-critical operations (capture/verify/compare/hash pathways).
- Classified in docs as historical support/reference tooling.

Interpretation:
- Operational bridge between active replay workflows and evidence/support packaging practices.

### crates/xtask
Why difficult:
- Ownership impact is Evidence (release/gate correctness).
- Implementation mode is orchestration glue across heterogeneous systems.

Interpretation:
- Coordination component: system reliability and legitimacy depend on it, even though it defines little core domain semantics.

## Boundary Stress Test (Descriptive)

Scenario:
Assume Replay becomes a consumable subsystem boundary for analysis purposes.

### Components That Stay With Replay (Core Replay Subsystem)

- crates/replay-core
- crates/replay-host (library)
- replay-host binary (bounded replay command surface)
- crates/replay-fw-f446 (capture producer for replay artifacts)
- crates/precision-cli replay-command implementation paths
- docs/spec/rpl0_format_contract.md (as replay format contract dependency)
- docs/replay/FW_F446_CAPTURE_v1.md (capture-side replay contract)
- scripts/artifact_diff.py

Reasoning:
- Removing these most directly damages replay execution, replay comparison, replay capture production, or replay artifact semantics.

### Components That Become External Consumers Of Replay

- scripts/rpl0_witness.py + scripts/check_replay_witness.py
- scripts/check_release_bundle.py
- docs/verification/releases/* retained records
- docs/authority/cli_contract.md
- docs/RELEASE_SURFACE.md
- docs/VERIFICATION_GUIDE.md
- artifacts/* retained/runtime evidence trees

Reasoning:
- These mostly consume replay outputs/contracts for legitimacy, retention, verification, and authority interpretation.

### Components That Become Ambiguous Under This Boundary

- scripts/artifact_tool.py family
- crates/precision-cli as a full crate (beyond replay commands)
- scripts/inspect_artifact.py and parser tests
- crates/xtask
- scripts/fw_gate.py

Why ambiguous:
- They mix replay behavior, evidence enforcement, and orchestration.
- Their value depends on whether the boundary is defined by semantics (replay logic), by operator surface (CLI/gates), or by authority workflow (release legitimacy).

## Synthesis Conclusion

Observed structure after three passes:
- Replay is the operational center of the implemented system.
- Trace provides deep semantic foundations that replay and evidence both rely on.
- Evidence provides the strongest organizing and legitimacy layer for repository-wide workflows.

Most informative result:
- The architecture is not single-axis.
- It repeatedly exhibits bridge components where replay semantics, evidence legitimacy, and orchestration concerns meet in the same surfaces.
