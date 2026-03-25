use dpw4::{apply_gain, tick_dpw4_raw, DpwGain, OscState};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// verify_nano_trace: Comparison utility for hardware-optimized traces.
/// This tool compares external PCM/CSV traces against the Q124 Reference.
fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run --example verify_nano_trace -- <trace.csv>");
        println!("Expected CSV format: phase_u64, sample_i16");
        return Ok(());
    }

    let file = File::open(&args[1])?;
    let reader = BufReader::new(file);

    let mut state = OscState::new();
    state.reset();

    // Normalized Reference Gain
    let c4 = 1.0 / 24.0;
    let m4_q63 = (libm::floor(c4 * 9223372036854775808.0 + 0.5)) as u64;
    let gain = DpwGain::new(m4_q63, 15, 0, 0); // exponent 15 to map to i16

    let mut count = 0;
    let mut total_deviation: f64 = 0.0;
    let mut max_deviation: i32 = 0;

    for (line_idx, line) in reader.lines().enumerate() {
        let l = line?;
        if l.starts_with('#') || l.is_empty() || l.starts_with("phase") {
            continue;
        }

        let parts: Vec<&str> = l.split(',').map(|s| s.trim()).collect();
        if parts.len() < 2 {
            continue;
        }

        let phase: u64 = match parts[0].parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!(
                    "Warning: Invalid phase at line {}: '{}'",
                    line_idx + 1,
                    parts[0]
                );
                continue;
            }
        };

        let nano_sample: i16 = match parts[1].parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!(
                    "Warning: Invalid sample at line {}: '{}'",
                    line_idx + 1,
                    parts[1]
                );
                continue;
            }
        };

        // Compute Reference
        let s_q31 = ((phase >> 32) as i64).wrapping_sub(1i64 << 31);
        let raw = tick_dpw4_raw(&mut state.saw_a, s_q31);
        let ref_sample = apply_gain(raw, gain.m4_q63, gain.e4);

        let dev = (ref_sample as i32 - nano_sample as i32).abs();
        total_deviation += dev as f64;
        if dev > max_deviation {
            max_deviation = dev;
        }

        count += 1;
    }

    if count == 0 {
        println!("No valid samples found in trace.");
        return Ok(());
    }

    println!("Verification Results for '{}'", args[1]);
    println!("Samples compared: {}", count);
    println!("Max Deviation: {} LSBy", max_deviation);
    println!("Avg Deviation: {:.6} LSBy", total_deviation / count as f64);

    if max_deviation == 0 {
        println!("STATUS: BIT-EXACT MATCH. Trace is mathematically identical to Q124 Reference.");
    } else if max_deviation < 5 {
        println!("STATUS: CONFORMANT. Minor quantization differences observed.");
    } else {
        println!("STATUS: NON-CONFORMANT. Significant divergence from Reference.");
    }

    Ok(())
}
