# Workspace Framing

Precision Signal is a deterministic execution validation system centered on
replay, operated through the `precision` CLI against an attached STM32 target
over UART.

This repository is the multi-crate workspace that implements that system.

This page is descriptive only.
Release status is classified in [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md).
Verification authority is defined in [VERIFICATION_GUIDE.md](../VERIFICATION_GUIDE.md).
Replay-tooling boundary routing lives in [docs/replay/tooling.md](../replay/tooling.md).
Packaged replay proof/support routing lives in
[docs/demos/demo_evidence_packaging.md](../demos/demo_evidence_packaging.md).

Workspace framing:
- `geom-signal`, `geom-spatial`, and `dpw4` provide the core math, spatial, and
  validation/build surfaces.
- Replay-related implementation and operator documentation lives under
  [docs/replay/](../replay/).
- The authoritative replay operator interface is the `precision` CLI, and the
  canonical attached-hardware route is an STM32 target over UART.
- Demo and evidence walkthroughs live under [docs/demos/](../demos/); the
  packaged replay evidence bundle for the demo support path is
  `artifacts/demo_evidence/retained/`.
- Experimental replay-adjacent work remains subject to the release classifications
  in [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md).
