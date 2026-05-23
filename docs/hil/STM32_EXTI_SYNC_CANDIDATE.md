# STM32 EXTI Sync Candidate

## Purpose

Single-STM32 bring-up and HIL validation notes for the EXTI acknowledgment candidate.

This is a measurement candidate, not a guaranteed `< 100 ns` solution.

## Build

```sh
FW_FEATURES="self_stim" make fw
FW_FEATURES="self_stim" make flash
```

## Wiring

```text
PA0 = trigger input
PA1 = acknowledgment output
GND = shared with trigger source and scope
```

## Meaning

```text
PA1 short pulse      = EXTI acknowledgment
PA1 repeating blink  = clock initialization fault
```

## Single-board bring-up

```text
1. Flash self_stim.
2. Confirm PA1 is not blinking.
3. Confirm UART decode still works.
4. Confirm TIM2 1 kHz capture still works.
5. Drive PA0 with a slow rising edge.
6. Confirm PA1 emits a short pulse.
```

## HIL gate

```text
Trigger frequency: 10 kHz
Window: 10,000 continuous triggers
Pass: zero missed PA1 pulses and max PA0-to-PA1 latency < 100 ns
Fail: any missed pulse or max latency >= 100 ns
```

## Run log

```text
commit:
feature:
flash command:
board:
trigger source:
scope:
max latency:
missed pulses:
UART decode:
TIM2 cadence:
result:
notes:
```

## Failure rule

Do not relax the 100 ns threshold. If EXTI fails, keep the run log and move next to timer Input Capture / Output Compare.
