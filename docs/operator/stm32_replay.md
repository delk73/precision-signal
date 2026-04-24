# STM32 Replay Operator Procedure

## Grounding
- Project: Precision Signal
- Core capability: deterministic execution validation via replay
- Interface: `precision` CLI
- Path: STM32 over UART

## Operator Procedure

CRITICAL PRECONDITION

Loopback must be installed between:

- PA6 (`TIM3_CH1`) → PA0 (`TIM2_CH1`)

On boards with Arduino-style silk labels, this corresponds to:

- D12 → A0

If incorrect: `CAPTURE_INCOMPLETE,0` will occur.

Preconditions:
- attached STM32 target over UART
- active UART device `/dev/ttyACM0`
- `make`, `python3`, and `cargo` are available in `PATH`
- `precision` mode: `runtime_mode`
- artifact directories are generated under `artifacts/` at runtime

1. Confirm the attached STM32 target is the exercised board for this path, the `PA6` -> `PA0` loopback is present, and the active UART device is `/dev/ttyACM0`.
2. Run `make flash-ur` and wait for it to succeed.
3. Run `make flash-compare-ur` and wait for it to succeed.
4. Run `python3 scripts/csv_capture.py --serial /dev/ttyACM0 --out observed.csv --reset-mode manual`, then perform the required manual reset promptly so capture begins before the timeout window expires.
5. Confirm capture emits a `STATE,...` line, writes `observed.csv`, and `observed.csv` begins with `index,interval_us` and contains rows. If reset is delayed too long, capture may time out before any `STATE,...` line appears.
6. Run `cargo run -q -p dpw4 --features cli --bin precision -- record observed.csv --mode runtime_mode`.
7. Confirm `precision record` reports `RESULT: PASS`, `EQUIVALENCE: exact`, and `FIRST_DIVERGENCE: none`, then record the printed `ARTIFACT: artifacts/<record_run_id>`.
8. Run `cargo run -q -p dpw4 --features cli --bin precision -- replay artifacts/<record_run_id> --mode runtime_mode`.
9. Confirm `precision replay` reports `RESULT: PASS`, `EQUIVALENCE: exact`, and `FIRST_DIVERGENCE: none`.

## Failure Signatures
- No STATE preamble
  Meaning: no `STATE,...` line was observed within the capture timeout.
  First check: confirm the manual reset was performed promptly after starting capture and that any `STATE,...` line appears on `/dev/ttyACM0`.
- `CAPTURE_INCOMPLETE,0`
  Meaning: the first transport state line is `STATE,CAPTURE_INCOMPLETE,0`; `0` corresponds to zero recorded intervals.
  First check: check the physical loopback from `PA6` (`TIM3_CH1`) to `PA0` (`TIM2_CH1`).

## Physical Replay Characterization Prep

Purpose: prepare bounded STM32 physical replay characterization for future
voltage / power-floor evidence capture while preserving the authoritative
`precision record` / `precision replay` replay path.

Setup boundary:
- use the same STM32-over-UART replay procedure and loopback precondition above
- vary only the documented voltage or power condition for a discrete sweep point
- keep the replay artifact path and hash tied to the sweep point
- record power provenance separately from execution provenance

High-level operator flow:
1. Establish the baseline setup and confirm the normal STM32 replay procedure.
2. Select one discrete voltage or power condition.
3. Capture the STM32 UART output and any power observation for that condition.
4. Run `precision record` on a complete capture when one exists.
5. Run `precision replay` on the resulting artifact directory when `record`
   produced one.
6. Classify the result using the procedure vocabulary in
   [hardware_procedures.md](../verification/hardware_procedures.md).
7. Preserve provenance for the replay artifact, capture artifact, power
   observation source, result classification, equivalence classification, first
   divergence if any, capture completeness, UART/log completeness, and operator
   notes.

This procedure does not expand the authoritative release surface and does not
establish full power-envelope correctness. See
[hardware_procedures.md](../verification/hardware_procedures.md) for full
characterization boundaries, non-claims, and provenance requirements.
