# STM32 Replay Operator Procedure

## Grounding
- Project: Precision Signal
- Core capability: deterministic execution validation via replay
- Interface: `precision` CLI
- Path: STM32 over UART

## Operator Procedure

CRITICAL PRECONDITION

`PA6` (`TIM3_CH1`) -> `PA0` (`TIM2_CH1`) loopback must be present.
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
4. Run `python3 scripts/csv_capture.py --serial /dev/ttyACM0 --out observed.csv --reset-mode manual`.
5. Confirm capture emits a `STATE,...` line, writes `observed.csv`, and `observed.csv` begins with `index,interval_us` and contains rows.
6. Run `cargo run -q -p dpw4 --features cli --bin precision -- record observed.csv --mode runtime_mode`.
7. Confirm `precision record` reports `RESULT: PASS`, `EQUIVALENCE: exact`, and `FIRST_DIVERGENCE: none`, then record the printed `ARTIFACT: artifacts/<record_run_id>`.
8. Run `cargo run -q -p dpw4 --features cli --bin precision -- replay artifacts/<record_run_id> --mode runtime_mode`.
9. Confirm `precision replay` reports `RESULT: PASS`, `EQUIVALENCE: exact`, and `FIRST_DIVERGENCE: none`.

## Failure Signatures
- No STATE preamble
  Meaning: no `STATE,...` line was observed within the capture timeout.
  First check: confirm any `STATE,...` line appears on `/dev/ttyACM0`.
- `CAPTURE_INCOMPLETE,0`
  Meaning: the first transport state line is `STATE,CAPTURE_INCOMPLETE,0`; `0` corresponds to zero recorded intervals.
  First check: check the physical loopback from `PA6` (`TIM3_CH1`) to `PA0` (`TIM2_CH1`).
