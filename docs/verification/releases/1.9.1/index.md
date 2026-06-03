# Release Evidence Bundle (1.9.1)

This directory is the retained release record for `1.9.1`, the active retained
release record for the precision crate-boundary split tree. The prior public
`1.9.0` release is historical and superseded; see
[release_supersession_note.md](release_supersession_note.md).

## Review Commands

```bash
make gate
make authoritative-replay-cli-tests
make release-bundle-check VERSION=1.9.1
```

For hardware-backed validation, use the documented STM32F446/ST-LINK/UART bench
setup and the active RPL0 firmware capture contract.

## Evidence Map

- [summary.md](summary.md): generated retained bundle file inventory and hashes.
- [make_gate.txt](make_gate.txt): retained release-facing software gate output.
- [firmware_release_evidence.md](firmware_release_evidence.md): retained
  firmware capture and repeat-capture evidence.
- [release_reproducibility.txt](release_reproducibility.txt): retained build
  reproducibility evidence.
- [kani_evidence.txt](kani_evidence.txt): retained Kani proof-boundary evidence.
- [make_release_bundle_check.txt](make_release_bundle_check.txt): retained
  release bundle check output.

## Supported Claims

The retained `1.9.1` bundle supports review of the active release surface for
the primary precision CLI surface and the STM32 RPL0 firmware capture path, as
routed by [docs/RELEASE_SURFACE.md](../../../RELEASE_SURFACE.md) and
[docs/verification/releases/index.md](../index.md).

Firmware validation for the current release uses the ST-LINK reset path as the
current release-gating authority. Manual reset is a legacy/manual/support
procedure, not current release-gating authority. BBB reset orchestration remains
support/experimental unless explicitly promoted by a future release or roadmap
document.

## Boundaries

This bundle does not change replay behavior, firmware behavior, reset
orchestration, schema, or proof scope. It does not claim full independent replay
equivalence across independently produced replay/capture paths.

Dual-board observer evidence demonstrates independent external observation of
actor timing/ack behavior. It is supplemental instrumentation evidence for the
current release surface. It is not, by itself, a claim of full independent
replay equivalence across independently produced replay artifacts.
