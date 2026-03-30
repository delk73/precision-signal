# NON-NORMATIVE / EXPERIMENTAL

This note is exploratory only. It does not define current release surface,
verification authority, or normative invariants.

## 2026-03-30 — Pi emitter physical validation [WIP-PI-EMITTER-001]
Status: blocked
Owner: signal

Problem
The `demo-signal-first-divergence` lane requires first physical validation of the
Pi emitter on `GPIO17`, but physical waveform presence and interval shape are not
yet established in this workspace session.

Hypothesis
If the Pi emitter is run on a Raspberry Pi with the expected `python3 + gpiod`
runtime and a scope or logic analyzer attached to `GPIO17`, the baseline run
should emit a square wave with a `10 x 200 us` preamble followed by `1000 us`
intervals, and the perturb run should show one `1500 us` interval at frame `50`.

Constraints
- Do not change firmware, replay tooling, host capture scripts, architecture docs, or release docs in this pass.
- Do not promote any observation from this note into normative verification authority.
- Do not substitute script execution or inference for physical observation on `GPIO17`.
- Preserve the current signal contract: source pin `GPIO17`, square wave, `10 x 200 us` preamble, steady `1000 us`, perturb frame `50 = 1500 us`.

Evidence Produced
- Starting point recorded: Pi emitter path was reported runnable without backend failure, but physical waveform on `GPIO17` was not yet confirmed.
- Environment distinction:
- Workspace (non-Pi): missing `gpiod`, not representative
- Actual Pi (`syn-core-pi3b`): `gpiod` present, emitter executed
- Command run: `make demo-signal-pi-baseline`
- Observed result (workspace, non-Pi environment): preflight failed due to missing Python `gpiod` bindings; this does not represent the actual Pi runtime.
- Observed result (actual Pi): preflight passed and `scripts/pi_emitter.py` executed without backend failure; physical waveform not yet verified.
- Command run: `make demo-signal-pi-perturb`
- Observed result (workspace, non-Pi environment): preflight failed due to missing Python `gpiod` bindings; this does not represent the actual Pi runtime.
- Observed result (actual Pi): preflight passed and `scripts/pi_emitter.py` executed without backend failure; physical waveform not yet verified.
- Instrumentation note: no oscilloscope or logic-analyzer channel was available to this agent session, so no physical observation was made on `GPIO17`.
- Proven in this session: the current workspace Make targets gate on `python3`, Python `gpiod`, and `/dev/gpiochip0` before invoking `scripts/pi_emitter.py`.
- Blocked on physical instrumentation only; Pi runtime path is confirmed to execute.
- Not yet proven: waveform present on `GPIO17`.
- Not yet proven: first `10` intervals are approximately `200 us`.
- Not yet proven: remaining baseline intervals are approximately `1000 us`.
- Not yet proven: perturb run contains exactly one approximately `1500 us` stretched interval.
- Not yet proven: stretched interval occurs at frame `50`.
- Not yet proven: timing jitter is acceptable for the demo path.
- Not yet proven: host plus STM32 capture path behavior.

Next Decision
BLOCKED until validation is rerun on the actual Pi with a scope or logic analyzer attached to `GPIO17`; the narrowest current blocker is physical instrumentation, so do not proceed to host plus STM32 capture until physical Pi-only evidence exists.
