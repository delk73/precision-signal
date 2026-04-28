use dpw4::math;
use dpw4::{DpwGain, OscState, Scalar};
use dpw4::{SignalShape, TriangleDPW4};

#[test]
#[ignore = "characterization: 1M ticks; run manually with --ignored"]
fn test_triangle_phase_pump() {
    let mut state = OscState::new();
    let gain = DpwGain::new(1 << 63, 0, 0, 0);

    // dphi_u32 = round(440 * 2^32 / 44100) = 42845688
    let dphi_u32: u32 = 42845688;
    let scale_2_32: Scalar = Scalar::from_bits(1_i128 << 96); // 2^32 in I64F64
    let normal_inc = (Scalar::from_num(dphi_u32) / scale_2_32) * math::TWO_PI;

    // Deterministic Jump Construction
    // Discontinuity freezes when dphi > DISCONTINUITY_THRESHOLD
    let discontinuity_threshold: u32 = 1073741824; // 2^30
    let jump_dphi_u32 = discontinuity_threshold + 1;
    let freeze_inc = (Scalar::from_num(jump_dphi_u32) / scale_2_32) * math::TWO_PI;

    let num_ticks = 1_000_000;

    // 1. Establish normative baseline for N ticks
    let mut baseline_state = OscState::new();
    let mut baseline_phase = Scalar::ZERO;
    let mut max_abs_z_norm: u128 = 0;

    for _ in 0..num_ticks {
        TriangleDPW4::tick(&mut baseline_state, baseline_phase, &gain);
        let z = baseline_state.tri.z;
        let abs_z: u128 = z.unsigned_abs();
        if abs_z > max_abs_z_norm {
            max_abs_z_norm = abs_z;
        }

        baseline_phase += normal_inc;
        clip_phase(&mut baseline_phase);
    }

    // 2. Adaptive pump
    let mut pump_phase = Scalar::ZERO;
    let mut max_abs_z_pump: u128 = 0;

    for _ in 0..num_ticks {
        // Dry run normal increment to predict delta
        let mut dry_state = state;
        let dry_z_before = dry_state.tri.z;
        TriangleDPW4::tick(&mut dry_state, pump_phase, &gain);
        let delta = dry_state.tri.z.wrapping_sub(dry_z_before);

        // If expected delta < 0, we jump phase to trigger a freeze.
        let actual_phase = if delta < 0 {
            let mut jump_phase = pump_phase + freeze_inc;
            clip_phase(&mut jump_phase);
            jump_phase
        } else {
            pump_phase
        };

        if delta < 0 {
            let phase_to_u32 = |mut p: Scalar| -> u32 {
                p %= math::TWO_PI;
                if p < Scalar::ZERO {
                    p += math::TWO_PI;
                }
                (p / math::TWO_PI * scale_2_32).to_num::<u32>()
            };

            let actual_u32 = phase_to_u32(actual_phase);
            let dphi = actual_u32.wrapping_sub(state.tri.prev_phase_u32);
            assert!(
                dphi > discontinuity_threshold,
                "Adaptive jump failed: dphi ({}) <= threshold ({})",
                dphi,
                discontinuity_threshold
            );
        }

        let z_before = state.tri.z;
        TriangleDPW4::tick(&mut state, actual_phase, &gain);

        if delta < 0 {
            let z_after = state.tri.z;
            assert_eq!(
                z_after, z_before,
                "Expected integrator freeze (z unchanged) when dphi > threshold"
            );
        }

        let z = state.tri.z;
        let abs_z: u128 = z.unsigned_abs();
        if abs_z > max_abs_z_pump {
            max_abs_z_pump = abs_z;
        }

        // Setup next normal predicted phase
        pump_phase = actual_phase + normal_inc;
        clip_phase(&mut pump_phase);
    }

    // Option A: Characterization test
    assert!(max_abs_z_norm != 0, "Normative max |z| should not be zero");

    let ratio_int: u128 = max_abs_z_pump / max_abs_z_norm;
    let rem: u128 = max_abs_z_pump % max_abs_z_norm;

    println!(
        "Normative Max |z| over {} ticks: {}",
        num_ticks, max_abs_z_norm
    );
    println!("Pump Max |z| over {} ticks: {}", num_ticks, max_abs_z_pump);
    println!("Ratio floor (Pump/Normative): {}x", ratio_int);
    println!("Remainder: {}", rem);
}

fn clip_phase(phase: &mut Scalar) {
    if *phase >= math::TWO_PI {
        *phase -= math::TWO_PI;
    } else if *phase < Scalar::ZERO {
        *phase += math::TWO_PI;
    }
}
