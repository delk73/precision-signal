# Release Surface

This document is a routing and classification aid for `precision-signal`.
It is not the release contract.

Normative behavior still comes from:

- [docs/MATH_CONTRACT.md](MATH_CONTRACT.md)
- [docs/spec/rpl0_artifact_contract.md](spec/rpl0_artifact_contract.md)
- [VERIFICATION_GUIDE.md](../VERIFICATION_GUIDE.md)

For release decisions, use:

- contract: [VERIFICATION_GUIDE.md](../VERIFICATION_GUIDE.md)
- canonical operator path: `make gate`
- canonical packaged proof path for the completed replay pipeline:
  [docs/demos/demo_evidence_packaging.md](demos/demo_evidence_packaging.md)
- retained packaged proof bundle for that path: `artifacts/demo_evidence/retained/`
- retained release records: [docs/verification/releases/](verification/releases/)
- active workspace/package version: `1.4.0`
- latest retained release record currently present in-tree:
  [docs/verification/releases/1.4.0/](verification/releases/1.4.0/)

If a descriptive document conflicts with a normative document, the normative
document wins.

If a capability is not listed in this document, it is not part of the release
surface.

## Classification

Release

- `precision validate` (canonical validation gate)
- `artifact_tool.py` (artifact verification / hashing / inspection)
- `artifact_diff.py` (deterministic divergence analysis)
- `precision generate` (operator-path evidence: [docs/verification/CLI_SURFACE_EVIDENCE.md](verification/CLI_SURFACE_EVIDENCE.md))
- `precision artifacts` (operator-path evidence: [docs/verification/CLI_SURFACE_EVIDENCE.md](verification/CLI_SURFACE_EVIDENCE.md))
- `precision inspect` (operator-path evidence: [docs/verification/CLI_SURFACE_EVIDENCE.md](verification/CLI_SURFACE_EVIDENCE.md))
- `precision verify` (operator-path evidence: [docs/verification/CLI_SURFACE_EVIDENCE.md](verification/CLI_SURFACE_EVIDENCE.md))
- `header_audit` (operator-path evidence: [docs/verification/CLI_SURFACE_EVIDENCE.md](verification/CLI_SURFACE_EVIDENCE.md))

What is proven for the completed Phase 1 through Phase 5 replay pipeline:

- the packaged proof workflow `make demo-evidence-package` reproduces the
  retained replay evidence bundle byte-for-byte from committed inputs
- released Python tooling proves artifact verification, hashing, inspection, and
  deterministic divergence analysis on the packaged fixtures
- the packaged proof bundle demonstrates the completed replay evidence ladder up
  through captured-evidence packaging without changing the release contract

Experimental

not part of the current release surface

- `replay-fw-f446` (active STM32 self-stimulus interval CSV contract is explicit in [docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md](replay/INTERVAL_CAPTURE_CONTRACT_v1.md), but the current path is not promoted as released operator tooling)
- `replay-host` (experimental Rust replay engine: RPL0 format version 0 replay, RPL0 `version = 1` container parsing, and legacy 16-byte `EventFrame0` replay semantics)

## Release Routing

- Canonical operator entrypoint: `make gate`
- Normative underlying command: `precision validate --mode quick`
- Canonical proof route for the completed replay pipeline:
  `make demo-evidence-package` via [docs/demos/demo_evidence_packaging.md](demos/demo_evidence_packaging.md)
- Canonical retained proof bundle for that route: `artifacts/demo_evidence/retained/`
- Canonical retained release-evidence location for release records:
  [docs/verification/releases/](verification/releases/)
- Retained verification scope for this release surface includes the `1.4.0`
  release-checklist outputs and verification-scope statement under [docs/verification/releases/1.4.0/](verification/releases/1.4.0/). Historical `1.3.1` and hardware-backed `1.2.2` retained evidence remains explicit under [docs/verification/releases/](verification/releases/).
- This document classifies surfaced tools and routes proof bundles; it does not
  define release admissibility
