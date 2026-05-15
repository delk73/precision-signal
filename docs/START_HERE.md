# Start Here

Precision Signal is a deterministic execution validation system centered on
replay, operated through the `precision` CLI against an attached STM32 target
over UART.

This page is a reader router. Follow the linked documents for authoritative
contracts, retained evidence, and operational details.

## Core Pillars

- Replay is the center of the system: captured execution evidence is used to
  validate deterministic behavior.
- The active operator surface is the `precision` CLI.
- The canonical hardware path is an attached STM32 target over UART.
- Authority documents define behavior and release boundaries; retained evidence
  records observed validation results.
- Supporting documentation explains architecture, operation, and background
  without replacing the active authority path.

## Naming

- Product name: **Precision Signal**
- CLI command: `precision`
- Hardware path: attached STM32 target over UART

## Active Reading Path

Read these documents in order:

1. [VERIFICATION_GUIDE.md](VERIFICATION_GUIDE.md): release-readiness authority
   and verification contract.
2. [RELEASE_SURFACE.md](RELEASE_SURFACE.md): release-surface classification and
   routing.
3. [authority/cli_contract.md](authority/cli_contract.md): active CLI contract.
4. [replay/FW_F446_CAPTURE_v1.md](replay/FW_F446_CAPTURE_v1.md): active STM32
   replay capture contract.
5. [spec/rpl0_format_contract.md](spec/rpl0_format_contract.md): portable RPL
   format ground truth.
6. [replay/DIVERGENCE_SEMANTICS.md](replay/DIVERGENCE_SEMANTICS.md):
   deterministic divergence explanation and classification contract.
7. [replay/INTERVAL_CAPTURE_CONTRACT_v1.md](replay/INTERVAL_CAPTURE_CONTRACT_v1.md):
   timing characterization capture contract.
8. [verification/releases/index.md](verification/releases/index.md): retained
   release records and historical verification landing.

After the active reading path, use [README.md](README.md) only as a thin docs
directory index for readers who land directly in `docs/`.

## Boundary Notes

- `phase8` is the retained baseline.
- `burst8` and `seeded_lfsr8` are validation-only surfaces.
- Replay self-diff is not independent replay equivalence.
- Retained evidence is distinct from transient artifacts.
