use dpw4::math;
use dpw4::{DpwGain, OscState, SignalShape, DISCONTINUITY_THRESHOLD, SCALE_2_32};

/// Convert a u32 phase to Scalar (radians) using fixed-point only.
/// Inverse of the normative phase_to_u32 mapping used in tick_triangle_dpw4.
fn phase_from_u32(phase_u32: u32) -> dpw4::Scalar {
    // SCALE_2_32 = 2^32 in I64F64; TWO_PI / SCALE_2_32 gives radians per u32 unit.
    dpw4::Scalar::from_num(phase_u32) * (math::TWO_PI / SCALE_2_32)
}

#[test]
fn test_triangle_freeze_tick_keeps_z_constant() {
    let gain = DpwGain::new(1u64 << 63, 0, 0, 0);
    let dphi_normal: u32 = 42_845_688; // 440 Hz at 44100 SR
                                       // We use a safety margin (0x1000) so that, after Scalar→u32 quantization,
                                       // the computed dphi is still > DISCONTINUITY_THRESHOLD. This avoids edge
                                       // sensitivity near the strict-> boundary.
                                       //
                                       // Bound: phase_from_u32 → Scalar → phase_to_u32 round-trip error is at most
                                       // 1 u32 unit (I64F64 has 64 fractional bits; absolute error for a u32 input
                                       // ≤ 2^32 is at most a few × 2^{-32} < 1, plus to_num::<u32>() truncation
                                       // of at most 1 unit). Margin 0x1000 = 4096 >> 1; freeze is assured.
    let dphi_freeze: u32 = DISCONTINUITY_THRESHOLD + 0x1000;

    let mut state = OscState::default();

    // Warm-up tick: clears state.tri.init (first tick returns 0 and sets init=true,
    // establishing state.tri.prev_phase_u32 as the baseline for loop phase driving).
    let warmup_phase = phase_from_u32(0u32);
    dpw4::TriangleDPW4::tick(&mut state, warmup_phase, &gain);

    // After warm-up: never assign state.tri.prev_phase_u32.
    // Each iteration reads it to derive the next phase_u32, then passes that phase
    // into tick. Tick owns all writes to state.tri.prev_phase_u32.
    for i in 0..8192_u32 {
        let is_freeze = (i % 2) == 1;

        let dphi_target = if is_freeze { dphi_freeze } else { dphi_normal };

        // Compute phase_u32 by advancing from tick's last recorded prev.
        // Never write to state.tri.prev_phase_u32 directly.
        let phase_u32 = state.tri.prev_phase_u32.wrapping_add(dphi_target);
        let phase = phase_from_u32(phase_u32);

        // Snapshot accumulator before freeze tick.
        let z_before: i128 = if is_freeze { state.tri.z } else { 0 };

        dpw4::TriangleDPW4::tick(&mut state, phase, &gain);

        // Primary invariant: freeze tick must not change z.
        if is_freeze {
            assert_eq!(
                state.tri.z, z_before,
                "freeze tick {}: z must be unchanged (dphi > DISCONTINUITY_THRESHOLD)",
                i
            );
        }
    }
}
