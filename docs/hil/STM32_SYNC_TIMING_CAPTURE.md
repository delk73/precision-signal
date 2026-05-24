# STM32 Sync Timing Capture

## Purpose

Retained HIL evidence for the single-board split-capture timing proof.

The measured value is:

```text
PB9 capture of PA1/A1 acknowledgment - PB8 capture of PA6/D12 trigger
```

This is observed timing through the selected measurement wiring. It is not exact
internal PA0-to-PA1 silicon latency.

## Wiring

Functional path:

```text
PA6/D12 -> PA0/A0
```

Measurement paths:

```text
PA6/D12 -> PB8/TIM4_CH3
PA1/A1  -> PB9/TIM4_CH4
GND shared
```

Use short direct wiring. Keep PA6->PA0 and PA6->PB8 as short and similar as
practical. Keep PA1->PB9 short. Avoid breadboard routing for timing evidence.

## Firmware

```sh
make fw
FW_FEATURES="sync_trigger_out sync_trigger_in" make fw
FW_FEATURES="sync_trigger_out sync_trigger_in sync_timing_capture" make fw
FW_FEATURES="sync_trigger_out sync_trigger_in sync_timing_capture" make flash-ur
```

`sync_timing_capture` emits only the `SYNC_TIMING_CAPTURE_V1` text report over
USART2. It does not emit RPL0 in the same run and does not change RPL0 format,
replay/diff behavior, `precision` CLI behavior, `sync_trigger_out`,
`sync_trigger_in`, or the strict `< 100 ns` threshold.

TIM4 is configured as a free-running 16-bit capture timer at 90 MHz:

```text
PB8/TIM4_CH3 captures PA6 rising edge
PB9/TIM4_CH4 captures PA1 rising edge
PSC=0
ARR=0xffff
threshold_ticks=9
```

After `trigger_count` reaches 10,000, firmware stops generating PA6 pulses but
keeps EXTI0 and TIM4 capture interrupts enabled for a bounded grace interval.
This lets the final PA0-triggered PA1 acknowledgment and PB9/TIM4_CH4 capture
arrive before the final drain, shutdown, pending-trigger accounting, and report.

`max_delta_ticks` is authoritative. `max_delta_ns` is display-only. Pass is valid
only when:

```text
missed_ack_count == 0
unexpected_ack_count == 0
capture_error_count == 0
max_delta_ticks < 9
result=PASS
```

A failing timing result is acceptable if reported honestly as `result=FAIL`.
`unexpected_ack_count` records PB9/TIM4_CH4 acknowledgment captures that arrived
when no trigger was pending. Any nonzero `unexpected_ack_count` means the timing
evidence failed because the acknowledgment stream contained an unpaired capture.
`capture_error_count` records TIM4 CH3/CH4 overcapture flags; any nonzero value
means at least one capture event was overwritten before firmware handled it.

## Report

```text
SYNC_TIMING_CAPTURE_V1
timer_hz=90000000
threshold_ticks=9
trigger_count=10000
ack_count=
missed_ack_count=
unexpected_ack_count=
capture_error_count=
max_delta_ticks=
max_delta_ns=
result=PASS|FAIL
capture_trigger=PB8_TIM4_CH3
capture_ack=PB9_TIM4_CH4
wiring_profile=single_board_split_capture_v1
measured_path=PB9_PA1_minus_PB8_PA6
```

## Retained Artifact

Capture and retain the report:

```sh
python3 scripts/hil_timing_capture.py \
  --serial /dev/ttyACM0 \
  --out artifacts/hil_timing/<run_id>
```

The retained directory contains:

```text
artifacts/hil_timing/<run_id>/
  timing_report.txt
  meta.json
  wiring.txt
```

`timing_report.txt` contains the raw firmware report. `wiring.txt` records:

```text
PA6/D12 -> PA0/A0
PA6/D12 -> PB8/TIM4_CH3
PA1/A1  -> PB9/TIM4_CH4
GND shared
```

`meta.json` records the feature set, wiring profile, measurement path, timer
settings, capture pins, functional pins, and honest `PASS` or `FAIL` result.

## Risks

- TIM4 is 16-bit; all delta math must use wrapping `u16` subtraction.
- TIM4 CH3/CH4 can be pending in the same ISR and must be ordered explicitly.
- TIM4 capture interrupts must be disabled before finalizing the report.
- PB8/PB9 are Arduino I2C pins; attached pullups or shields can disturb timing.
- Wiring skew is part of the selected measurement path.
