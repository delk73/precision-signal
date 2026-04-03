use super::ValidateMode;
use dpw4::{
    apply_gain, math, DpwGain, OscState, Pulse, Sawtooth, Scalar, SignalFrameHeader, SignalShape,
    Sine, TriangleDPW4,
};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

pub(crate) const Q124_LSB: f64 = 4.70197740328915e-38;
pub(crate) const C4: f64 = 1.0 / 24.0;
pub(crate) const GAIN_M4_Q63_QUICK: u64 = 384_307_168_202_282_304;
pub(crate) const HEADER_TEST_FRAMES: u64 = 100;
pub(crate) const HEADER_TEST_RATE: u32 = 48_000;

const NORMATIVE_DET_HASHES: &[(&str, &str)] = &[
    (
        "saw_20_headroom",
        "ec99d4d0407bb48ec442e629e66f08f13547913c0583b31fe1c0e48df067edc1",
    ),
    (
        "pulse_relational_8k",
        "a3b8e9f6cfa2e0f9c35819eb2d23247b97c5acbf01703f242849a68f767d70cd",
    ),
    (
        "triangle_linearity_1k",
        "9d2cb61f1edc5eb0d2a288f0632db02662ccfd091369eb6819b16270c813c74e",
    ),
    (
        "sine_linearity_1k",
        "e30e44002036a3f296b84c4907c7172364457b9ac751f55ddfce311419eed4ab",
    ),
    (
        "master_sweep_20_20k",
        "6ad85015a9eeee2d81305013c238bc0e666b40e3bb786d4befa4c9f5d3688b0c",
    ),
    (
        "long_run_0_1hz",
        "3f2a364cf0e0697a77b75ff89cb0f55153b41cdd070e4eedafb6868a1017fa12",
    ),
];

pub(crate) fn expected_det_hash(_mode: ValidateMode, id: &str) -> Option<&'static str> {
    NORMATIVE_DET_HASHES
        .iter()
        .find(|(k, _)| *k == id)
        .map(|(_, h)| *h)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CanonChannels {
    Mono,
    MasterTuple,
}

#[derive(Clone, Copy)]
pub(crate) enum GoldenPolicy {
    Pinned(&'static str),
    Unpinned,
}

pub(crate) struct VerificationScenario {
    pub(crate) id: &'static str,
    pub(crate) target_desc: &'static str,
    pub(crate) freq: f64,
    pub(crate) duty_pct: f64,
    pub(crate) num_samples: usize,
    pub(crate) shape: u32,
    pub(crate) gain_exponent: i32,
    pub(crate) sample_rate: f64,
    pub(crate) channels: CanonChannels,
    pub(crate) golden: GoldenPolicy,
}

pub(crate) static SCENARIOS: &[VerificationScenario] = &[
    VerificationScenario {
        id: "phase_wrap_440",
        target_desc: "440Hz Rigorous Phase-Wrap Trace (SR: 44100)",
        freq: 440.0,
        duty_pct: 0.0,
        num_samples: 1000,
        shape: 0,
        gain_exponent: 10,
        sample_rate: 44100.0,
        channels: CanonChannels::Mono,
        golden: GoldenPolicy::Unpinned,
    },
    VerificationScenario {
        id: "pulse_relational_8k",
        target_desc: "8kHz Pulse 10% (Relational i128 Proof)",
        freq: 8000.0,
        duty_pct: 10.0,
        num_samples: 300,
        shape: 1,
        gain_exponent: 0,
        sample_rate: 48000.0,
        channels: CanonChannels::Mono,
        golden: GoldenPolicy::Pinned(
            "953d0533d83ba3573e7f7948199f5cd66adc6f5c43fdedf9ceb2ffe45fdb324c",
        ),
    },
    VerificationScenario {
        id: "saw_20_headroom",
        target_desc: "20Hz Sawtooth (Unbounded Headroom Proof)",
        freq: 20.0,
        duty_pct: 0.0,
        num_samples: 4800,
        shape: 0,
        gain_exponent: 34,
        sample_rate: 48000.0,
        channels: CanonChannels::Mono,
        golden: GoldenPolicy::Pinned(
            "143d4613ab993dbacc5e5f735d4985ee1a188b8b18d1a906dd44ae621086993e",
        ),
    },
    VerificationScenario {
        id: "triangle_linearity_1k",
        target_desc: "1kHz Triangle (Linearity & Symmetry Check)",
        freq: 1000.0,
        duty_pct: 0.0,
        num_samples: 4800,
        shape: 2,
        gain_exponent: 0,
        sample_rate: 48000.0,
        channels: CanonChannels::Mono,
        golden: GoldenPolicy::Pinned(
            "74ef101edcdcbffb20be729ed503eaa04dd47301ec2f3b26f76786879850dbfd",
        ),
    },
    VerificationScenario {
        id: "sine_linearity_1k",
        target_desc: "1kHz Sine (Curvature & CORDIC Identity)",
        freq: 1000.0,
        duty_pct: 0.0,
        num_samples: 4800,
        shape: 4,
        gain_exponent: 0,
        sample_rate: 48000.0,
        channels: CanonChannels::Mono,
        golden: GoldenPolicy::Pinned(
            "00a9577a82ffe1c9b6a05ee9bcd6f93947463bd901ef05d528e2a7704006bfbc",
        ),
    },
    VerificationScenario {
        id: "long_run_0_1hz",
        target_desc: "0.1Hz Long-Run Stability Test (1M samples)",
        freq: 0.1,
        duty_pct: 0.0,
        num_samples: 1_000_000,
        shape: 0,
        gain_exponent: 0,
        sample_rate: 48000.0,
        channels: CanonChannels::Mono,
        golden: GoldenPolicy::Pinned(
            "1e116cc76d4c460f5eab421a30f20b42157fe516d53440baca3c7dabdb92d420",
        ),
    },
    VerificationScenario {
        id: "master_sweep_20_20k",
        target_desc: "Full Spectrum Synchronous Chirp (20Hz-20kHz, 1s)",
        freq: 20.0,
        duty_pct: 0.0,
        num_samples: 48_000,
        shape: 10,
        gain_exponent: 0,
        sample_rate: 48000.0,
        channels: CanonChannels::MasterTuple,
        golden: GoldenPolicy::Pinned(
            "0ac058b60498cbfb129d4a35b37f2f6c785752fa888c6c4367b0608a4ef825ea",
        ),
    },
];

pub(crate) fn quick_validate_scenarios() -> &'static [VerificationScenario] {
    SCENARIOS
}

pub(crate) fn quick_validate_gain_for_scenario(scenario: &VerificationScenario) -> DpwGain {
    DpwGain::new(GAIN_M4_Q63_QUICK, scenario.gain_exponent + 15, 0, 0)
}

pub(crate) fn generate_forensic_artifacts(
    out_dir: &Path,
    _mode: Option<ValidateMode>,
) -> io::Result<()> {
    fs::create_dir_all(out_dir)?;
    for scenario in quick_validate_scenarios() {
        generate_artifact(out_dir, scenario)?;
    }
    Ok(())
}

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

fn tick_dpw4_f64(state: &mut Dpw4StateF64, phase_01: f64) -> f64 {
    let s = (phase_01 * 2.0) - 1.0;
    let p4 = s * s * s * s;
    let d1 = p4 - state.z1;
    state.z1 = p4;
    let d2 = d1 - state.z2;
    state.z2 = d1;
    let d3 = d2 - state.z3;
    state.z3 = d2;
    d3 * C4
}

pub(crate) fn generate_header_test_artifact(path: &Path) -> io::Result<()> {
    let mut file = File::create(path)?;
    for i in 0..HEADER_TEST_FRAMES {
        let header = SignalFrameHeader::new(i, HEADER_TEST_RATE);
        file.write_all(&header.to_bytes())?;
    }
    file.flush()?;
    Ok(())
}

fn generate_artifact(dir: &Path, scenario: &VerificationScenario) -> io::Result<()> {
    let stem = scenario.id;
    let path = dir.join(format!("{}.csv", stem));
    let det_path = dir.join(format!("{}.det.csv", stem));
    let bin_path = dir.join(format!("{}_headers.bin", stem));
    let canon_path = dir.join(format!("{}.canon.sig", stem));

    let mut file = io::BufWriter::new(File::create(&path)?);
    let mut det_file = io::BufWriter::new(File::create(&det_path)?);
    let mut bin_file = io::BufWriter::new(File::create(&bin_path)?);
    let mut canon_file = io::BufWriter::new(File::create(&canon_path)?);

    writeln!(
        file,
        "# Precision-DPW v{} Reference Finalization",
        env!("CARGO_PKG_VERSION")
    )?;
    writeln!(file, "# Target: {}", scenario.target_desc)?;
    writeln!(file, "# SR: {}", scenario.sample_rate)?;
    if scenario.shape == 10 {
        writeln!(
            file,
            "# Columns: tick, saw_s32, pulse_s32, tri_s32, sine_s32, saw_phase_ref, phase_residual, sine_ref, sine_residual, raw_u64_phase"
        )?;
        writeln!(
            file,
            "tick, saw_s32, pulse_s32, tri_s32, sine_s32, saw_phase_ref, phase_residual, sine_ref, sine_residual, raw_u64_phase"
        )?;
        writeln!(
            det_file,
            "tick,saw_s32,pulse_s32,tri_s32,sine_s32,phase_u32_hex"
        )?;
    } else {
        writeln!(file, "# Columns: tick, sample_i32, i128_scaled, f64_ref, internal_residual, clipping_active, current_exponent, raw_u64_phase")?;
        writeln!(file, "tick, sample_i32, i128_scaled, f64_ref, internal_residual, clipping_active, current_exponent, raw_u64_phase")?;
        writeln!(det_file, "tick,sample_i32,raw_i128_hex,phase_u32_hex,shape")?;
    }

    let mut hasher = Sha256::new();
    let mut state_i128 = OscState::new();
    state_i128.reset();

    let mut ref_a = Dpw4StateF64::new();
    let mut ref_b = Dpw4StateF64::new();
    ref_a.reset();
    ref_b.reset();

    let gain_i128 = quick_validate_gain_for_scenario(scenario);
    let duty_scalar = Scalar::from_num(scenario.duty_pct / 100.0);
    state_i128.duty = duty_scalar;

    let mut current_phase = Scalar::ZERO;
    let freq_scalar = Scalar::from_num(scenario.freq);
    let rate_scalar = Scalar::from_num(scenario.sample_rate);
    let phase_inc_scalar = (freq_scalar / rate_scalar) * math::TWO_PI;
    let float_gain = libm::pow(2.0, scenario.gain_exponent as f64);

    let chirp_f0 = Scalar::from_num(20.0);
    let chirp_f1 = Scalar::from_num(20000.0);
    let chirp_step = if scenario.num_samples > 1 {
        (chirp_f1 - chirp_f0) / Scalar::from_num((scenario.num_samples - 1) as f64)
    } else {
        Scalar::ZERO
    };

    let mut sweep_saw = OscState::new();
    let mut sweep_pulse = OscState::new();
    let mut sweep_tri = OscState::new();
    let mut sweep_sine = OscState::new();
    sweep_saw.reset();
    sweep_pulse.reset();
    sweep_tri.reset();
    sweep_sine.reset();
    sweep_pulse.duty = Scalar::from_num(0.5);

    let sweep_f0_f64 = 20.0f64;
    let sweep_f1_f64 = 20000.0f64;
    let sweep_k_f64 = if scenario.num_samples > 1 {
        (sweep_f1_f64 - sweep_f0_f64) / ((scenario.num_samples - 1) as f64)
    } else {
        0.0
    };

    for tick in 0..scenario.num_samples {
        if scenario.shape == 10 {
            let phase_u32 =
                (current_phase / math::TWO_PI * Scalar::from_num(4294967296.0)).to_num::<u32>();

            let saw_s32 = Sawtooth::tick(&mut sweep_saw, current_phase, &gain_i128);
            let pulse_s32 = Pulse::tick(&mut sweep_pulse, current_phase, &gain_i128);
            let tri_s32 = TriangleDPW4::tick(&mut sweep_tri, current_phase, &gain_i128);
            let sine_s32 = Sine::tick(&mut sweep_sine, current_phase, &gain_i128);

            if scenario.channels == CanonChannels::MasterTuple {
                hasher.update(&saw_s32.to_le_bytes());
                hasher.update(&pulse_s32.to_le_bytes());
                hasher.update(&tri_s32.to_le_bytes());
                hasher.update(&sine_s32.to_le_bytes());
            }

            let n = tick as f64;
            let phase_ref = 2.0
                * core::f64::consts::PI
                * ((sweep_f0_f64 * n) / scenario.sample_rate
                    + 0.5 * sweep_k_f64 * n * n / scenario.sample_rate);
            let phase_ref_01 = phase_ref / (2.0 * core::f64::consts::PI);
            let saw_ref = 2.0 * (phase_ref_01 - libm::floor(phase_ref_01)) - 1.0;
            let phase_from_scalar_01 = (current_phase / math::TWO_PI).to_num::<f64>();
            let saw_from_scalar = 2.0 * phase_from_scalar_01 - 1.0;
            let phase_residual = saw_from_scalar - saw_ref;
            let sine_ref = libm::sin(phase_ref);
            let sine_norm = (sine_s32 as f64) / (i32::MAX as f64);
            let sine_residual = sine_norm - sine_ref;

            writeln!(
                file,
                "{}, {}, {}, {}, {}, {:.18e}, {:.18e}, {:.18e}, {:.18e}, {:.18e}",
                tick,
                saw_s32,
                pulse_s32,
                tri_s32,
                sine_s32,
                saw_ref,
                phase_residual,
                sine_ref,
                sine_residual,
                current_phase.to_num::<f64>()
            )?;
            writeln!(
                det_file,
                "{},{},{},{},{},0x{:08x}",
                tick, saw_s32, pulse_s32, tri_s32, sine_s32, phase_u32
            )?;

            let header = SignalFrameHeader::new(tick as u64, scenario.sample_rate as u32);
            bin_file.write_all(&header.to_bytes())?;

            let inst_freq = chirp_f0 + chirp_step * Scalar::from_num(tick as f64);
            let phase_inc_chirp = (inst_freq / rate_scalar) * math::TWO_PI;
            current_phase += phase_inc_chirp;
            if current_phase >= math::TWO_PI {
                current_phase -= math::TWO_PI;
            }
            continue;
        }

        let (sample_i32, raw_i128) = if scenario.shape == 1 || scenario.shape == 3 {
            let duty_val = if scenario.shape == 3 {
                Scalar::from_num(0.5)
            } else {
                state_i128.duty
            };
            let duty_ph = duty_val * math::TWO_PI;

            let to_q31 = |mut p: Scalar| {
                p %= math::TWO_PI;
                if p < Scalar::ZERO {
                    p += math::TWO_PI;
                }
                let u32_phase = (p / math::TWO_PI * Scalar::from_num(4294967296.0)).to_num::<u32>();
                (u32_phase as i64).wrapping_sub(1i64 << 31)
            };

            let s_a_q31 = to_q31(current_phase);
            let raw_a = dpw4::tick_dpw4_raw(&mut state_i128.saw_a, s_a_q31);

            let s_b_q31 = to_q31(current_phase - duty_ph);
            let raw_b = dpw4::tick_dpw4_raw(&mut state_i128.saw_b, s_b_q31);

            let raw_diff = raw_a.wrapping_sub(raw_b);
            let raw_mix = raw_diff >> 1;
            let sample = apply_gain(raw_mix, gain_i128.m4_q63, gain_i128.e4);
            (sample, raw_diff)
        } else if scenario.shape == 2 {
            let sample = TriangleDPW4::tick(&mut state_i128, current_phase, &gain_i128);
            (sample, sample as i128)
        } else if scenario.shape == 4 {
            let sample = Sine::tick(&mut state_i128, current_phase, &gain_i128);
            (sample, sample as i128)
        } else {
            let mut p = current_phase % math::TWO_PI;
            if p < Scalar::ZERO {
                p += math::TWO_PI;
            }
            let u32_phase = (p / math::TWO_PI * Scalar::from_num(4294967296.0)).to_num::<u32>();
            let s_q31 = (u32_phase as i64).wrapping_sub(1i64 << 31);
            let raw4 = dpw4::tick_dpw4_raw(&mut state_i128.saw_a, s_q31);
            let sample = apply_gain(raw4, gain_i128.m4_q63, gain_i128.e4);
            (sample, raw4)
        };

        if scenario.channels == CanonChannels::Mono {
            hasher.update(&sample_i32.to_le_bytes());
        }

        let ph_01 = (current_phase / math::TWO_PI).to_num::<f64>();
        let out_ref = if scenario.shape == 1 || scenario.shape == 3 {
            let duty_f = scenario.duty_pct / 100.0;
            let sa = tick_dpw4_f64(&mut ref_a, ph_01);

            let mut ph_b = ph_01 - duty_f;
            if ph_b < 0.0 {
                ph_b += 1.0;
            }
            let sb = tick_dpw4_f64(&mut ref_b, ph_b);
            (sa - sb) * 0.5 * float_gain
        } else if scenario.shape == 2 {
            let tri = 4.0 * libm::fabs(ph_01 - libm::floor(ph_01 + 0.75) + 0.25) - 1.0;
            tri * float_gain
        } else if scenario.shape == 4 {
            libm::sin(current_phase.to_num::<f64>()) * float_gain
        } else {
            tick_dpw4_f64(&mut ref_a, ph_01) * float_gain
        };

        let scaling_factor = if scenario.shape == 1 || scenario.shape == 3 {
            0.5
        } else {
            1.0
        };
        let i128_scaled = if scenario.shape == 2 || scenario.shape == 4 {
            (sample_i32 as f64) / (i32::MAX as f64)
        } else {
            (raw_i128 as f64) * Q124_LSB * scaling_factor * float_gain * C4
        };
        let residual = i128_scaled - out_ref;
        let clipping_active = if libm::fabs(i128_scaled) > 1.0 { 1 } else { 0 };

        writeln!(
            file,
            "{}, {}, {:.18e}, {:.18e}, {:.18e}, {}, {}, {:.18e}",
            tick,
            sample_i32,
            i128_scaled,
            out_ref,
            residual,
            clipping_active,
            scenario.gain_exponent,
            current_phase.to_num::<f64>()
        )?;
        let phase_u32 =
            (current_phase / math::TWO_PI * Scalar::from_num(4294967296.0)).to_num::<u32>();
        writeln!(
            det_file,
            "{},{},0x{:032x},0x{:08x},{}",
            tick, sample_i32, raw_i128 as u128, phase_u32, scenario.shape
        )?;

        let header = SignalFrameHeader::new(tick as u64, scenario.sample_rate as u32);
        bin_file.write_all(&header.to_bytes())?;

        current_phase += phase_inc_scalar;
        if current_phase >= math::TWO_PI {
            current_phase -= math::TWO_PI;
        }
    }

    let hash_string = hex::encode(hasher.finalize());
    let channels_str = match scenario.channels {
        CanonChannels::Mono => "mono",
        CanonChannels::MasterTuple => "saw,pulse,triangle,sine",
    };
    writeln!(
        canon_file,
        "v1 | le-i32 | channels={} | samples={}",
        channels_str, scenario.num_samples
    )?;
    writeln!(canon_file, "{}", hash_string)?;

    Ok(())
}
