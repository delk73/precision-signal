use dpw4::{
    apply_gain, math, tick_dpw4_raw, DpwGain, OscState, Pulse, Scalar, SignalShape, Sine,
    TriangleDPW4,
};
use std::env;
use std::fs;
use std::process;

const C4: f64 = 1.0 / 24.0;
const Q124_LSB: f64 = 4.70197740328915e-38; // 2^-124

#[derive(Clone, Copy)]
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

fn make_gain(exp: i32) -> DpwGain {
    let m4_q63 = (libm::floor(C4 * 9_223_372_036_854_775_808.0 + 0.5)) as u64;
    DpwGain::new(m4_q63, exp + 15, 0, 0)
}

#[derive(Clone, Copy)]
enum Mode {
    Pr,
    Nightly,
}

impl Mode {
    fn as_str(self) -> &'static str {
        match self {
            Mode::Pr => "pr",
            Mode::Nightly => "nightly",
        }
    }
}

struct ProbeConfig {
    n_sub_hz: usize,
    n_nyquist: usize,
    n_cordic: usize,
    n_symmetry: usize,
}

impl ProbeConfig {
    fn for_mode(mode: Mode) -> Self {
        match mode {
            Mode::Pr => Self {
                n_sub_hz: 1_000_000,
                n_nyquist: 48_000,
                n_cordic: 48_000,
                n_symmetry: 48_000,
            },
            Mode::Nightly => Self {
                n_sub_hz: 5_000_000,
                n_nyquist: 192_000,
                n_cordic: 192_000,
                n_symmetry: 192_000,
            },
        }
    }
}

struct SubHzMetrics {
    sample_rate_hz: f64,
    frequency_hz: f64,
    ticks: usize,
    non_monotonic_phase_steps: usize,
    phase_u32_stutter_steps: usize,
    max_abs_residual: f64,
    residual_unit: &'static str,
}

struct NyquistMetrics {
    sample_rate_hz: f64,
    frequency_hz: f64,
    ticks: usize,
    min_sample: i32,
    max_sample: i32,
    dc_offset_samples: f64,
    even_odd_abs_peak_delta: i64,
}

struct CordicMetrics {
    sample_rate_hz: f64,
    frequency_hz: f64,
    ticks: usize,
    hit_i32_max_count: usize,
    hit_i32_min_count: usize,
    max_abs_normalized: f64,
    clipping_active_count: usize,
}

struct SymmetryMetrics {
    sample_rate_hz: f64,
    frequency_hz: f64,
    ticks: usize,
    nonzero_diff_count: usize,
    max_abs_diff_lsb: i64,
    nonzero_sum_count: usize,
    max_abs_sum_lsb: i64,
}

fn run_sub_hz_probe(n: usize) -> SubHzMetrics {
    let sr = 48_000.0;
    let f = 0.0000001;

    let mut osc = OscState::new();
    osc.reset();
    let mut ref_saw = Dpw4StateF64::new();

    let mut phase = Scalar::ZERO;
    let phase_inc = (Scalar::from_num(f) / Scalar::from_num(sr)) * math::TWO_PI;
    let gain = make_gain(0);

    let mut max_abs_residual = 0.0f64;
    let mut non_monotonic = 0usize;
    let mut prev_phase = Scalar::from_num(-1.0);
    let mut stutter_steps = 0usize;
    let mut prev_phase_u32: Option<u32> = None;

    for _ in 0..n {
        if phase <= prev_phase {
            non_monotonic += 1;
        }
        prev_phase = phase;

        let u32_phase = (phase / math::TWO_PI * Scalar::from_num(4_294_967_296.0)).to_num::<u32>();
        if let Some(prev) = prev_phase_u32 {
            if prev == u32_phase {
                stutter_steps += 1;
            }
        }
        prev_phase_u32 = Some(u32_phase);
        let s_q31 = (u32_phase as i64).wrapping_sub(1i64 << 31);

        let raw = tick_dpw4_raw(&mut osc.saw_a, s_q31);
        let _ = apply_gain(raw, gain.m4_q63, gain.e4);

        let phase_01 = (phase / math::TWO_PI).to_num::<f64>();
        let out_ref = tick_dpw4_f64(&mut ref_saw, phase_01);
        let i128_scaled = (raw as f64) * Q124_LSB * C4;
        let residual = i128_scaled - out_ref;
        let a = libm::fabs(residual);
        if a > max_abs_residual {
            max_abs_residual = a;
        }

        phase += phase_inc;
        if phase >= math::TWO_PI {
            phase -= math::TWO_PI;
        }
    }

    SubHzMetrics {
        sample_rate_hz: sr,
        frequency_hz: f,
        ticks: n,
        non_monotonic_phase_steps: non_monotonic,
        phase_u32_stutter_steps: stutter_steps,
        max_abs_residual,
        residual_unit: "abs((raw_q124*2^-124*(1/24)) - f64_shadow_dpw4)",
    }
}

fn run_nyquist_probe(n: usize) -> NyquistMetrics {
    let sr = 48_000.0;
    let f = 23_999.0;

    let mut osc = OscState::new();
    osc.reset();
    let gain = make_gain(0);

    let mut phase = Scalar::ZERO;
    let phase_inc = (Scalar::from_num(f) / Scalar::from_num(sr)) * math::TWO_PI;

    let mut max = i32::MIN;
    let mut min = i32::MAX;
    let mut sum = 0f64;
    let mut abs_peak_even = 0i64;
    let mut abs_peak_odd = 0i64;

    for i in 0..n {
        let s = TriangleDPW4::tick(&mut osc, phase, &gain);
        if s > max {
            max = s;
        }
        if s < min {
            min = s;
        }
        sum += s as f64;
        if i % 2 == 0 {
            abs_peak_even = abs_peak_even.max((s as i64).abs());
        } else {
            abs_peak_odd = abs_peak_odd.max((s as i64).abs());
        }
        phase += phase_inc;
        if phase >= math::TWO_PI {
            phase -= math::TWO_PI;
        }
    }

    NyquistMetrics {
        sample_rate_hz: sr,
        frequency_hz: f,
        ticks: n,
        min_sample: min,
        max_sample: max,
        dc_offset_samples: sum / (n as f64),
        even_odd_abs_peak_delta: (abs_peak_even - abs_peak_odd).abs(),
    }
}

fn run_cordic_probe(n: usize) -> CordicMetrics {
    let sr = 48_000.0;
    let f = 1_000.0;

    let mut osc = OscState::new();
    osc.reset();
    let gain = make_gain(0);

    let mut phase = Scalar::ZERO;
    let phase_inc = (Scalar::from_num(f) / Scalar::from_num(sr)) * math::TWO_PI;

    let mut clipping_active_count = 0usize;
    let mut hit_i32_max_count = 0usize;
    let mut hit_i32_min_count = 0usize;
    let mut max_abs_normalized = 0.0f64;

    for _ in 0..n {
        let s = Sine::tick(&mut osc, phase, &gain);
        if s == i32::MAX {
            hit_i32_max_count += 1;
        }
        if s == i32::MIN {
            hit_i32_min_count += 1;
        }

        let norm = (s as f64) / (i32::MAX as f64);
        let a = libm::fabs(norm);
        if a > max_abs_normalized {
            max_abs_normalized = a;
        }
        if a > 1.0 {
            clipping_active_count += 1;
        }

        phase += phase_inc;
        if phase >= math::TWO_PI {
            phase -= math::TWO_PI;
        }
    }

    CordicMetrics {
        sample_rate_hz: sr,
        frequency_hz: f,
        ticks: n,
        hit_i32_max_count,
        hit_i32_min_count,
        max_abs_normalized,
        clipping_active_count,
    }
}

fn run_symmetry_probe(n: usize) -> SymmetryMetrics {
    let sr = 48_000.0;
    let f = 1_000.0;

    let gain = make_gain(0);

    let mut p10 = OscState::new();
    p10.reset();
    p10.duty = Scalar::from_num(0.1);

    let mut p90 = OscState::new();
    p90.reset();
    p90.duty = Scalar::from_num(0.9);

    let mut phase = Scalar::ZERO;
    let phase_inc = (Scalar::from_num(f) / Scalar::from_num(sr)) * math::TWO_PI;

    let mut nonzero_diff_count = 0usize;
    let mut max_abs_diff = 0i64;
    let mut nonzero_sum_count = 0usize;
    let mut max_abs_sum = 0i64;

    for _ in 0..n {
        let a = Pulse::tick(&mut p10, phase, &gain);
        let mut phase_inv = phase + math::PI;
        if phase_inv >= math::TWO_PI {
            phase_inv -= math::TWO_PI;
        }
        let b = Pulse::tick(&mut p90, phase_inv, &gain);

        let d = (a as i64) - (b as i64);
        if d != 0 {
            nonzero_diff_count += 1;
        }
        max_abs_diff = max_abs_diff.max(d.abs());

        let s = (a as i64) + (b as i64);
        if s != 0 {
            nonzero_sum_count += 1;
        }
        max_abs_sum = max_abs_sum.max(s.abs());

        phase += phase_inc;
        if phase >= math::TWO_PI {
            phase -= math::TWO_PI;
        }
    }

    SymmetryMetrics {
        sample_rate_hz: sr,
        frequency_hz: f,
        ticks: n,
        nonzero_diff_count,
        max_abs_diff_lsb: max_abs_diff,
        nonzero_sum_count,
        max_abs_sum_lsb: max_abs_sum,
    }
}

fn parse_args() -> Result<(Mode, String), String> {
    let mut mode = Mode::Pr;
    let mut out = "chaos_metrics.json".to_string();

    let mut it = env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--mode" => {
                let v = it
                    .next()
                    .ok_or_else(|| "missing value for --mode".to_string())?;
                mode = match v.as_str() {
                    "pr" => Mode::Pr,
                    "nightly" => Mode::Nightly,
                    _ => return Err(format!("invalid --mode '{}'; use 'pr' or 'nightly'", v)),
                };
            }
            "--out" => {
                out = it
                    .next()
                    .ok_or_else(|| "missing value for --out".to_string())?;
            }
            "--help" | "-h" => {
                return Err(
                    "usage: cargo run -p dpw4 --example chaos_audit -- --mode <pr|nightly> [--out chaos_metrics.json]"
                        .to_string(),
                );
            }
            _ => return Err(format!("unknown arg '{}'", arg)),
        }
    }
    Ok((mode, out))
}

fn render_json(
    mode: Mode,
    cfg: ProbeConfig,
    a: SubHzMetrics,
    b: NyquistMetrics,
    c: CordicMetrics,
    d: SymmetryMetrics,
) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"schema_version\": 1,\n",
            "  \"non_normative\": true,\n",
            "  \"mode\": \"{}\",\n",
            "  \"config\": {{\n",
            "    \"sub_hz_ticks\": {},\n",
            "    \"nyquist_ticks\": {},\n",
            "    \"cordic_ticks\": {},\n",
            "    \"symmetry_ticks\": {}\n",
            "  }},\n",
            "  \"probes\": {{\n",
            "    \"sub_hz_accumulator\": {{\n",
            "      \"sample_rate_hz\": {:.1},\n",
            "      \"frequency_hz\": {:.10},\n",
            "      \"ticks\": {},\n",
            "      \"non_monotonic_phase_steps\": {},\n",
            "      \"phase_u32_stutter_steps\": {},\n",
            "      \"max_abs_residual\": {:.18e},\n",
            "      \"residual_unit\": \"{}\"\n",
            "    }},\n",
            "    \"nyquist_corner_triangle\": {{\n",
            "      \"sample_rate_hz\": {:.1},\n",
            "      \"frequency_hz\": {:.1},\n",
            "      \"ticks\": {},\n",
            "      \"min_sample\": {},\n",
            "      \"max_sample\": {},\n",
            "      \"dc_offset_samples\": {:.18e},\n",
            "      \"even_odd_abs_peak_delta\": {}\n",
            "    }},\n",
            "    \"cordic_gain_hunt_sine\": {{\n",
            "      \"sample_rate_hz\": {:.1},\n",
            "      \"frequency_hz\": {:.1},\n",
            "      \"ticks\": {},\n",
            "      \"hit_i32_max_count\": {},\n",
            "      \"hit_i32_min_count\": {},\n",
            "      \"max_abs_normalized\": {:.18e},\n",
            "      \"clipping_active_count\": {}\n",
            "    }},\n",
            "    \"pulse_symmetry_diagnostic\": {{\n",
            "      \"sample_rate_hz\": {:.1},\n",
            "      \"frequency_hz\": {:.1},\n",
            "      \"ticks\": {},\n",
            "      \"nonzero_diff_count\": {},\n",
            "      \"max_abs_diff_lsb\": {},\n",
            "      \"nonzero_sum_count\": {},\n",
            "      \"max_abs_sum_lsb\": {}\n",
            "    }}\n",
            "  }}\n",
            "}}\n"
        ),
        mode.as_str(),
        cfg.n_sub_hz,
        cfg.n_nyquist,
        cfg.n_cordic,
        cfg.n_symmetry,
        a.sample_rate_hz,
        a.frequency_hz,
        a.ticks,
        a.non_monotonic_phase_steps,
        a.phase_u32_stutter_steps,
        a.max_abs_residual,
        a.residual_unit,
        b.sample_rate_hz,
        b.frequency_hz,
        b.ticks,
        b.min_sample,
        b.max_sample,
        b.dc_offset_samples,
        b.even_odd_abs_peak_delta,
        c.sample_rate_hz,
        c.frequency_hz,
        c.ticks,
        c.hit_i32_max_count,
        c.hit_i32_min_count,
        c.max_abs_normalized,
        c.clipping_active_count,
        d.sample_rate_hz,
        d.frequency_hz,
        d.ticks,
        d.nonzero_diff_count,
        d.max_abs_diff_lsb,
        d.nonzero_sum_count,
        d.max_abs_sum_lsb
    )
}

fn main() {
    let (mode, out_path) = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(2);
        }
    };

    let cfg = ProbeConfig::for_mode(mode);
    let sub_hz = run_sub_hz_probe(cfg.n_sub_hz);
    let nyq = run_nyquist_probe(cfg.n_nyquist);
    let cordic = run_cordic_probe(cfg.n_cordic);
    let sym = run_symmetry_probe(cfg.n_symmetry);
    let json = render_json(mode, cfg, sub_hz, nyq, cordic, sym);

    if let Err(e) = fs::write(&out_path, &json) {
        eprintln!("failed to write {}: {}", out_path, e);
        process::exit(1);
    }

    println!("{}", json);
}
