# Start Here

Precision Signal is a deterministic execution validation system centered on
replay, operated through the `precision` CLI against an attached STM32 target
over UART.

This page routes readers to the active contracts, evidence, and operational documentation.
It is the only active reader path. Folder README files and indexes are routing
aids only.

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

After the active reading path, use [DOCS_INDEX.md](DOCS_INDEX.md) only as a thin docs
directory index for readers who land directly in `docs/`.

## Documentation Classes

| Class | Route | Use |
| --- | --- | --- |
| Authority | [VERIFICATION_GUIDE.md](VERIFICATION_GUIDE.md), [RELEASE_SURFACE.md](RELEASE_SURFACE.md), [authority/cli_contract.md](authority/cli_contract.md) | Active release, CLI, and replay authority. These route to lower-level contracts where required. |
| Active operator | [operator/stm32_replay.md](operator/stm32_replay.md), [operations/USB_WORKFLOW.md](operations/USB_WORKFLOW.md), [hardware/REFERENCE_HARDWARE.md](hardware/REFERENCE_HARDWARE.md), [replay/ARTIFACT_VALIDATION_WORKFLOW.md](replay/ARTIFACT_VALIDATION_WORKFLOW.md) | Operator procedures that must defer to authority documents. |
| Active support | [replay/REPLAY_INDEX.md](replay/REPLAY_INDEX.md), [architecture/workspace.md](architecture/workspace.md), [governance/ARTIFACT_POLICY.md](governance/ARTIFACT_POLICY.md), [demos/demo.md](demos/demo.md), [audits/AUDIT_INDEX.md](audits/AUDIT_INDEX.md), [BENCHMARK_GUIDE.md](BENCHMARK_GUIDE.md) | Explanatory, diagnostic, governance, and demo material. |
| Retained release evidence | [verification/releases/index.md](verification/releases/index.md) | Release-scoped records. Leave retained evidence in place. |
| Historical/demo | [demos/demo.md](demos/demo.md), [archive/verification/CLI_SURFACE_EVIDENCE.md](archive/verification/CLI_SURFACE_EVIDENCE.md), [replay/REPLAY_CAPTURE_CONTRACT_v0.md](replay/REPLAY_CAPTURE_CONTRACT_v0.md) | Historical or demonstration material that does not override the active path. |
| Internal/wip | [wip/WIP_INDEX.md](wip/WIP_INDEX.md), [internal/DOC_SYSTEM_CATEGORY_AUDIT.md](internal/DOC_SYSTEM_CATEGORY_AUDIT.md), [archive/wip/1.6.0_surface_inventory.md](archive/wip/1.6.0_surface_inventory.md) | Provisional notes and archived exploratory material. |

## Boundary Notes

- `phase8` is the retained baseline.
- `burst8` and `seeded_lfsr8` are validation-only surfaces.
- Replay self-diff is not independent replay equivalence.
- Retained evidence is distinct from transient artifacts.
