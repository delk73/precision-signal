use dpw4::tick_dpw4_raw;
use std::env;

/// Scaling factor for DPW4 (1/24)
const C4: f64 = 1.0 / 24.0;
const Q124_LSB: f64 = 4.70197740328915e-38;

// Shadow Reference logic
struct RefState {
    z1: f64,
    z2: f64,
    z3: f64,
}

fn tick_ref(state: &mut RefState, phase_01: f64) -> f64 {
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

fn main() {
    let args: Vec<String> = env::args().collect();
    let pro_audit = args.contains(&"--pro-audit".to_string());

    let num_samples: u64 = if pro_audit { 1_000_000_000 } else { 1_000_000 };
    let freq = 0.1;
    let sample_rate = 48000.0;

    println!(
        "Starting {} sample Stability Test (0.1 Hz @ 48kHz)...",
        num_samples
    );
    if pro_audit {
        println!("PRO-AUDIT MODE: Billion sample verification engaged.");
    }

    let mut state_i128 = dpw4::Dpw4State::new();
    let mut state_ref = RefState {
        z1: 0.0,
        z2: 0.0,
        z3: 0.0,
    };

    let phase_inc = (libm::floor((freq / sample_rate) * (u64::MAX as f64) + 0.5)) as u64;
    let mut phase: u64 = 0;

    let mut cumulative_dc: f64 = 0.0;
    let mut max_residual: f64 = 0.0;

    // We don't verify the i16 output for DC drift (too coarse),
    // we verify the raw i128 scaled to f64.

    let report_interval = num_samples / 10;

    for i in 0..num_samples {
        // Core Path
        let s_q31 = ((phase >> 32) as i64).wrapping_sub(1i64 << 31);
        let raw_i128 = tick_dpw4_raw(&mut state_i128, s_q31);
        let i128_scaled = (raw_i128 as f64) * Q124_LSB * C4;

        // Shadow Path
        let ph_01 = (phase >> 32) as f64 / 4294967296.0;
        let out_ref = tick_ref(&mut state_ref, ph_01);

        // Analysis
        cumulative_dc += i128_scaled;
        let residual = libm::fabs(i128_scaled - out_ref);
        if residual > max_residual {
            max_residual = residual;
        }

        if i % report_interval == 0 && i > 0 {
            println!("Progress: {}% ({} samples)", (i * 100) / num_samples, i);
        }

        phase = phase.wrapping_add(phase_inc);
    }

    let avg_dc = cumulative_dc / (num_samples as f64);

    println!("\nAudit Results:");
    println!("Samples processed: {}", num_samples);
    println!("Max internal residual: {:.18e}", max_residual);
    println!("Average DC offset: {:.18e}", avg_dc);

    // Assertions
    if avg_dc.abs() > 1e-14 {
        println!(
            "FAIL: DC offset threshold exceeded ({} > 1e-14)",
            avg_dc.abs()
        );
        std::process::exit(1);
    }

    if max_residual > 1e-7 {
        println!(
            "FAIL: Residual threshold exceeded ({} > 1e-7)",
            max_residual
        );
        std::process::exit(1);
    }

    println!("\nSUCCESS: Reference Lock Batch Audit Passed.");
}
