# replay-fw-f446

Bare-metal STM32F446RE capture firmware for the active released replay path.

Release classification is owned by `docs/RELEASE_SURFACE.md`. The active
capture contract is `docs/replay/FW_F446_CAPTURE_v1.md`, and retained release
evidence for release `1.2.2` lives under `docs/verification/releases/1.2.2/`.

## Current capture path

The firmware captures deterministic frames from the TIM2 ISR and emits an RPL0
format version 1 artifact over USART2 after capture halts. Replay semantics remain
the legacy 16-byte `EventFrame0` interpretation documented in the capture
contract.

## Behavior

- Capture source: TIM2 update interrupt at nominal 1 kHz.
- Frame count: 10,000.
- Capture phase: TIM2 ISR writes deterministic placeholder samples into static buffer.
- Dump phase: TIM2 interrupt disabled, global interrupts disabled, then raw artifact bytes written over USART2 TX using polling.

## Host capture

Validated manual-reset operator path for the self-stimulus CSV flow is documented in
`docs/debug/reset_run_characterization.md`.

Typical Linux capture command:

```bash
python3 scripts/csv_capture.py --serial /dev/ttyACM0 --out observed.csv --reset-mode manual
```

Notes:
- Manual reset is canonical for UART capture on this flow.
- Success prints `STATE,CAPTURE_DONE,138` before CSV output.
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
