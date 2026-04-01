# replay-fw-f446

Bare-metal STM32F446RE capture firmware for the active released replay path.

Release classification is owned by `docs/RELEASE_SURFACE.md`. The active
capture contract is `docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md`. The retained
historical RPL0 artifact-capture description lives in
`docs/replay/FW_F446_CAPTURE_v1.md`, and retained release evidence for release
`1.2.2` lives under `docs/verification/releases/1.2.2/`.

## Current capture path

The active self-stimulus operator path emits a UART success preamble followed by
interval CSV over USART2 after capture halts. The canonical downstream file is
the CSV payload body captured by `scripts/csv_capture.py`, not the `STATE,...`
transport line.

## Behavior

- Capture source: TIM2 update interrupt at nominal 1 kHz.
- Interval count: 138.
- Capture phase: TIM2/TIM3 self-stimulus path records interval measurements into
  a fixed in-memory buffer.
- Dump phase: capture halts, firmware emits `STATE,CAPTURE_DONE,138` on success,
  then writes the canonical `index,interval_us` CSV body over USART2 TX using
  polling.

## Host capture

Validated manual-reset operator path for the self-stimulus CSV flow is documented in
`docs/debug/reset_run_characterization.md`.

Typical Linux capture command:

```bash
python3 scripts/csv_capture.py --serial /dev/ttyACM0 --out observed.csv --reset-mode manual
```

Notes:
- Manual reset is canonical for UART capture on this flow.
- `STATE,CAPTURE_DONE,138` is the transport-level success condition.
- The canonical downstream file is the CSV payload only.
- `--reset-mode stlink` is present in tooling but not validated as reliable in this characterization.

## Debug IRQ counter (optional)

Enable feature `debug-irq-count` to expose a debugger-visible TIM2 ISR execution counter:

```bash
cargo build -p replay-fw-f446 --features debug-irq-count
```

With this feature enabled:
- `IRQ_COUNT` is exported with `#[no_mangle]` for symbol visibility.
- `tim2_isr()` increments `IRQ_COUNT` on each interrupt entry.
- `fw_main()` resets `IRQ_COUNT` to zero before TIM2 IRQ enable.

In GDB, inspect with:

```gdb
p IRQ_COUNT
```

Interpretation:
- `0`: TIM2 ISR has not executed.
- Increasing values: TIM2 ISR is firing repeatedly.
- Value stops increasing: ISR progress has stalled.
