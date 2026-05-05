# replay-fw-f446

Bare-metal STM32F446RE firmware for the hardware-backed RPL0 capture path.

Captures 10,000 interrupt-driven execution samples via TIM2 update interrupt and
emits them as a binary RPL0 v1 file over USART2. Every `timer_delta` is a real
TIM2 counter reading; every `input_sample` is a real phase accumulator output.
No host-side transformation step is involved.

Release classification is owned by `docs/RELEASE_SURFACE.md`. The active capture
contract is `docs/replay/FW_F446_CAPTURE_v1.md`. Retained release evidence for
release `1.2.2` lives under `docs/verification/releases/1.2.2/`.

For the timing characterization firmware (TIM2 input-capture, 138 intervals,
CSV output), see `crates/replay-fw-f446-timing`.

## Behavior

- Capture source: TIM2 update interrupt at nominal 1 kHz (PSC=15, ARR=999).
- Sample count: 10,000.
- Signal model: `phase8` — `(phase >> 24) as i32`, where phase advances by
  `STEP = 0x0100_0000` per interrupt.
- Capture phase: ISR records samples into a static `[i32; 10_000]` buffer via
  Cortex-M critical sections.
- Dump phase: TIM2 IRQ disabled, global interrupts disabled, firmware emits
  `[HEADER][SCHEMA BLOCK][FRAME DATA]` over USART2 TX using polling.
- Output format: RPL0 v1 (`version = 1`, `header_len = 0x98`).

## Host capture

```bash
python3 scripts/read_artifact.py --out artifacts/run.rpl --signal-model phase8
```

## Demo perturbation modes

Feature flags for divergence analysis demonstration:

- `demo-divergence`: perturbs one sample at frame 4096 (single-frame offset).
- `demo-persistent-divergence`: shifts phase state at frame 4096 (all following
  samples remain shifted). Mutually exclusive with `demo-divergence`.

## Debug IRQ counter (optional)

Enable feature `debug-irq-count` to expose a debugger-visible TIM2 ISR counter:

```bash
cargo build -p replay-fw-f446 --features debug-irq-count
```

In GDB, inspect with `p IRQ_COUNT_PROBE`.


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

## Physical loopback

This STM32 self-stimulus path is not an internal timer-to-timer route. It
requires a physical loopback from the configured stimulus output pin to the
configured capture input pin:

- stimulus source: `TIM3_CH1` on `PA6`
- capture input: `TIM2_CH1` on `PA0`

If the `PA6 -> PA0` loopback is missing or landed on the wrong pin, the
firmware can boot, UART can work, and the stimulus square wave can be visible
on scope while capture still ends with `STATE,CAPTURE_INCOMPLETE,0`.

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
