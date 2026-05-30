# STM32 Sync Timing Capture

## Purpose

Retained HIL evidence for STM32 split-capture timing profiles.

Two firmware roles are supported:

```text
single-board actor+observer:
  emits PA6
  consumes PA0 through TIM2
  emits PA1 through TIM2
  captures PA6/PA1 through PB8/PB9 TIM4

observer-only:
  captures external trigger/ack edges through PB8/PB9 TIM4
  emits timing report only
```

The measured value is:

```text
PB9 capture of PA1/A1 acknowledgment - PB8 capture of PA6/D12 trigger
```

This is observed timing through the selected measurement wiring. It is not exact
internal PA0-to-PA1 silicon latency.

## Single-Board Wiring

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

## Observer Wiring

Observer-only firmware does not drive PA6, does not configure PA0/TIM2 as a
functional input path, and does not drive PA1. It only observes external edges:

```text
external actor trigger edge -> observer PB8/TIM4_CH3
external actor ack edge     -> observer PB9/TIM4_CH4
GND shared
```

`dual_edge_timing_observer_v1` identifies only the observer firmware/profile. It
does not identify a complete dual-board topology or BBB orchestration loop.

## Firmware

```sh
make fw
FW_FEATURES="sync_trigger_out sync_trigger_in" make fw
FW_FEATURES="sync_trigger_out sync_trigger_in sync_timing_capture" make fw
FW_FEATURES="sync_trigger_out sync_trigger_in sync_timing_capture" make flash-ur
FW_FEATURES="sync_timing_observer" make fw
```

`sync_timing_capture` emits only the `SYNC_TIMING_CAPTURE_V1` text report over
USART2. It does not emit RPL0 in the same run and does not change RPL0 format,
replay/diff behavior, `precision` CLI behavior, `sync_trigger_out`,
`sync_trigger_in`, or the strict `< 100 ns` threshold.

`sync_timing_observer` also emits only `SYNC_TIMING_CAPTURE_V1` over USART2. It
does not emit RPL0 and does not enable actor-side PA6/PA0/PA1 behavior.

TIM4 is configured as a free-running 16-bit capture timer at 90 MHz:

```text
PB8/TIM4_CH3 captures PA6 rising edge
PB9/TIM4_CH4 captures PA1 rising edge
PSC=0
ARR=0xffff
threshold_ticks=9
```

After `trigger_count` reaches 10,000, firmware stops generating PA6 pulses but
keeps the acknowledgment path and TIM4 capture interrupts enabled for a bounded
grace interval. This lets the final PA0-triggered PA1 acknowledgment and
PB9/TIM4_CH4 capture arrive before the final drain, shutdown, pending-trigger
accounting, and report.

In single-board mode, `trigger_count` means generated PA6 trigger pulses. In
observer-only mode, `trigger_count` means observed PB8/TIM4_CH3 trigger
captures. Observer-only mode stops after 10,000 observed trigger captures and
then uses the same bounded grace, drain, finalization, and report flow.

Evidence-window arming for future dual-board observer work is captured in
[Dual-Board Observer Evidence-Window Arming Design](DUAL_BOARD_OBSERVER_EVIDENCE_WINDOW_ARMING.md).
The implemented observer report keeps raw full-run counters separate from the
declared evidence-window counters.

Observer-only pairing is count-based. If a new PB8/TIM4_CH3 trigger capture
arrives while an earlier trigger timestamp is still unpaired, firmware replaces
the pending timestamp. It does not increment `missed_ack_count` immediately;
finalization derives misses once as `trigger_count - paired_ack_count`.

`max_delta_ticks` is authoritative. `max_delta_ns` is display-only. Pass is valid
for the raw full-run `result` only when:

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
The boundary diagnostic fields split `unexpected_ack_count` into acknowledgments
before the first accepted trigger, acknowledgments inside the 10,000-trigger
window that could not be paired one-to-one, and acknowledgments after the final
accepted trigger before report finalization. These fields explain a failure;
they do not change PASS policy. `unexpected_ack_count` must equal:

```text
pre_first_trigger_ack_count
+ in_window_unexpected_ack_count
+ post_final_trigger_ack_count
```

`first_in_window_unexpected_ack_trigger_count` and
`last_in_window_unexpected_ack_trigger_count` locate in-window unexpected
acknowledgments relative to the accepted trigger stream. Zero means no in-window
unexpected acknowledgment was observed. For a single in-window unexpected
acknowledgment, first and last are equal. These fields are diagnostic-only and
do not relax PASS semantics; any nonzero `unexpected_ack_count` remains
`result=FAIL`.

`capture_error_count` records TIM4 CH3/CH4 overcapture flags; any nonzero value
means at least one capture event was overwritten before firmware handled it.

Raw full-run counters report all observed startup and evidence-window behavior.
`evidence_window_result` applies only to the declared evidence window, which
starts at `evidence_window_start_trigger_count=8`. A PASS evidence-window result
does not mean the raw full run had no startup transients. Any nonzero raw
`unexpected_ack_count` still makes raw `result=FAIL`.

## Report

```text
SYNC_TIMING_CAPTURE_V1
timer_hz=90000000
threshold_ticks=9
trigger_count=10000
ack_count=
missed_ack_count=
unexpected_ack_count=
pre_first_trigger_ack_count=
in_window_unexpected_ack_count=
first_in_window_unexpected_ack_trigger_count=
last_in_window_unexpected_ack_trigger_count=
post_final_trigger_ack_count=
capture_error_count=
max_delta_ticks=
max_delta_ns=
result=PASS|FAIL
evidence_window_start_trigger_count=8
evidence_window_trigger_count=
evidence_window_ack_count=
evidence_window_unexpected_ack_count=
evidence_window_missed_ack_count=
evidence_window_capture_error_count=
evidence_window_max_delta_ticks=
evidence_window_max_delta_ns=
evidence_window_result=PASS|FAIL
capture_trigger=PB8_TIM4_CH3
capture_ack=PB9_TIM4_CH4
wiring_profile=single_board_split_capture_v1
measured_path=PB9_PA1_minus_PB8_PA6
```

Observer-only firmware emits the same V1 fields but uses:

```text
wiring_profile=dual_edge_observer_v1
```

## Retained Artifact

Capture and retain the report:

```sh
python3 scripts/hil_timing_capture.py \
  --profile single_board_tim2_hardware_ack_v1 \
  --serial /dev/ttyACM0 \
  --out artifacts/hil_timing/<run_id>

python3 scripts/hil_timing_capture.py \
  --profile dual_edge_timing_observer_v1 \
  --serial /dev/ttyACM0 \
  --out artifacts/hil_timing/<run_id>
```

Generate a retained artifact from an existing raw report without live serial
capture:

```sh
python3 scripts/hil_timing_capture.py \
  --profile single_board_tim2_hardware_ack_v1 \
  --input artifacts/hil_timing/0004/timing_report.txt \
  --out /tmp/hil_timing_profile_check
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

For `dual_edge_timing_observer_v1`, `wiring.txt` records:

```text
external actor trigger edge -> observer PB8/TIM4_CH3
external actor ack edge     -> observer PB9/TIM4_CH4
GND shared
```

`meta.json` records the feature set, wiring profile, timer settings, capture
pins, functional pins, measured values, and honest `PASS` or `FAIL` result.
`--profile` is required for new captures because the raw
`SYNC_TIMING_CAPTURE_V1` report does not encode the acknowledgment mechanism.
The selected profile supplies `evidence_profile`, `run_profile`,
`functional_path`, `measurement_path`, and `claim_boundary`:

```json
{
  "evidence_profile": "single_board_tim2_hardware_ack_v1",
  "run_profile": "tim2_hardware_ack",
  "functional_path": {
    "trigger_output": "PA6_D12",
    "trigger_input": "PA0_A0",
    "ack_mechanism": "tim2_hardware_output_compare",
    "ack_output": "PA1_A1"
  },
  "measurement_path": {
    "trigger_capture": "PB8_TIM4_CH3",
    "ack_capture": "PB9_TIM4_CH4",
    "measured_delta": "ack_capture_minus_trigger_capture"
  },
  "claim_boundary": {
    "proves": "selected TIM2 hardware acknowledgment path split-capture timing",
    "does_not_prove": [
      "software EXTI acknowledgment path timing pass",
      "exact internal PA0-to-PA1 silicon latency"
    ]
  }
}
```

The flat fields `measured_path`, `capture_trigger`, `capture_ack`,
`trigger_output`, `trigger_input`, and `ack_output` remain for compatibility.
`timing_report.txt` remains the raw `SYNC_TIMING_CAPTURE_V1` firmware report;
the report does not encode the acknowledgment mechanism.

## Dual-Board Observer Artifact

Dual-board observer evidence is retained under a separate namespace:

```text
artifacts/hil_timing_dual/<run_id>/
  timing_report.txt
  meta.json
  wiring.txt
  run_context.json
  notes.txt
```

Do not edit generated files after capture:

```text
timing_report.txt
meta.json
wiring.txt
```

For `dual_edge_timing_observer_v1`, generated `wiring.txt` remains the
observer-profile wiring:

```text
external actor trigger edge -> observer PB8/TIM4_CH3
external actor ack edge     -> observer PB9/TIM4_CH4
GND shared
```

`meta.json` is observer artifact metadata. `run_context.json` is the dual-board
topology and procedure context. `notes.txt` records operator interpretation,
bench details, board identity, serial path, ST-LINK/VCP ambiguity, reset
ordering, and result classification.

Before retained dual-board capture, `run_context.json` must bind logical board
roles to stable USB/ST-LINK identities under `board_aliases.actor` and
`board_aliases.observer`. Each alias requires `stlink_serial`, `vcp_by_id`,
`firmware_features`, and `role`; the live capture command must use
`board_aliases.observer.vcp_by_id` as `--serial`.

`run_context.json` is manual context and may be updated before capture after
bench enumeration. Confirm the aliases with:

```sh
ls -l /dev/serial/by-id/
```

Expected bench mappings:

```text
066CFF505487525067182651 -> actor / Board A
0668FF514988525067215029 -> observer / Board B
```

After confirming those mappings from `/dev/serial/by-id/`, set
`board_alias_confirmation.confirmed_from_dev_serial_by_id=true` in
`run_context.json`. The confirmation block must also retain
`required_mappings` with the actor ST-LINK serial mapped to `actor / Board A`
and the observer ST-LINK serial mapped to `observer / Board B`.

Do not use `ttyACM0`/`ttyACM1` ordering as board identity, and do not use
post-flash `st-info` as observer board-state authority once observer firmware is
running.

For the `dual_board_tim2_hardware_ack_observed_v1` topology, Board A is the
actor and Board B is the observer:

```text
Board A PA6/D12 -> Board A PA0/A0
Board A PA6/D12 -> Board B PB8/TIM4_CH3
Board A PA1/A1  -> Board B PB9/TIM4_CH4
Board A GND     -> Board B GND
```

Board A still needs the local `PA6/D12 -> PA0/A0` functional loop because the
actor TIM2 hardware acknowledgment path is triggered from PA0/TIM2_CH1. Board B
does not need PA0 or PA1 connected; it only observes PB8/PB9 plus shared ground.

### Dual-board observer retained run

1. Confirm board aliases in `run_context.json`.
2. Use by-id serials, not `ttyACM` ordering.
3. Run:

   ```sh
   make hil-dual-observer-run RUN=<id>
   ```

4. For scratch validation:

   ```sh
   make hil-dual-observer-scratch RUN=<id>
   ```

5. Inspect raw `result` and `evidence_window_result` separately.

Raw result reports all observed startup and evidence-window behavior.
`evidence_window_result` applies only to the declared evidence window. A raw
`FAIL` with `evidence_window_result=PASS` is valid and intentional.

### ST-LINK attach recovery note

The standard operator path remains:

```sh
STFLASH_SERIAL=<serial> FW_FEATURES="<features>" make flash-ur
```

The Makefile under-reset flash path uses `STFLASH_FREQ ?= 200`, emitted as
`--freq=200`. This numeric kHz form is the supported compatibility spelling for
the observed ST-LINK `st-flash` 1.7.0 dev-host path and the BBB `st-flash`
1.8.0 bench path; do not use the older `--freq=200K` spelling in under-reset
flash/read/reset commands.

On this bench, an STM32 board may occasionally fail the first `make flash-ur`
attempt after being disconnected or after entering a bad attach/run state. The
observed recovery is:

1. Hold the board RESET button down.
2. Plug the board USB in while RESET is held.
3. Start the same `make flash-ur` command.
4. Release RESET during/after the first failed attach attempt, then rerun the
   same `make flash-ur` command if needed.

Do not replace the Makefile flash path with direct `st-flash` commands. This is
a recovery procedure for ST-LINK attach state only; it is not the normal capture
or reset procedure.

If observer flash fails, `hil-dual-observer-run` exits before starting capture.
Recover the board with the same `make flash-ur` path, then rerun the runner.

The retained result determines the allowed claim. `PASS` means the external
observer measured Board A's TIM2 hardware-ack path under the existing gate.
`FAIL` with clean counters means the external observer produced honest timing
evidence, but the observed path did not satisfy the gate. Timeout or nonzero
integrity counters must be retained/classified as bring-up failure, not timing
success.

## Supported Timing Evidence Profiles

Profile:

```text
single_board_tim2_hardware_ack_v1
```

Feature set:

```text
sync_trigger_out sync_trigger_in sync_timing_capture
```

Wiring profile:

```text
single_board_split_capture_v1
```

Functional path:

```text
PA6/D12 -> PA0/A0 -> TIM2_CH1 -> TIM2 reset/PWM -> TIM2_CH2 -> PA1/A1
```

Measurement path:

```text
PB8/TIM4_CH3 observes PA6/D12
PB9/TIM4_CH4 observes PA1/A1
```

Claim:

This profile supports the split-capture timing claim only for the named TIM2
hardware acknowledgment path.

Non-claims:

It does not prove the EXTI software acknowledgment path passes. It is not exact
internal PA0-to-PA1 silicon latency. It is not, by itself, RPL0/replay
authority or release evidence.

Profile:

```text
dual_edge_timing_observer_v1
```

Feature set:

```text
sync_timing_observer
```

Wiring profile:

```text
dual_edge_observer_v1
```

Functional path:

```text
external actor trigger edge and acknowledgment edge are actor-defined
```

Measurement path:

```text
PB8/TIM4_CH3 observes external actor trigger edge
PB9/TIM4_CH4 observes external actor acknowledgment edge
```

Claim:

This profile supports only the observer-board measurement claim: external actor
trigger-to-ack timing observed through PB8/PB9.

Non-claims:

It does not prove actor internal PA0-to-PA1 silicon latency. It does not prove a
software EXTI acknowledgment path timing pass unless the actor profile states
that separately. It is not platform proof, RPL0/replay authority, or release
evidence.

## Replay Carryforward

The retained timing runs establish that timing evidence must identify both the
functional path and the measurement path.

Runs 0001-0003 measured software acknowledgment paths:

```text
PA6 -> PA0/EXTI0 -> software writes PA1
```

Run 0004 changed the functional acknowledgment path:

```text
PA6 -> PA0/TIM2_CH1 -> TIM2 hardware drives PA1/TIM2_CH2
```

The measurement path stayed constant:

```text
PB8/TIM4_CH3 observes PA6
PB9/TIM4_CH4 observes PA1
```

Replay-facing implication:

Future replay/capture iterations should treat timing artifacts as
path-qualified evidence. A timing result is not just PASS or FAIL; it is a
result for a named `run_profile` and `functional_path`, observed through a
named `measurement_path`, with retained wiring and firmware feature context.

This prevents overclaiming. Run 0004 proves the selected TIM2 hardware-ack path
passes the split-capture timing gate. It does not prove the software EXTI
acknowledgment path passes, and it is not exact internal PA0-to-PA1 silicon
latency.

Replay-facing code should consume the structured `meta.json` fields rather than
freeform notes when carrying this distinction forward.

## Risks

- TIM4 is 16-bit; all delta math must use wrapping `u16` subtraction.
- TIM4 CH3/CH4 can be pending in the same ISR and must be ordered explicitly.
- TIM4 capture interrupts must be disabled before finalizing the report.
- PB8/PB9 are Arduino I2C pins; attached pullups or shields can disturb timing.
- Wiring skew is part of the selected measurement path.
