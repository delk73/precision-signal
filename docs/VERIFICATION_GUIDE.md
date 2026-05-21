# Verification Guide

## Purpose

Use this guide to choose the correct verification path.

## Choose Your Path

| Intent | Do this | Not this |
| --- | --- | --- |
| Validate local repo | `make gate` | Kani, release bundle work |
| Bring up STM32 board | `make bench-check SERIAL=<device>`, then `make fw-gate SERIAL=<device>` | Kani, manual reset |
| Inspect retained release | `docs/verification/releases/<version>/` | New capture unless revalidating |
| Prepare retained release | `docs/verification/releases/index.md` | Ad hoc board/debug flow |

## Local Validation

```text
make gate
```

## STM32 Board Bring-Up

```text
make bench-check SERIAL=<device>
make fw-gate SERIAL=<device>
```

The active board-bringup path uses ST-LINK reset by default. It must not ask
the operator to press reset manually.

Kani is not part of board bring-up.

## Release Inspection

Start with the retained release record:

```text
docs/verification/releases/<version>/
```

## Release Preparation

Use the retained-release mechanics index:

```text
docs/verification/releases/index.md
```

## Kani Boundary

Kani is release/proof-boundary evidence, not local validation or board bring-up.

## Authority Links

- release mechanics: [docs/verification/releases/index.md](verification/releases/index.md)
- release classification: [docs/RELEASE_SURFACE.md](RELEASE_SURFACE.md)
- STM32 firmware contract: [docs/replay/FW_F446_CAPTURE_v1.md](replay/FW_F446_CAPTURE_v1.md)
- RPL0 format contract: [docs/spec/rpl0_format_contract.md](spec/rpl0_format_contract.md)
