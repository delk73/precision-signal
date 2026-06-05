# Dual STM32 Replay Witness Profile v1

## Purpose

This profile describes the implemented dual-STM32 replay witness v0 path. A
second STM32F446RE observes the existing actor PA6/PA1 event path and checks the
observed stream against a fixed replay witness rule before emitting a PASS/FAIL
report.

## Roles

- Actor board: Board A, STM32F446RE.
- Witness board: Board B, STM32F446RE.
- Host: flashes both boards, starts witness capture, and stores non-retained
  scratch artifacts.

## Wiring

```text
actor PA6/D12 -> witness PB8/TIM4_CH3
actor PA1/A1  -> witness PB9/TIM4_CH4
actor PA6/D12 -> actor PA0/A0
actor GND     -> witness GND
```

Board A keeps the local PA6-to-PA0 loopback because the existing actor
acknowledgment path is triggered through PA0/TIM2_CH1. Board B observes only
PB8/PB9 plus shared ground.

## Firmware Features

- Actor firmware: `sync_trigger_out sync_trigger_in sync_timing_capture`.
- Witness firmware: `replay_witness_observer`.

The witness feature uses the existing TIM4 PB8/PB9 observer hardware setup but
emits the replay witness report described here. It does not replace the existing
timing-only observer path.

## Stimulus and Observed Outputs

The host flashes the actor quiescent, flashes the witness, starts witness serial
capture, then flashes the actor active. The actor emits the existing
deterministic PA6 trigger path and PA1 acknowledgment path. The witness observes
PA6 as `TRIGGER` on PB8/TIM4_CH3 and PA1 as `ACK` on PB9/TIM4_CH4.

## Replay Witness Rule

Rule ID: `dual_stm32_pa6_pa1_pair_v0`.

`ACTOR_EVENT_COUNT` is the witness-declared expected event-pair count for the
run. For this implementation, the declared full-run count is 10,007 pairs: the
existing actor path emits through the 10,000-trigger evidence window that starts
at trigger 8. The witness checks ordered `TRIGGER, ACK` pairs for that count.
Validation covers event count, order, and channel identity. If the firmware representation
exposes a stable event value, that value is included in validation and digest
input.

`FIRST_INVALID_EVENT` is the first zero-based observed event index where the
stream diverges from the rule. It is `none` for PASS. For FAIL, either
`FIRST_INVALID_EVENT` or `ERROR` identifies the deterministic failure.

`WITNESS_DIGEST` is a fixed-width 16-hex deterministic digest over accepted
observed event data. The implemented digest tuple is
`event_index/channel/timer_delta`; trigger events use `timer_delta=0`, and ACK
events use the observed ACK-minus-TRIGGER timer delta. `none` is valid only when
no event was accepted.

## Report Fields

```text
RESULT: PASS|FAIL
PROFILE: dual_stm32_replay_witness_v1
RULE_ID: dual_stm32_pa6_pa1_pair_v0
ACTOR_EVENT_COUNT: <n>
WITNESS_DIGEST: <16-hex|none>
FIRST_INVALID_EVENT: <index|none>
ERROR: <deterministic text|none>
ACTOR_BOARD: STM32F446RE
WITNESS_BOARD: STM32F446RE
ACTOR_FIRMWARE: sync_trigger_out+sync_trigger_in+sync_timing_capture
WITNESS_FIRMWARE: replay_witness_observer
EVENT_WINDOW: all_observed_events
```

## Artifact Shape

The host writes non-retained scratch artifacts only:

```text
witness_report.txt
meta.json
wiring.txt
run_context.json
```

Do not write this output into retained release evidence directories. Do not
update release summaries or retained evidence for this path.

## Claim

A second STM32F446RE board can observe the existing actor PA6/PA1 replay event
path through PB8/PB9, apply a fixed count/order/channel rule, and emit a
deterministic PASS/FAIL witness report with a compact digest.

## Claim Boundary

This profile does not claim retained release evidence, RPL0 witness semantics,
`precision` CLI replay authority changes, BBB orchestration, platform proof,
security proof, tamper resistance, independent clock-truth proof, or replacement
of the existing timing-only observer path.

## Host Entry Points

```sh
python3 scripts/hil_replay_witness_capture.py --input witness_report.txt --out /tmp/witness_parse
python3 scripts/hil_dual_replay_witness_run.py --run-id <id> --out /tmp/dual_replay_witness_probe --scratch --overwrite-generated
make hil-dual-replay-witness-scratch RUN=<id>
```
