# Documentation Index

This directory contains the descriptive, architectural, operational, and evidence
documents for `precision-signal`.

`precision-signal` is a multi-crate workspace. Its system category is
deterministic execution analysis infrastructure.

This index is descriptive only. Use [docs/RELEASE_SURFACE.md](RELEASE_SURFACE.md)
for release classification, [VERIFICATION_GUIDE.md](../VERIFICATION_GUIDE.md) for
verification authority, and [docs/replay/tooling.md](replay/tooling.md)
for replay-tooling boundaries.

Normative behavior is defined outside this index:

- [docs/MATH_CONTRACT.md](MATH_CONTRACT.md): arithmetic, signal-path, narrowing, and saturation contract
- [docs/spec/rpl0_artifact_contract.md](spec/rpl0_artifact_contract.md): normative replay artifact format
- [docs/spec/dpw_gain.md](spec/dpw_gain.md): gain model invariants and domain specification
- [docs/spec/oscillator_api.md](spec/oscillator_api.md): oscillator dispatch contract
- [docs/spec/reference_invariants.md](spec/reference_invariants.md): mathematical reference invariants
- [docs/spec/pulse_implementation_spec.md](spec/pulse_implementation_spec.md): pulse and square waveform spec
- [docs/spec/header_layout_addendum.md](spec/header_layout_addendum.md): header layout addendum
- [VERIFICATION_GUIDE.md](../VERIFICATION_GUIDE.md): conformance governance and verification protocol

If a descriptive document disagrees with a normative one, the normative document
wins.

## Start Here

- [docs/architecture/workspace.md](architecture/workspace.md): workspace framing and routing
- [docs/RELEASE_SURFACE.md](RELEASE_SURFACE.md): release-surface routing and classification
- [docs/architecture/architecture_whitepaper.md](architecture/architecture_whitepaper.md): replay architecture whitepaper
- [docs/architecture/signal_path.md](architecture/signal_path.md): non-normative signal-path overview
- [docs/architecture/float_boundary.md](architecture/float_boundary.md): float quarantine and allowed surfaces
- [docs/architecture/system_surfaces.md](architecture/system_surfaces.md): build surfaces, environment segmentation,
  and implementation routing (not a release classifier)
- [docs/system_architecture_disclosure.md](system_architecture_disclosure.md): system architecture overview

## Replay

- [docs/replay/README.md](replay/README.md): replay subsystem scope and document index
- [docs/replay/tooling.md](replay/tooling.md): replay-tooling boundary, operator tools, validation suites, and local gates
- [docs/replay/HOST_REPLAY_v0.md](replay/HOST_REPLAY_v0.md): host replay and divergence semantics
- [docs/replay/FW_F446_CAPTURE_v1.md](replay/FW_F446_CAPTURE_v1.md): current capture contract and board workflow
- [docs/replay/FW_F446_CAPTURE_v0.md](replay/FW_F446_CAPTURE_v0.md): legacy historical capture contract
- [docs/replay/WIRE_FORMAT_v0.md](replay/WIRE_FORMAT_v0.md): byte-level artifact format
- [docs/replay/DIVERGENCE_SEMANTICS.md](replay/DIVERGENCE_SEMANTICS.md): normative replay explanation and classification contract
- [docs/architecture/replay_explained.md](architecture/replay_explained.md): single-pass replay system narrative and demo ladder

## Verification

- [docs/verification/artifact_sets.md](verification/artifact_sets.md): determinism artifacts, forensic hashing, Kani,
  and physical verification routing
- [docs/verification/build_reproducibility.md](verification/build_reproducibility.md): pinned toolchain, release build, and
  reproducibility checks
- [docs/verification/CI_EVIDENCE.md](verification/CI_EVIDENCE.md): CI evidence notes
- [docs/verification/CROSS_CONTEXT_INVARIANCE.md](verification/CROSS_CONTEXT_INVARIANCE.md): cross-context replay invariance check for RPL0 format version 1 artifacts
- [docs/verification/chaos_probes.md](verification/chaos_probes.md): chaos-probe verification notes
- [docs/verification/FIRMWARE_CAPTURE_EVIDENCE.md](verification/FIRMWARE_CAPTURE_EVIDENCE.md): retained firmware capture evidence routing
- [docs/verification/D-03_TriangleDPW4_Audit.md](verification/D-03_TriangleDPW4_Audit.md): retained TriangleDPW4 audit note
- [docs/verification/releases/](verification/releases/): retained release evidence bundles

## CLI

- [docs/cli/precision.md](cli/precision.md): `precision` CLI reference
- [docs/cli/examples.md](cli/examples.md): CLI usage examples

## Demo

- [docs/demos/demo.md](demos/demo.md): deterministic replay divergence demo
- [docs/demos/demo_capture_perturbation.md](demos/demo_capture_perturbation.md): controlled capture perturbation workflow
- [docs/demos/demo_persistent_divergence.md](demos/demo_persistent_divergence.md): persistent divergence replay workflow
- [docs/demos/demo_v3_divergence_classification.md](demos/demo_v3_divergence_classification.md): divergence classification demo
- [docs/demos/demo_v4_region_attribution.md](demos/demo_v4_region_attribution.md): divergence region attribution demo
- [docs/demos/demo_v5_evolution.md](demos/demo_v5_evolution.md): divergence evolution semantics demo
- [docs/demos/demo_captured_divergence.md](demos/demo_captured_divergence.md): hardware capture evidence demo
- [docs/demos/demo_visual.html](demos/demo_visual.html): visual demo artifact retained for local inspection
- [docs/architecture/replay_explained.md](architecture/replay_explained.md): narrative replay explanation ladder
- [docs/replay/DIVERGENCE_SEMANTICS.md](replay/DIVERGENCE_SEMANTICS.md): normative replay explanation contract
- [docs/demos/demo_claim_matrix.md](demos/demo_claim_matrix.md): claims-to-evidence map for reference demo surfaces

## Governance

- [docs/governance/DESIGN_AXIOMS.md](governance/DESIGN_AXIOMS.md): normative design principles
- [docs/governance/DEBT.md](governance/DEBT.md): tracked design debt

## Hardware

- [docs/hardware/REFERENCE_HARDWARE.md](hardware/REFERENCE_HARDWARE.md): optional hardware observation procedures

## Operations

- [docs/operations/USB_WORKFLOW.md](operations/USB_WORKFLOW.md): USB flashing and reporting workflow

## Performance

- [docs/architecture/performance/README.md](architecture/performance/README.md): performance benchmarking index
- [docs/architecture/performance/CONTROL_SCHEDULER_BENCHMARKING.md](architecture/performance/CONTROL_SCHEDULER_BENCHMARKING.md): control scheduler benchmarking notes
- [docs/architecture/performance/HOT_PATH_EXECUTION_AND_BENCHMARKING.md](architecture/performance/HOT_PATH_EXECUTION_AND_BENCHMARKING.md): hot-path execution and benchmarking notes

## Debug

- [docs/debug/diagnostic_features.md](debug/diagnostic_features.md): diagnostic-only firmware features and usage
- [docs/debug/reset_run_characterization.md](debug/reset_run_characterization.md): bring-up, reset, attach, and recovery
  runbook

## Audits

- [docs/audits/README.md](audits/README.md): audit ledger index and runlog retention
- [docs/audits/PRE_RELEASE_AUDIT.md](audits/PRE_RELEASE_AUDIT.md): release audit process

## Roadmap

- [docs/roadmap/witness_model_direction.md](roadmap/witness_model_direction.md): witness-model direction note
