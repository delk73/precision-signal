# STM32 Power Sweep Evidence - 2026-05-12

This is retained physical characterization evidence.

It is NOT release evidence.

Project direction:

Precision Signal is a deterministic execution validation system centered on replay, operated through the precision CLI against an attached STM32 target over UART.

## Sweep Evidence Chain

1. binary RPL0 artifact capture
2. artifact verification
3. artifact hash
4. run matrix classification

This sweep does not redefine replay authority, CLI authority, release authority, or release evidence.

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

## Sweep Capture Workflow

Capture uses binary RPL0 artifact capture.

Binary RPL0 artifacts are collected with:

`scripts/artifact_tool.py capture`

Captured artifacts are verified with:

`scripts/artifact_tool.py verify`

Artifact hashes are recorded with:

`scripts/artifact_tool.py hash`

Active firmware emits binary RPL0 framing.
`scripts/csv_capture.py` is incompatible with the active firmware path and is not used for this sweep.

## Bundle Files

- [power_notes.md](power_notes.md)
- [run_matrix.csv](run_matrix.csv)
- [videos/index.md](videos/index.md)
