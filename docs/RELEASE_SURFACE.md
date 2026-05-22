# Release Surface

This document is a routing and classification aid for `precision-signal`.
It does not define detailed release procedure.

Normative behavior still comes from:

- [docs/MATH_CONTRACT.md](MATH_CONTRACT.md)
- [docs/spec/rpl0_format_contract.md](spec/rpl0_format_contract.md)
- [VERIFICATION_GUIDE.md](VERIFICATION_GUIDE.md)

For release and verification routing, use:

- core verification authority: [VERIFICATION_GUIDE.md](VERIFICATION_GUIDE.md)
- retained-release mechanics:
  [docs/verification/releases/index.md](verification/releases/index.md)
- per-version retained records:
  [docs/verification/releases/<version>/](verification/releases/)
- canonical operator path: `make gate`
- retained release records: [docs/verification/releases/](verification/releases/)
- active workspace/package version: `1.8.0`
- active retained release record: `1.8.0`
- latest retained release record currently present in-tree:
  [docs/verification/releases/1.8.0/](verification/releases/1.8.0/)
- active retained release summary: firmware-including release record for the
  primary precision CLI surface and the RPL0 firmware capture path
- replay demo packaging material under
  [docs/demos/demo_evidence_packaging.md](demos/demo_evidence_packaging.md) and
  `artifacts/demo_evidence/retained/` is support/reference material, not the
  canonical `1.8.0` operator release surface

If a descriptive document conflicts with a normative document, the normative
document wins.

If a capability is not listed in this document, it is not part of the release
surface.

## Classification

Canonical Active Commands And Routes

- `make gate` (canonical operator-facing release gate)
- `make bench-check` (bench readiness preflight for hardware-backed release
  operations)
- `make fw-gate` as the firmware capture gate executed inside the retained
  `1.8.0` release orchestration
- retained-release preparation commands are routed through
  [docs/verification/releases/index.md](verification/releases/index.md)
- the primary precision CLI surface and RPL0 firmware capture path retained for
  `1.8.0`; read the exact release boundary, limits, and retained evidence under
  [docs/verification/releases/1.8.0/](verification/releases/1.8.0/)
- active RPL0 firmware capture contract:
  [docs/replay/FW_F446_CAPTURE_v1.md](replay/FW_F446_CAPTURE_v1.md)
- authority and retained-evidence routing under
  [docs/verification/releases/](verification/releases/)

Support / Reference / Historical Only

- `sig-util validate` (underlying implementation of `make gate`, not separate
  operator-facing release authority)
- `artifact_tool.py`, `repeat_capture.py`, and `artifact_diff.py` (support tools
  used by release workflows and diagnostics; standalone invocations are not
  separate release authority)
- `replay-host diff` (historical bounded `1.5.0` released slice only; exact
  scope note retained under
  [docs/verification/releases/1.5.0/RUST_REPLAY_DIFF_SCOPE.md](verification/releases/1.5.0/RUST_REPLAY_DIFF_SCOPE.md);
  not canonical `1.8.0` operator surface)
- `make demo-evidence-package` and `artifacts/demo_evidence/retained/`
  (retained replay demo proof/support material, not the active `1.8.0` release
  contract)
- `make demo-divergence` and `make replay-demo-audit` (active demo/support
  validation entrypoints for replay explanation material; not release
  authority)
- `make demo-captured-verify` and `make demo-captured-release` (support checks
  for the committed captured divergence pair)
- historical Demo V2-V5 lifecycle flows (`demo-v2-*`, `demo-v3-*`,
  `demo-v4-*`, `demo-v5-*`) are retained by git history and by their
  fixtures/generators/tests, but are no longer top-level Make operator targets
- interval CSV firmware evidence and timing characterization material are
  support/reference unless explicitly invoked through the timing crate workflows

Experimental

not part of the current release surface

- `replay-fw-f446-timing` (active timing characterization contract is
  [docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md](replay/INTERVAL_CAPTURE_CONTRACT_v1.md);
  timing characterization is not the active release firmware capture path)
- `replay-host` commands other than `diff`
- broader `replay-host` capability outside the exact `artifacts/rpl0/` proof corpus and accepted RPL input class retained under [docs/verification/releases/1.5.0/RUST_REPLAY_DIFF_SCOPE.md](verification/releases/1.5.0/RUST_REPLAY_DIFF_SCOPE.md)
- schema-aware Rust replay semantics
- `substrate_probe`, `make conformance-audit`, and `make kill-switch-audit`
  (retained audit/probe workflow support, not canonical `1.8.0` operator
  surface)

## Release Routing

- Run active verification flow: [VERIFICATION_GUIDE.md](VERIFICATION_GUIDE.md)
- Classify active release surface: this document
- Inspect retained records: [docs/verification/releases/](verification/releases/)
- Prepare retained releases:
  [docs/verification/releases/index.md](verification/releases/index.md)
- Read firmware contract:
  [docs/replay/FW_F446_CAPTURE_v1.md](replay/FW_F446_CAPTURE_v1.md)

This document classifies surfaced tools and routes readers. It does not define
release procedure or release admissibility.
