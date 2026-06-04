# single_stm32_uart_replay_v1

## Purpose

This profile names the current single-board STM32F446 UART replay/evidence path.

It binds the active firmware capture path to a concrete hardware and operator
context:

- target board: STM32F446
- firmware: `replay-fw-f446`
- transport: USART2/UART serial capture
- setup path: ST-LINK reset/flashing
- retained artifact: RPL0/v1 firmware capture
- retained evidence: `docs/verification/releases/1.9.1/`

The profile exists so reviewers can identify the hardware assumptions for the
active replay/evidence path without reconstructing them from the README, release
surface, firmware contract, and retained evidence bundle.

## Active Path

```text
ST-LINK reset/flashing
  -> STM32F446 running replay-fw-f446
  -> USART2/UART capture
  -> RPL0/v1 artifact
  -> retained 1.9.1 evidence
```

## Board and Firmware

The actor target is the STM32F446 path implemented by `replay-fw-f446`.

The firmware emits RPL0 execution evidence over USART2/UART according to the
active [STM32F446 Firmware Capture Contract](../replay/FW_F446_CAPTURE_v1.md).

## Transport and Artifact

The transport is USART2/UART serial capture.

The retained firmware capture artifact is
[fw_capture.bin](../verification/releases/1.9.1/fw_capture.bin), an RPL0/v1
binary execution-evidence artifact emitted by `replay-fw-f446` over USART2/UART.

For the retained `1.9.1` capture, the artifact contains a 152-byte RPL0/v1
header, a 91-byte schema block, and 10,000 fixed-size 16-byte `EventFrame0`
records. Each frame records `frame_idx`, `irq_id`, `flags`, `rsv`,
`timer_delta`, and `input_sample`.

The artifact is replay evidence, not a firmware image and not a sensor
measurement. The RPL0/v1 layout is defined by the
[RPL0 Format Contract](../spec/rpl0_format_contract.md).

## Reset and Flashing

ST-LINK reset/flashing is the setup path for the current release-gating firmware
capture flow.

Reset/flashing establishes setup context. It is not replay evidence by itself.

## Retained Evidence

- [fw_capture.bin](../verification/releases/1.9.1/fw_capture.bin)
- [firmware_release_evidence.md](../verification/releases/1.9.1/firmware_release_evidence.md)
- [fw_capture_hash_check.txt](../verification/releases/1.9.1/fw_capture_hash_check.txt)
- [rpl0_witness_fw_capture.txt](../verification/releases/1.9.1/rpl0_witness_fw_capture.txt)

## Evidence Claim

The STM32F446 target running replay-fw-f446 emits retained RPL0/v1 execution
evidence over UART, and that retained artifact is checked through the 1.9.1
evidence chain.

The retained RPL0 witness report strengthens review by independently parsing and
deterministically folding `fw_capture.bin` through a standalone support path. It
is support evidence, not semantic replay authority.

## Boundaries

This profile is limited to the current STM32F446/ST-LINK/UART replay path.

It does not promote:

- BBB reset orchestration
- dual-board observer evidence
- timing characterization
- sensor validation
- Trace Authority naming
- RPL0 witness output as semantic replay authority
- general embedded-system correctness

## References

- [Precision Signal 2.0 Roadmap](../roadmap/precision-signal-2.0.md)
- [Release Surface](../RELEASE_SURFACE.md)
- [STM32F446 Firmware Capture Contract](../replay/FW_F446_CAPTURE_v1.md)
- [RPL0 Format Contract](../spec/rpl0_format_contract.md)
- [Math Contract](../MATH_CONTRACT.md)
- [RPL0 Witness v1](../replay/RPL0_WITNESS_v1.md)
- [1.9.1 Release Evidence](../verification/releases/1.9.1/index.md)
