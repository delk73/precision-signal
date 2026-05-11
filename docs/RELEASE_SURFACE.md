# Release Surface

This document is a routing and classification aid for `precision-signal`.
It is not the release contract.

Normative behavior still comes from:

- [docs/MATH_CONTRACT.md](MATH_CONTRACT.md)
- [docs/spec/rpl0_format_contract.md](spec/rpl0_format_contract.md)
- [VERIFICATION_GUIDE.md](VERIFICATION_GUIDE.md)

For release decisions, use:

- contract: [VERIFICATION_GUIDE.md](VERIFICATION_GUIDE.md)
- canonical operator path: `make gate`
- unified release proof path for new release execution:
  `make release-proof VERSION=<ver>`
- canonical retained-record orchestration: `make release-1.8.0`
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

Canonical For Retained Release `1.8.0`

- `make gate` (canonical operator-facing release gate)
- `make release-proof VERSION=<ver>` (unified release-proof orchestration for
  new release execution; see [VERIFICATION_GUIDE.md](VERIFICATION_GUIDE.md))
- `make bench-check` (bench readiness preflight for hardware-backed release
  operations)
- `make release-1.8.0` (canonical retained-record orchestration for the active
  firmware-including release)
- `make fw-gate` as the firmware capture gate executed inside the retained
  `1.8.0` release orchestration
- the primary precision CLI surface and RPL0 firmware capture path retained for
  `1.8.0`; read the exact release boundary, limits, and retained evidence under
  [docs/verification/releases/1.8.0/](verification/releases/1.8.0/) and
  [VERIFICATION_GUIDE.md](VERIFICATION_GUIDE.md)
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

- Canonical operator entrypoint: `make gate`
- Unified release proof path for new release execution:
  `make release-proof VERSION=<ver>`
- Bench readiness preflight: `make bench-check`
- Canonical retained-record orchestration: `make release-1.8.0`
- Firmware capture gate for this release: `make fw-gate`
- Underlying support command: `sig-util validate --mode quick`
- Canonical retained release-evidence location for release records:
  [docs/verification/releases/](verification/releases/)
- Active retained release record: `1.8.0`
- Active retained release route:
  [docs/verification/releases/1.8.0/](verification/releases/1.8.0/) for the
  firmware-including release record
- Active retained release summary route:
  [docs/verification/releases/1.8.0/index.md](verification/releases/1.8.0/index.md)
- Future generated bundle summaries are retained under
  `docs/verification/releases/<version>/`; historical bundles without generated
  summaries remain valid
- Active RPL0 firmware capture contract route:
  [docs/replay/FW_F446_CAPTURE_v1.md](replay/FW_F446_CAPTURE_v1.md)
- Timing characterization route:
  [docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md](replay/INTERVAL_CAPTURE_CONTRACT_v1.md)
  for `replay-fw-f446-timing`, support/reference relative to the active release
- Replay demo packaging material and the historical `1.5.0` `replay-host diff`
  slice are reference routes, not active `1.8.0` release authority
- Retained verification scope for this release surface includes the `1.8.0`
  release-checklist outputs and archived RPL0 firmware evidence. Historical
  `1.7.0`, `1.5.0`, `1.4.0`, `1.3.1`, and
  hardware-backed `1.2.2` retained evidence remains explicit under [docs/verification/releases/](verification/releases/).
- This document classifies surfaced tools and routes proof bundles; it does not
  define release admissibility
