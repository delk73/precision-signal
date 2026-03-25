use dpw4::{math, signal_pipe, DpwGain, OscState, Pulse, Sawtooth, Scalar, HEADROOM_BITS};
use sha2::{Digest, Sha256};

#[test]
fn test_golden_lock() {
    let gain = DpwGain::new(1u64 << 63, 0, 0, 0); // Reference Unity Gain

    // 1. Saw-440Hz-10k
    let mut osc = OscState::new();
    let mut phases = [Scalar::ZERO; 1000];
    let freq = Scalar::from_num(440.0);
    let rate = Scalar::from_num(44100.0);
    let phase_inc = (freq / rate) * math::TWO_PI;

    let mut hasher = Sha256::new();
    let mut output = [0i32; 1000];
    let mut current_phase = Scalar::ZERO;

    for count in 0..10 {
        for phase in &mut phases {
            *phase = current_phase;
            current_phase += phase_inc;
            if current_phase >= math::TWO_PI {
                current_phase -= math::TWO_PI;
            }
        }
        signal_pipe::<Sawtooth>(&mut osc, &phases, &gain, &mut output);

        // VERIFICATION: Check first sample matches theoretical start
        // Sawtooth(0) = 1.0 normalized = (1 << 16) >> HEADROOM_BITS
        if count == 0 {
            assert_eq!(
                output[0],
                (1 << 16) >> HEADROOM_BITS,
                "Sawtooth first sample must match headroom-normalized unity at phase 0"
            );
        }

        for &sample in &output {
            hasher.update(sample.to_le_bytes());
        }
    }

    let digest = hasher.finalize();
    let actual_saw = hex::encode(digest);
    let expected_saw = hex::encode(dpw4::goldens::SAW_GOLDEN_HASH);
    assert_eq!(
        actual_saw, expected_saw,
        "Sawtooth Golden Hash mismatch - Reference Lock Breached!"
    );

    // 2. Pulse-10%-440Hz-10k
    osc.reset();
    osc.duty = Scalar::from_num(0.1);
    current_phase = Scalar::ZERO;
    let mut hasher_p = Sha256::new();

    for _ in 0..10 {
        for phase in &mut phases {
            *phase = current_phase;
            current_phase += phase_inc;
            if current_phase >= math::TWO_PI {
                current_phase -= math::TWO_PI;
            }
        }
        signal_pipe::<Pulse>(&mut osc, &phases, &gain, &mut output);
        for &sample in &output {
            hasher_p.update(sample.to_le_bytes());
        }
    }

    let digest_pulse = hasher_p.finalize();
    let actual_pulse = hex::encode(digest_pulse);
    let expected_pulse = hex::encode(dpw4::goldens::PULSE_GOLDEN_HASH);
    assert_eq!(
        actual_pulse, expected_pulse,
        "Pulse Golden Hash mismatch - Reference Lock Breached!"
    );
}
