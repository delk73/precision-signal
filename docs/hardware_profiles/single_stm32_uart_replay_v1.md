# single_stm32_uart_replay_v1

## Status

This is the named hardware profile for the current active STM32F446 UART replay
path.

It is a documentation profile for the current replay/evidence path. It is not a
new release artifact and not a new hardware capability.

## Purpose

This profile collects the board, firmware, transport, reset/flashing, artifact,
and evidence assumptions for the current single-board STM32F446 UART
replay/capture path.

## Board Role

Board: STM32F446 target, specifically the current replay-fw-f446 path.

Role: actor target that emits RPL0 execution evidence over UART.

This profile does not generalize the current release path to all STM32-class
targets.

## Firmware Role

Firmware: replay-fw-f446.

Role: produces the current RPL0 firmware capture artifact over USART2/UART.

The active firmware capture contract is
[docs/replay/FW_F446_CAPTURE_v1.md](../replay/FW_F446_CAPTURE_v1.md).

## Transport / Interface

Transport: USART2 / UART serial capture.

Artifact: RPL0/v1 firmware capture.

The portable RPL0/v1 file layout and parser contract are defined by
[docs/spec/rpl0_format_contract.md](../spec/rpl0_format_contract.md).

## Reset and Flashing Assumptions

ST-LINK reset / flashing is the current release-gating reset/flashing path.

Reset/flashing establishes setup context. It is not replay evidence by itself.

BBB reset orchestration remains support/experimental unless explicitly promoted
by a later release or roadmap.

Manual reset is legacy/manual/support, not current release-gating authority.

## Wiring Assumptions

The profile assumes the documented STM32F446/ST-LINK/UART bench setup used by
the active firmware capture contract and retained 1.9.1 evidence.

The active firmware capture contract documents USART2/UART capture but does not
define pin-level wiring in this profile. Pin-level wiring details are therefore
outside this profile unless promoted by a later contract or release record.

## Generated / Retained Artifacts

Current retained firmware evidence for this profile includes:

- [docs/verification/releases/1.9.1/fw_capture.bin](../verification/releases/1.9.1/fw_capture.bin)
- [docs/verification/releases/1.9.1/firmware_release_evidence.md](../verification/releases/1.9.1/firmware_release_evidence.md)
- [docs/verification/releases/1.9.1/fw_capture_hash_check.txt](../verification/releases/1.9.1/fw_capture_hash_check.txt)
- [docs/verification/releases/1.9.1/rpl0_witness_fw_capture.txt](../verification/releases/1.9.1/rpl0_witness_fw_capture.txt)

`rpl0_witness_fw_capture.txt` is retained support evidence, not release
authority.

## Evidence Claim

This profile supports review of the current STM32F446 UART replay/evidence path:
firmware capture produces retained RPL0/v1 execution evidence over UART; the
retained artifact is checked through the current release evidence chain; the
retained RPL0 witness report independently parses and folds the retained
firmware capture as support evidence.

The profile does not claim general embedded-system correctness.

## Explicit Non-Claims

This profile does not claim:

- sensor validation
- calibrated measurement accuracy
- field instrumentation readiness
- universal STM32 support
- general embedded-system correctness
- universal embedded security
- adversarial/provenance completeness
- networked multi-node resilience
- BBB reset authority
- observer timing as actor-internal replay equivalence
- RPL0 witness output as semantic replay authority
- Trace Authority rebrand completion

## Support / Supplemental Material

Dual-board observer, timing characterization, BBB orchestration, replay
diagnostics, demo evidence, and RPL0 witness material may support review or
future roadmap decisions, but are not promoted to release authority by this
profile.

The retained RPL0 witness output is support evidence that bolsters review of the
replay/evidence path. It is not semantic replay authority.

## References

- [Precision Signal 2.0 Roadmap](../roadmap/precision-signal-2.0.md)
- [Release Surface](../RELEASE_SURFACE.md)
- [STM32F446 Firmware Capture Contract](../replay/FW_F446_CAPTURE_v1.md)
- [RPL0 Format Contract](../spec/rpl0_format_contract.md)
- [Math Contract](../MATH_CONTRACT.md)
- [RPL0 Witness v1](../replay/RPL0_WITNESS_v1.md)
- [1.9.1 Release Evidence](../verification/releases/1.9.1/index.md)
