# STM32 EXTI Sync Candidate

## Purpose

Single-STM32 bring-up and HIL validation notes for the EXTI acknowledgment candidate.

This is a measurement candidate, not a guaranteed `< 100 ns` solution.

## Build

```sh
FW_FEATURES="sync_trigger_out sync_trigger_in" make fw
FW_FEATURES="sync_trigger_out sync_trigger_in" make flash-ur
```

## Wiring

```text
PA6/D12 = trigger output
PA0/A0  = trigger input
PA1/A1  = acknowledgment output
GND     = shared with scope/source
```

## Single-board loopback

```text
PA6/D12 -> PA0/A0
```

## Meaning

```text
PA6/D12 pulse       = trigger output
PA0/A0 rising       = trigger input observed
PA1/A1 short pulse  = acknowledgment output
PA1/A1 repeating blink = clock failed
```

## Single-board bring-up

```text
1. Flash sync_trigger_out sync_trigger_in.
2. Jumper PA6/D12 to PA0/A0.
3. Confirm PA1/A1 is not blinking.
4. Confirm UART decode still works.
5. Confirm TIM2 1 kHz capture still works.
6. Confirm PA0/A0 shows the PA6/D12 trigger pulse.
7. Confirm PA1/A1 emits a short pulse after PA0/A0 rises.
```

## Scope check order

Use this order when the scope result is unclear.

```text
1. Confirm RPL0 capture completes.
   - This proves UART/RPL0 output and TIM2 capture are alive.
   - It does not prove PA6/PA0/PA1 sync I/O.

2. Probe PA6/D12 directly.
   - Expected: throttled trigger pulse during capture.
   - If PA6 is flat, debug trigger output or flashed feature set.

3. Probe PA0/A0 with PA6/D12 jumpered to PA0/A0.
   - Expected: PA0 shows the PA6-driven trigger pulse.
   - If PA6 pulses but PA0 is flat, debug jumper or pin mapping.

4. Probe PA0/A0 and PA1/A1 together.
   - CH1: PA0/A0
   - CH2: PA1/A1
   - Trigger on CH1 rising edge.
   - Expected: PA1 emits a short acknowledgment pulse after PA0 rises.
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

## Bring-up result notes

Record these separately:

```text
RPL0 capture: pass/fail
TIM2 delta: 1000/other
PA6 trigger output observed: yes/no
PA0 loopback trigger observed: yes/no
PA1 acknowledgment pulse observed: yes/no
notes:
```

## Failure rule

Do not relax the 100 ns threshold. If EXTI fails, keep the run log and move next to timer Input Capture / Output Compare.
