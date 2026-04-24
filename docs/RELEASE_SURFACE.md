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
- retained release records: [docs/verification/releases/](verification/releases/)
- active workspace/package version: `1.7.0`
- active retained release record: `1.7.0`
- latest retained release record currently present in-tree:
  [docs/verification/releases/1.7.0/](verification/releases/1.7.0/)
- active retained release summary: narrowed non-firmware release record for the
  primary precision CLI surface only
- replay demo packaging material under
  [docs/demos/demo_evidence_packaging.md](demos/demo_evidence_packaging.md) and
  `artifacts/demo_evidence/retained/` is support/reference material, not the
  canonical `1.7.0` operator release surface

The `1.7.0` release remains the retained baseline until a later release bundle
exists.

Physical replay characterization for STM32 power-floor and degradation
workflows is prepared as supporting documentation only. It does not expand the
authoritative release surface beyond the exercised `precision` CLI replay path.

Physical characterization evidence is supporting evidence unless explicitly
promoted by a later release packet.

See:

- [docs/verification/hardware_procedures.md](verification/hardware_procedures.md)
  for characterization procedure, provenance requirements, and non-claims

If a descriptive document conflicts with a normative document, the normative
document wins.

If a capability is not listed in this document, it is not part of the release
surface.

## Classification

Canonical For Retained Release `1.7.0`

- `make gate` (canonical operator-facing release gate)
- authoritative replay through `precision record` and `precision replay` over
  retained replay artifacts and exact equivalence / first-divergence reporting
- the narrowed primary precision CLI surface retained for `1.7.0`; read the
  exact release boundary, limits, and retained evidence under
  [docs/verification/releases/1.7.0/](verification/releases/1.7.0/) and
  [VERIFICATION_GUIDE.md](../VERIFICATION_GUIDE.md)
- authority and retained-evidence routing under
  [docs/verification/releases/](verification/releases/)

Support / Reference / Historical Only

- `sig-util validate` (underlying implementation of `make gate`, not separate
  operator-facing release authority)
- `artifact_tool.py` and `artifact_diff.py` (retained support/reference tooling;
  not canonical `1.7.0` operator surface)
- `replay-host diff` (historical bounded `1.5.0` released slice only; exact
  scope note retained under
  [docs/verification/releases/1.5.0/RUST_REPLAY_DIFF_SCOPE.md](verification/releases/1.5.0/RUST_REPLAY_DIFF_SCOPE.md);
  not canonical `1.7.0` operator surface)
- `make demo-evidence-package` and `artifacts/demo_evidence/retained/`
  (retained replay demo proof/support material, not the active `1.7.0` release
  contract)
- firmware capture/import evidence retained under
  [docs/verification/releases/1.7.0/](verification/releases/1.7.0/) is bounded
  supporting evidence only; it does not promote a firmware release for `1.7.0`
- STM32 physical replay characterization and power-floor characterization are
  supporting physical evidence unless promoted by a later release packet with
  retained evidence

Experimental

not part of the current release surface

- `replay-fw-f446` (active STM32 self-stimulus interval CSV contract is explicit in [docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md](replay/INTERVAL_CAPTURE_CONTRACT_v1.md), but the current path is not promoted as released operator tooling)
- `replay-host` commands other than `diff`
- broader `replay-host` capability outside the exact `artifacts/rpl0/` proof corpus and accepted artifact class retained under [docs/verification/releases/1.5.0/RUST_REPLAY_DIFF_SCOPE.md](verification/releases/1.5.0/RUST_REPLAY_DIFF_SCOPE.md)
- schema-aware Rust replay semantics
- `substrate_probe`, `make conformance-audit`, and `make kill-switch-audit`
  (retained audit/probe workflow support, not canonical `1.7.0` operator
  surface)

## Release Routing

- Canonical operator entrypoint: `make gate`
- Underlying support command: `sig-util validate --mode quick`
- Canonical retained release-evidence location for release records:
  [docs/verification/releases/](verification/releases/)
- Active retained release record: `1.7.0`
- `1.7.0` remains the retained baseline until a later retained release bundle
  exists
- Active retained release route:
  [docs/verification/releases/1.7.0/](verification/releases/1.7.0/) for the
  narrowed primary precision non-firmware release record
- Active retained release summary routes:
  [docs/verification/releases/1.7.0/index.md](verification/releases/1.7.0/index.md)
- Supporting firmware evidence retained under the `1.7.0` bundle is bounded
  supporting evidence only and does not widen the released operator surface
- Replay demo packaging material and the historical `1.5.0`
-  `replay-host diff` slice are reference routes, not active `1.7.0` release
  authority
- Retained verification scope for this release surface includes the `1.7.0`
  release-checklist outputs. Historical `1.5.0`, `1.4.0`, `1.3.1`, and
  hardware-backed `1.2.2` retained evidence remains explicit under [docs/verification/releases/](verification/releases/).
- This document classifies surfaced tools and routes proof bundles; it does not
  define release admissibility
- Physical characterization prep is routed through
  [docs/operator/stm32_replay.md](operator/stm32_replay.md) and
  [docs/verification/hardware_procedures.md](verification/hardware_procedures.md)
  as support documentation only; firmware, replay-host, and sidecar surfaces
  are not promoted by implication
