# Documentation Index

This directory contains the descriptive, architectural, operational, and
supporting evidence documents for `precision-signal`.

Precision Signal is a deterministic execution validation system centered on
replay, operated through the `precision` CLI against an attached STM32 target
over UART.
This documentation set describes the architecture, operation, and evidence for that system.

This index is descriptive only. For release and verification routing, use the
active authority path before reading supporting material here.

## Active Authority Path

- [VERIFICATION_GUIDE.md](../VERIFICATION_GUIDE.md): release-readiness authority and verification contract
- [docs/RELEASE_SURFACE.md](RELEASE_SURFACE.md): release-surface classification and routing
- [docs/authority/cli_contract.md](authority/cli_contract.md): sole active CLI contract authority
- [docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md](replay/INTERVAL_CAPTURE_CONTRACT_v1.md): sole active STM32 capture contract authority

## Historical / Release-Record Landing

- [docs/verification/releases/index.md](verification/releases/index.md): retained release records and historical verification landing

Experimental notes live under [docs/wip/README.md](wip/README.md) and are non-normative.

Normative behavior is covered by the contract and spec documents listed below:

- [docs/MATH_CONTRACT.md](MATH_CONTRACT.md): arithmetic, signal-path, narrowing, and saturation contract
- [docs/spec/rpl0_artifact_contract.md](spec/rpl0_artifact_contract.md): normative replay artifact format
- [docs/spec/dpw_gain.md](spec/dpw_gain.md): gain model invariants and domain specification
- [docs/spec/oscillator_api.md](spec/oscillator_api.md): oscillator dispatch contract
- [docs/spec/reference_invariants.md](spec/reference_invariants.md): mathematical reference invariants
- [docs/spec/pulse_implementation_spec.md](spec/pulse_implementation_spec.md): pulse and square waveform spec
- [docs/spec/header_layout_addendum.md](spec/header_layout_addendum.md): header layout addendum

## Start Here

- [docs/architecture/workspace.md](architecture/workspace.md): workspace framing and routing
- [docs/architecture/architecture_whitepaper.md](architecture/architecture_whitepaper.md): replay architecture whitepaper
- [docs/architecture/signal_path.md](architecture/signal_path.md): non-normative signal-path overview
- [docs/architecture/float_boundary.md](architecture/float_boundary.md): float quarantine and allowed surfaces
- [docs/architecture/system_surfaces.md](architecture/system_surfaces.md): build surfaces, environment segmentation,
  and implementation routing (not a release classifier)
- [docs/system_architecture_disclosure.md](system_architecture_disclosure.md): system architecture overview

## Replay

- [docs/replay/README.md](replay/README.md): replay subsystem scope and document index
- [docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md](replay/INTERVAL_CAPTURE_CONTRACT_v1.md): current STM32 self-stimulus capture contract
- [docs/replay/tooling.md](replay/tooling.md): support/reference replay tooling boundary and validation guidance
- [docs/replay/DIVERGENCE_SEMANTICS.md](replay/DIVERGENCE_SEMANTICS.md): normative replay explanation and classification contract
- [docs/architecture/replay_explained.md](architecture/replay_explained.md): single-pass replay system narrative and demo ladder

## Verification

- [docs/verification/artifact_sets.md](verification/artifact_sets.md): determinism artifacts, forensic hashing, Kani,
  and physical verification routing
- [docs/verification/build_reproducibility.md](verification/build_reproducibility.md): pinned toolchain, release build, and
  reproducibility checks
- [docs/verification/CROSS_CONTEXT_INVARIANCE.md](verification/CROSS_CONTEXT_INVARIANCE.md): cross-context replay invariance check for RPL0 format version 1 artifacts
- [docs/verification/chaos_probes.md](verification/chaos_probes.md): chaos-probe verification notes
- [docs/verification/hardware_procedures.md](verification/hardware_procedures.md): non-authoritative manual hardware support procedures
- [docs/verification/releases/index.md](verification/releases/index.md): retained release records and historical verification landing

## CLI

- [docs/authority/cli_contract.md](authority/cli_contract.md): authoritative CLI invocation, stream, and exit contract
- [docs/replay/tooling.md](replay/tooling.md): support/reference tooling guidance and local replay validation gates

## Demo

- [docs/demos/demo.md](demos/demo.md): deterministic replay divergence demo
- [docs/demos/demo_capture_perturbation.md](demos/demo_capture_perturbation.md): controlled capture perturbation workflow
- [docs/demos/demo_persistent_divergence.md](demos/demo_persistent_divergence.md): persistent divergence replay workflow
- [docs/demos/demo_v3_divergence_classification.md](demos/demo_v3_divergence_classification.md): divergence classification demo
- [docs/demos/demo_v4_region_attribution.md](demos/demo_v4_region_attribution.md): divergence region attribution demo
- [docs/demos/demo_v5_evolution.md](demos/demo_v5_evolution.md): divergence evolution semantics demo
- [docs/demos/demo_captured_divergence.md](demos/demo_captured_divergence.md): hardware capture evidence demo
- [docs/demos/demo_evidence_packaging.md](demos/demo_evidence_packaging.md): reference packaged proof route and retained bundle for the replay demo pipeline; not part of the `1.6.0` release contract
- [docs/demos/demo_visual.html](demos/demo_visual.html): visual demo artifact retained for local inspection
- [docs/architecture/replay_explained.md](architecture/replay_explained.md): narrative replay explanation ladder
- [docs/replay/DIVERGENCE_SEMANTICS.md](replay/DIVERGENCE_SEMANTICS.md): normative replay explanation contract
- [docs/demos/demo_claim_matrix.md](demos/demo_claim_matrix.md): claims-to-evidence map for reference demo surfaces

## Governance

- [docs/governance/DESIGN_AXIOMS.md](governance/DESIGN_AXIOMS.md): normative design principles
- [docs/governance/ARTIFACT_POLICY.md](governance/ARTIFACT_POLICY.md): artifact retention policy and bounded support classification
- [docs/governance/DEBT.md](governance/DEBT.md): tracked design debt

## Hardware

- [docs/hardware/REFERENCE_HARDWARE.md](hardware/REFERENCE_HARDWARE.md): optional hardware observation procedures

## Operations

- [docs/operations/USB_WORKFLOW.md](operations/USB_WORKFLOW.md): USB flashing and reporting workflow

## Performance

- [docs/architecture/performance/README.md](architecture/performance/README.md): retained performance background and benchmarking index

## Debug

- [docs/debug/diagnostic_features.md](debug/diagnostic_features.md): diagnostic-only firmware features and usage
- [docs/debug/reset_run_characterization.md](debug/reset_run_characterization.md): bring-up, reset, attach, and recovery
  runbook

## Audits

- [docs/audits/README.md](audits/README.md): audit ledger index and runlog retention
- [docs/audits/PRE_RELEASE_AUDIT.md](audits/PRE_RELEASE_AUDIT.md): release audit process

## Roadmap

- [docs/roadmap/witness_model_direction.md](roadmap/witness_model_direction.md): witness-model direction note
