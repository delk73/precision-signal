# Diagnostic Features

This document covers diagnostic-only instrumentation and debug-oriented build
surfaces. These features are not part of the canonical release or capture path.

## `debug-irq-count`

- surface: `replay-fw-f446`
- default: disabled
- purpose: expose deterministic evidence that TIM2 ISR execution continues during
  capture

Implementation notes:

- exports `IRQ_COUNT` as `#[no_mangle] pub static AtomicU32`
- resets the counter before TIM2 IRQ enable
- increments the counter at TIM2 ISR entry

Build command:

```bash
cargo build -p replay-fw-f446 --features debug-irq-count
```

Flash path:

```bash
make FW_FEATURES=debug-irq-count flash-ur
make FW_FEATURES=debug-irq-count flash-compare-ur
```

## Operational Boundary

- diagnostic features are for observability only
- they do not define replay artifact semantics
- they are outside the canonical release/capture workflow

For reset, attach, failure signatures, and recovery guidance, use
[docs/debug/reset_run_characterization.md](reset_run_characterization.md).
