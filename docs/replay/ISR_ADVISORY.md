# ISR Advisory — Phase A Findings

Findings from Phase A sprint (`mvp/phaseA`) review of `replay-fw-f446` TIM2 ISR.

## Finding A — Status-register hardening

**Issue:** `clear_tim2_update_flag()` used `sr().modify()` (read-modify-write) on the TIM2 SR register, which has `rc_w0` (read-clear-by-writing-zero) semantics.

**Risk:** Latent hazard. In the current TIM2-only configuration (only UIF active), this is functionally safe. However:
- `modify()` reads SR, clears UIF, writes back — if other SR flags become relevant, they would be silently cleared.
- On Cortex-M, the RMW can theoretically race with NVIC pending-bit de-assertion, though at 1 kHz ISR rate this is unlikely to manifest.

**Resolution:** Changed to `sr().write(|w| w.uif().clear_bit())`, which writes `0` to UIF (clearing it) and `0` to all other SR bits. Safe because only UIF is active in the current configuration.

This hardening is valid for the current configuration, not a blanket rule for arbitrary future TIM2 flag use.

**Ref:** `crates/replay-fw-f446/src/fw.rs`, `clear_tim2_update_flag()`

## Finding B — Phase-ordering contract normalization

**Issue:** ISR used increment-before-read phase accumulator ordering, producing `sample = (frame_idx + 1) & 0xFF`. Firmware and tooling were aligned on this convention.

**Decision:** Normalize to read-then-advance ordering to produce `sample = frame_idx & 0xFF` per the frozen phase8 contract.

| convention | formula |
|---|---|
| **prior** | `sample = (frame_idx + 1) & 0xFF` |
| **frozen** | `sample = frame_idx & 0xFF` |

The accumulator is a 32-bit phase register (`u32`) with `STEP = 0x0100_0000`. Extracting `phase >> 24` yields the high byte as the phase8 sample.

**Resolution:** Reordered ISR to read phase before advancing. Updated `read_artifact.py` `expected_sample_for_model()` to match.

**Ref:** `crates/replay-fw-f446/src/fw.rs`, `tim2_isr()`; `scripts/read_artifact.py`, `expected_sample_for_model()`
