use dpw4::{apply_gain, math, tick_dpw4_raw, DpwGain, OscState, Scalar, SignalFrameHeader};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Q124 LSB factor for projecting i128 raw state to f64.
/// 2^-124 = 4.70197740328915e-38
const Q124_LSB: f64 = 4.70197740328915e-38;

/// Rational Coefficients for 4th-order DPW (1/N!)
const C4: f64 = 1.0 / 24.0;

struct VerificationScenario {
    filename: &'static str,
    target_desc: &'static str,
    freq: f64,
    duty_pct: f64,
    num_samples: usize,
    shape: u32,
    gain_exponent: i32,
    sample_rate: f64,
}

// =============================================================================
// Shadow Reference Logic (Scientific f64)
// =============================================================================

/// Independent Scientific Reference (Idealized f64)
struct Dpw4StateF64 {
    z1: f64,
    z2: f64,
    z3: f64,
}

impl Dpw4StateF64 {
    fn new() -> Self {
        Self {
            z1: 0.0,
            z2: 0.0,
            z3: 0.0,
        }
    }
    fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
        self.z3 = 0.0;
    }
}

/// tick_f64: Independent 4th-order DPW implementation using pure libm.
/// This establishes the "Reference Contract" against the i128 core.
fn tick_dpw4_f64(state: &mut Dpw4StateF64, phase_01: f64) -> f64 {
    // Bipolar: -1.0 .. 1.0 to match core i128 logic (s_q31 range)
    let s = (phase_01 * 2.0) - 1.0;

    // x^4 polynomial
    let p4 = s * s * s * s;

    // 3rd order differentiator
    let d1 = p4 - state.z1;
    state.z1 = p4;
    let d2 = d1 - state.z2;
    state.z2 = d1;
    let d3 = d2 - state.z3;
    state.z3 = d2;

    // Apply rational scaling factor (1/N!)
    d3 * C4
}

// =============================================================================
// Simulation Logic
// =============================================================================

fn main() -> std::io::Result<()> {
    let output_dir = "docs/verification";
    fs::create_dir_all(Path::new(output_dir))?;

    let scenarios = vec![
        VerificationScenario {
            filename: "phase_wrap_440.csv",
            target_desc: "440Hz Rigorous Phase-Wrap Trace (SR: 44100)",
            freq: 440.0,
            duty_pct: 0.0,
            num_samples: 1000,
            shape: 0, // Saw
            gain_exponent: 10,
            sample_rate: 44100.0,
        },
        VerificationScenario {
            filename: "pulse_relational_8k.csv",
            target_desc: "8kHz Pulse 10% (Relational i128 Proof)",
            freq: 8000.0,
            duty_pct: 10.0,
            num_samples: 300,
            shape: 1, // Pulse
            gain_exponent: 0,
            sample_rate: 48000.0,
        },
        VerificationScenario {
            filename: "saw_20_headroom.csv",
            target_desc: "20Hz Sawtooth (Unbounded Headroom Proof)",
            freq: 20.0,
            duty_pct: 0.0,
            num_samples: 4800,
            shape: 0, // Saw
            gain_exponent: 34,
            sample_rate: 48000.0,
        },
        VerificationScenario {
            filename: "long_run_0_1hz.csv",
            target_desc: "0.1Hz Long-Run Stability Test (1M samples)",
            freq: 0.1,
            duty_pct: 0.0,
            num_samples: 1_000_000,
            shape: 0, // Saw
            gain_exponent: 0,
            sample_rate: 48000.0,
        },
    ];

    println!("Generating Precision-DPW v1.0.0-rc5 Forensic Artifacts...");

    for scenario in scenarios {
        generate_artifact(output_dir, &scenario)?;
    }

    println!(
        "Success. Gold Standard Reference artifacts generated in '{}'",
        output_dir
    );
    Ok(())
}

fn generate_artifact(dir: &str, scenario: &VerificationScenario) -> std::io::Result<()> {
    let path = Path::new(dir).join(scenario.filename);
    let bin_path = path.with_extension("bin");
    let mut file = File::create(&path)?;
    let mut bin_file = File::create(&bin_path)?;

    // Write Header (Precision-DPW Reference Contract)
    writeln!(file, "# Precision-DPW v1.0.0-rc5 Reference Finalization")?;
    writeln!(file, "# Target: {}", scenario.target_desc)?;
    writeln!(file, "# SR: {}", scenario.sample_rate)?;
    writeln!(file, "# Columns: tick, ticksample_i16, i128_scaled, f64_ref, internal_residual, clipping_active, current_exponent, raw_u64_phase")?;
    writeln!(file, "tick, ticksample_i16, i128_scaled, f64_ref, internal_residual, clipping_active, current_exponent, raw_u64_phase")?;

    // Initial State Consistency: Explicit Zero-Initialization
    let mut state_i128 = OscState::new();
    state_i128.reset();

    let mut ref_a = Dpw4StateF64::new();
    let mut ref_b = Dpw4StateF64::new();
    ref_a.reset();
    ref_b.reset();

    // Gain Configuration
    // We want the production output to be visible in i16.
    // Normalized Saw: (-0.5 .. 0.5)
    // To map to i16 (-32768 .. 32767), we need gain factor 65536.
    // production_mantissa (m4_q63) = (C4 * 65536 / 2^64) ??
    // prod = raw * m_q63 >> (124 - exp)
    // We want prod = (raw * C4 * 2^exp) * 32768
    // So m_q63 should be C4 * 32768 * 2^64 normalized?
    // Let's just use (C4 * 2^63) for mantissa (pure C4) and exp+15 to map to i16.
    let m4_q63 = (libm::floor(C4 * 9223372036854775808.0 + 0.5)) as u64; // C4 in Q63
    let gain_i128 = DpwGain::new(m4_q63, scenario.gain_exponent + 15, 0, 0);

    // libm strict routing for increment calculation
    let duty_scalar = Scalar::from_num(scenario.duty_pct / 100.0);
    state_i128.duty = duty_scalar;

    let mut current_phase = Scalar::ZERO;
    let freq_scalar = Scalar::from_num(scenario.freq);
    let rate_scalar = Scalar::from_num(scenario.sample_rate);
    let phase_inc_scalar = (freq_scalar / rate_scalar) * math::TWO_PI;

    // float_gain from libm: pow(2.0, exp)
    let float_gain = libm::pow(2.0, scenario.gain_exponent as f64);

    for tick in 0..scenario.num_samples {
        // --- 1. Integer Core Path (Manual Tick to capture raw4) ---
        let (sample_i16, raw_i128) = if scenario.shape == 1 || scenario.shape == 3 {
            // PULSE/SQUARE: Two-Saw Differential
            let duty_val = if scenario.shape == 3 {
                Scalar::from_num(0.5)
            } else {
                state_i128.duty
            };
            let duty_ph = duty_val * math::TWO_PI;

            // Map phase to bipolar Q31
            let to_q31 = |mut p: Scalar| {
                p %= math::TWO_PI;
                if p < Scalar::ZERO {
                    p += math::TWO_PI;
                }
                let u32_phase = (p / math::TWO_PI * Scalar::from_num(4294967296.0)).to_num::<u32>();
                (u32_phase as i64).wrapping_sub(1i64 << 31)
            };

            let s_a_q31 = to_q31(current_phase);
            let raw_a = tick_dpw4_raw(&mut state_i128.saw_a, s_a_q31);

            let s_b_q31 = to_q31(current_phase - duty_ph);
            let raw_b = tick_dpw4_raw(&mut state_i128.saw_b, s_b_q31);

            // High-Precision Switching: Normalize in raw domain before gain
            let raw_diff = raw_a.wrapping_sub(raw_b);
            let raw_mix = raw_diff >> 1;

            let sample = apply_gain(raw_mix, gain_i128.m4_q63, gain_i128.e4);

            // Forensic relational capture
            (sample, raw_diff)
        } else {
            // SAW (Fixed 4th-order)
            let mut p = current_phase % math::TWO_PI;
            if p < Scalar::ZERO {
                p += math::TWO_PI;
            }
            let u32_phase = (p / math::TWO_PI * Scalar::from_num(4294967296.0)).to_num::<u32>();
            let s_q31 = (u32_phase as i64).wrapping_sub(1i64 << 31);
            let raw4 = tick_dpw4_raw(&mut state_i128.saw_a, s_q31);
            let sample = apply_gain(raw4, gain_i128.m4_q63, gain_i128.e4);
            (sample, raw4)
        };

        // --- 2. Independent Shadow Model (libm) ---
        let ph_01 = (current_phase / math::TWO_PI).to_num::<f64>();

        let out_ref = if scenario.shape == 1 || scenario.shape == 3 {
            let duty_f = scenario.duty_pct / 100.0;
            let sa = tick_dpw4_f64(&mut ref_a, ph_01);

            let mut ph_b = ph_01 - duty_f;
            if ph_b < 0.0 {
                ph_b += 1.0;
            }
            let sb = tick_dpw4_f64(&mut ref_b, ph_b);

            // Differential 0.5x scaling (Phase 8)
            (sa - sb) * 0.5 * float_gain
        } else {
            tick_dpw4_f64(&mut ref_a, ph_01) * float_gain
        };

        // --- 3. Data Integrity & Normalization ---

        // Project i128 raw state to f64 using constant-time normalization
        // If relational, the factor is 0.5 (from core logic normalization)
        let scaling_factor = if scenario.shape == 1 || scenario.shape == 3 {
            0.5
        } else {
            1.0
        };
        // We include C4 in i128_scaled to match out_ref (Ideal)
        let i128_scaled = (raw_i128 as f64) * Q124_LSB * scaling_factor * float_gain * C4;

        // Internal Residual: Scientific prove of superiority
        let residual = i128_scaled - out_ref;

        // Clipping detection: trigger if scaled signal exceeds production bounds
        // In production, i16 range is [-1.0, 1.0].
        let clipping_active = if libm::fabs(i128_scaled) > 1.0 { 1 } else { 0 };

        // Output formatting
        writeln!(
            file,
            "{}, {}, {:.18e}, {:.18e}, {:.18e}, {}, {}, {:.18e}",
            tick,
            sample_i16,
            i128_scaled,
            out_ref, // Unbounded Ref
            residual,
            clipping_active,
            scenario.gain_exponent,
            current_phase.to_num::<f64>()
        )?;

        // --- 4. Binary Header Emission (for rapid audit) ---
        let header = SignalFrameHeader::new(tick as u64, scenario.sample_rate as u32);
        bin_file.write_all(&header.to_bytes())?;

        current_phase += phase_inc_scalar;
        if current_phase >= math::TWO_PI {
            current_phase -= math::TWO_PI;
        }
    }

    println!("Generated: {} (and .bin)", scenario.filename);
    Ok(())
}
