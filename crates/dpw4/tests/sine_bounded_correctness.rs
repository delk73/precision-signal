use dpw4::math;
use dpw4::{DpwGain, OscState, Scalar, SignalShape, Sine, HEADROOM_BITS, SINE_EGRESS_SCALE_Q31};

const PHASE_SWEEP_STEPS: u32 = 4096;
const MAX_ABS_RESIDUAL_BOUND: f64 = 1.0e-6;

#[test]
fn test_sine_bounded_correctness_over_phase_sweep() {
    let gain = DpwGain::new(1u64 << 63, 0, 0, 0);
    let mut state = OscState::new();
    let egress_scale = (SINE_EGRESS_SCALE_Q31 >> HEADROOM_BITS) as f64;

    let mut max_abs_residual = 0.0_f64;
    let mut max_index = 0_u32;

    for i in 0..=PHASE_SWEEP_STEPS {
        let phase = Scalar::from_num(i) * (math::TWO_PI / Scalar::from_num(PHASE_SWEEP_STEPS));
        let sample = Sine::tick(&mut state, phase, &gain);
        let observed = (sample as f64) / egress_scale;
        let reference = libm::sin(phase.to_num::<f64>());
        let residual = observed - reference;
        let abs_residual = libm::fabs(residual);

        if abs_residual > max_abs_residual {
            max_abs_residual = abs_residual;
            max_index = i;
        }
    }

    println!(
        "domain=phase_i=i*2pi/4096,count=4097,max_abs_residual={:.12e},max_index={}",
        max_abs_residual, max_index
    );

    assert!(
        max_abs_residual <= MAX_ABS_RESIDUAL_BOUND,
        "max sine residual {:.12e} exceeded bound {:.12e} over 4097-point phase sweep",
        max_abs_residual,
        MAX_ABS_RESIDUAL_BOUND
    );
}
