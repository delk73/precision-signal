# replay-fw-f446-timing

Bare-metal STM32F446RE timing characterization firmware.

This crate implements the TIM2 input-capture / TIM3 PWM self-stimulus path that
measures 138 timing intervals and emits them as a text CSV over USART2. It is a
narrower, separate use case from the RPL0 capture firmware — it characterizes
MCU timing jitter and does not produce an RPL file.

The active capture contract for this crate is
`docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md`.

Release classification is owned by `docs/RELEASE_SURFACE.md`.

The RPL0-emitting firmware (direct hardware-backed capture) lives in
`crates/replay-fw-f446`.

## Behavior

- Capture source: TIM2 input-capture on PA0 (TIM2_CH1).
- Stimulus source: TIM3 PWM on PA6 (TIM3_CH1).
- Interval count: 138.
- Capture phase: TIM2/TIM3 self-stimulus path records interval measurements
  into a fixed in-memory buffer.
- Dump phase: capture halts, firmware emits `STATE,CAPTURE_DONE,138` on
  success, then writes the canonical `index,interval_us` CSV body over
  USART2 TX using polling.

## Physical loopback

This path requires a physical loopback from PA6 to PA0:

- stimulus source: `TIM3_CH1` on `PA6`
- capture input: `TIM2_CH1` on `PA0`

If the loopback is missing, the firmware can boot and UART can work while
capture ends with `STATE,CAPTURE_INCOMPLETE,0`.

## Host capture

Validated manual-reset operator path:

```bash
python3 scripts/csv_capture.py --serial /dev/ttyACM0 --out observed.csv --reset-mode manual
```

## Debug IRQ counter (optional)

Enable feature `debug-irq-count` to expose a debugger-visible TIM2 ISR counter:

```bash
cargo build -p replay-fw-f446-timing --features debug-irq-count
```
