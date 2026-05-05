# replay-fw-f446

Bare-metal STM32F446RE firmware for the hardware-backed RPL0 capture path.

Captures 10,000 interrupt-driven execution samples via TIM2 update interrupt and
emits them as a binary RPL0 v1 file over USART2. Every `timer_delta` is the
nominal constant `1000` (one TIM2 period at PSC=15, ARR=999); every `input_sample`
is a real phase accumulator output. No host-side transformation step is involved.

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
make rpl0-replay-check
```

Or invoke the capture step directly:

```bash
PYTHONPATH="$PWD" python3 scripts/artifact_tool.py capture --out artifacts/run.rpl --signal-model phase8
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

