# STM32 Power Sweep Evidence - 2026-05-12

This is retained physical characterization evidence.

It is NOT release evidence.

Project direction:

Precision Signal is a deterministic execution validation system centered on replay, operated through the precision CLI against an attached STM32 target over UART.

Replay authority remains:

1. CSV capture
2. precision record
3. authoritative artifact
4. precision replay
5. ResultBlock
6. artifact hash

Bench provenance only:

- power measurements
- scope observations
- video observations
- operator notes

## Current Bench State

- Board: STM32F446RE
- Loopback: PA6 -> PA0
- Serial: /dev/ttyACM0
- ST-LINK connected
- External supply in use

External power is dominant.

Residual interface-side power contribution was not independently isolated.

## Completed Scope Work

External input rail:
- flat
- no visible sag
- no oscillation

3V3 rail:
- flat
- tracked downward
- continued into ~1.x range
- no visible oscillation

Exact dropout threshold not precisely measured.

## Planned Runs

Planned runs remain pending. See [run_matrix.csv](run_matrix.csv).

## Bundle Files

- [power_notes.md](power_notes.md)
- [run_matrix.csv](run_matrix.csv)
- [videos/index.md](videos/index.md)
