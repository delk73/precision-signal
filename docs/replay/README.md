# Replay MVP Docs

This folder documents the replay subsystem within the broader deterministic
execution analysis infrastructure implemented by `precision-signal`.
Precision Signal is a deterministic execution validation system centered on
replay, operated through the `precision` CLI against an attached STM32 target
over UART.

Use the active authority spine first:

- [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md)
- [docs/authority/cli_contract.md](../authority/cli_contract.md)
- [docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md](INTERVAL_CAPTURE_CONTRACT_v1.md)
- [docs/verification/releases/index.md](../verification/releases/index.md)

## In Scope
- Legacy artifact wire contract for RPL0 format version 0 (`Header0`,
  `EventFrame0`, LE encoding)
- Current parser and firmware path for RPL0 format version 1 containers with
  legacy 16-byte `EventFrame0` replay semantics
- Deterministic host-side hash-stream and first-divergence logic used by the
  experimental Rust replay host
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
- [docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md](INTERVAL_CAPTURE_CONTRACT_v1.md): canonical STM32 self-stimulus interval CSV capture contract frozen for
  downstream import
- [docs/spec/rpl0_artifact_contract.md](../spec/rpl0_artifact_contract.md): normative RPL0 artifact contract
  (`[HEADER][SCHEMA BLOCK][FRAME DATA]`)
- [docs/replay/tooling.md](tooling.md): support/reference replay tooling boundary and local validation guidance
- [docs/replay/RPL0_PARSER_TRACEABILITY.md](RPL0_PARSER_TRACEABILITY.md): rule-level spec -> parser -> tests -> CI traceability matrix
- [docs/replay/ARTIFACT_VALIDATION_WORKFLOW.md](ARTIFACT_VALIDATION_WORKFLOW.md): operator/local/CI validation workflow for artifact parser + toolchain
- [docs/replay/CI_GATES.md](CI_GATES.md): replay-related CI checks that must remain true
- [docs/demos/demo_evidence_packaging.md](../demos/demo_evidence_packaging.md): reference packaged proof route and retained bundle for the completed replay pipeline
- [docs/replay/ISR_ADVISORY.md](ISR_ADVISORY.md): advisory note on ISR behavior and current replay capture limitations

## Historical Replay and Capture References

- [docs/replay/FW_F446_CAPTURE_v1.md](FW_F446_CAPTURE_v1.md): retained historical RPL0 artifact-capture contract note
  superseded by the active STM32 interval CSV contract
- [docs/replay/REPLAY_CAPTURE_CONTRACT_v0.md](REPLAY_CAPTURE_CONTRACT_v0.md): legacy RPL0 format version 0 capture acceptance contract retained
  for historical inspection
- [docs/replay/WIRE_FORMAT_v0.md](WIRE_FORMAT_v0.md): byte-level artifact format and parser validation rules
- [docs/replay/HOST_REPLAY_v0.md](HOST_REPLAY_v0.md): experimental Rust replay state transition, hashing, and legacy-frame replay semantics
- [docs/replay/FW_F446_CAPTURE_v0.md](FW_F446_CAPTURE_v0.md): legacy RPL0 format version 0 capture contract note retained for historical traceability
