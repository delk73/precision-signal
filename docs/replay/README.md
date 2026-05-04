# Replay MVP Docs

Precision Signal is a deterministic execution validation system centered on
replay, operated through the `precision` CLI against an attached STM32 target
over UART.
This folder documents the replay subsystem within that system.

Use the active authority path first:

- [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md)
- [docs/authority/cli_contract.md](../authority/cli_contract.md)
- [docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md](INTERVAL_CAPTURE_CONTRACT_v1.md)
- [docs/verification/releases/index.md](../verification/releases/index.md)

Terminology used in this folder:

- `artifact` means the authoritative published `precision` provenance artifact
  directory under `artifacts/<run_id>/`
- `RPL` means the portable replay binary format used by `.rpl` files and the
  support/reference replay tooling
- `precision.meta.v1` and `precision.meta.v2` are provenance metadata schema
  versions for published artifacts, not RPL container versions

Current authoritative `precision replay` consumes the published artifact
directory emitted by the CLI result block (`ARTIFACT: artifacts/<run_id>`).
`.rpl` RPL files and `replay-host` are retained as support/reference material,
not current authoritative replay inputs.

## In Scope
- Authoritative replay boundary over published `artifacts/<run_id>` directories
- Legacy RPL wire contract for RPL0 format version 0 (`Header0`,
  `EventFrame0`, LE encoding)
- Current parser and firmware path for RPL0 format version 1 containers with
  legacy 16-byte `EventFrame0` replay semantics (support/reference)
- Deterministic host-side hash-stream and first-divergence logic used by the
  experimental Rust replay host (support/reference)
- Attached STM32 target over UART as the canonical replay path operated through
  the `precision` CLI
- F446 firmware capture->halt->dump path for the current operator workflow over
  USART2 (ST-LINK VCP)
- CI boundary gates relevant to replay crates

## Out of Scope
- Evidence chain / CRC / SHA attestation
- Compression or encryption
- Multi-IRQ capture semantics
- DPW integration/runtime wiring
- Stability guarantees beyond the currently documented implementation
- [docs/wip/](../wip/) research work

## Active Replay Routing
- [docs/authority/cli_contract.md](../authority/cli_contract.md): sole active
  CLI authority, including the published `ARTIFACT: artifacts/<run_id>` result
  path consumed by current authoritative `precision replay`
- [docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md](INTERVAL_CAPTURE_CONTRACT_v1.md): canonical STM32 self-stimulus interval CSV capture contract frozen for
  downstream import
- [docs/spec/rpl0_format_contract.md](../spec/rpl0_format_contract.md): normative portable RPL format authority (support/reference)
- [docs/replay/tooling.md](tooling.md): support/reference replay tooling boundary and local validation guidance
- [docs/replay/RPL0_PARSER_TRACEABILITY.md](RPL0_PARSER_TRACEABILITY.md): rule-level spec -> parser -> tests -> CI traceability matrix
- [docs/replay/ARTIFACT_VALIDATION_WORKFLOW.md](ARTIFACT_VALIDATION_WORKFLOW.md): operator/local/CI validation workflow for artifact parser + toolchain
- [docs/replay/CI_GATES.md](CI_GATES.md): replay-related CI checks that must remain true
- [docs/demos/demo_evidence_packaging.md](../demos/demo_evidence_packaging.md): reference packaged proof route and retained bundle for the completed replay pipeline
- [docs/replay/ISR_ADVISORY.md](ISR_ADVISORY.md): advisory note on ISR behavior and current replay capture limitations

## Historical Replay and Capture References

- [docs/replay/FW_F446_CAPTURE_v1.md](FW_F446_CAPTURE_v1.md): retained historical RPL0 capture contract note
  superseded by the active STM32 interval CSV contract
- [docs/replay/REPLAY_CAPTURE_CONTRACT_v0.md](REPLAY_CAPTURE_CONTRACT_v0.md): legacy RPL0 format version 0 capture acceptance contract retained
  for historical inspection
- [docs/replay/WIRE_FORMAT_v0.md](WIRE_FORMAT_v0.md): byte-level RPL format and parser validation rules
- [docs/replay/HOST_REPLAY_v0.md](HOST_REPLAY_v0.md): experimental Rust replay state transition, hashing, and legacy-frame replay semantics
- [docs/replay/FW_F446_CAPTURE_v0.md](FW_F446_CAPTURE_v0.md): legacy RPL0 format version 0 capture contract note retained for historical traceability
