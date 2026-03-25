# replay-fw-f446

Bare-metal STM32F446RE capture firmware for the active released replay path.

Release classification is owned by `docs/RELEASE_SURFACE.md`. The active
capture contract is `docs/replay/FW_F446_CAPTURE_v1.md`, and retained release
evidence lives under `docs/verification/releases/1.2.1/`.

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

Typical Linux capture command:

```bash
cat /dev/ttyACM0 > run0.bin
```

Notes:
- Output is binary artifact data; avoid terminal tools that transform bytes.
- Stop capture after one full artifact transfer completes.

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
