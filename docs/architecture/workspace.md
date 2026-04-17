# Workspace Framing

This repository is a multi-crate workspace implementing
deterministic execution analysis infrastructure.
Precision Signal is a deterministic execution validation system centered on
replay, operated through the `precision` CLI against an attached STM32 target
over UART.

System category:
deterministic execution analysis infrastructure

This page is descriptive only.
Release status is classified in [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md).
Verification authority is defined in [VERIFICATION_GUIDE.md](../../VERIFICATION_GUIDE.md).
Replay-tooling boundary routing lives in [docs/replay/tooling.md](../replay/tooling.md).
The canonical packaged proof route for the completed replay pipeline is
[docs/demos/demo_evidence_packaging.md](../demos/demo_evidence_packaging.md).

Workspace framing:
- `geom-signal`, `geom-spatial`, and `dpw4` provide the core math, spatial, and
  validation/build surfaces.
- Replay-related implementation and operator documentation lives under
  [docs/replay/](../replay/).
- The authoritative replay operator interface is the `precision` CLI, and the
  canonical attached-hardware route is an STM32 target over UART.
- Demo and evidence walkthroughs live under [docs/demos/](../demos/); the
  canonical retained proof bundle for the completed replay pipeline is
  `artifacts/demo_evidence/retained/`.
- Experimental replay-adjacent work remains subject to the release classifications
  in [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md).
