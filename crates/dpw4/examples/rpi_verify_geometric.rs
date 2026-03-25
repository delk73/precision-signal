use dpw4::{math, signal_pipe, DpwGain, OscState, Scalar, Sine};
use rppal::gpio::Gpio;
use std::time::Instant;

/// Protocol Level 0: Geometric Validation Harness
///
/// This harness verifies the performance cost of the 128-bit CORDIC engine
/// on the Raspberry Pi. It runs the Sine waveform generator flat out.
///
/// ADVISORY: This is a TIMING/HARDWARE harness.
/// - It does not validate waveform correctness.
/// - It must not be cited for DPW conformance or SHA-256 evidence.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. The Fixed "Gear Ratio" (Calculated for 440Hz @ 48kHz)
    let freq_audio = 440.0;
    let rate_audio = 48_000.0;
    let freq_scalar = Scalar::from_num(freq_audio);
    let rate_scalar = Scalar::from_num(rate_audio);
    let increment = (freq_scalar / rate_scalar) * math::TWO_PI;

    // 2. Setup Hardware (Resilient)
    let gpio_result = Gpio::new();
    let mut pin = match gpio_result {
        Ok(gpio) => {
            println!("Hardware:   Detected Raspberry Pi");
            Some(gpio.get(18)?.into_output())
        }
        Err(e) => {
            println!("Hardware:   MOCK MODE ({})", e);
            None
        }
    };

    // 3. Initialize Core
    let mut osc = OscState::new();
    let gain = DpwGain::default();
    let mut phase = Scalar::ZERO;

    // Buffers for signal_pipe (to simulate real load)
    let mut phase_buf = [Scalar::ZERO; 1];
    let mut out_buf = [0i32; 1];

    // 4. Instrumentation Setup
    let mut loop_counter: u64 = 0;
    let mut last_report = Instant::now();
    let report_interval = std::time::Duration::from_secs(1);

    println!("Protocol Level 0: Geometric Validation Harness");
    println!("---------------------------------------------------");
    println!("Signal:       Sine (Geometric CORDIC)");
    println!("Target Ratio: 440Hz / 48kSps");
    println!("Pin Output:   GPIO 18 (MSB - Square Wave)");
    println!("Note:         Frequency will scale with CPU speed.");
    println!("---------------------------------------------------");

    loop {
        // A. Drive Hardware (Phase MSB - Digital Square Wave)
        if let Some(ref mut p) = pin {
            if phase >= math::PI {
                p.set_high();
            } else {
                p.set_low();
            }
        }

        // B. Drive Core (Math Manifestation - Load Simulation)
        phase_buf[0] = phase;
        signal_pipe::<Sine>(&mut osc, &phase_buf, &gain, &mut out_buf);

        // C. Advance State
        phase += increment;
        if phase >= math::TWO_PI {
            phase -= math::TWO_PI;
        }

        // D. Instrumentation
        loop_counter += 1;

        if (loop_counter & 0xFFFF) == 0 {
            let now = Instant::now();
            if now.duration_since(last_report) >= report_interval {
                let elapsed = now.duration_since(last_report).as_secs_f64();
                let loop_rate = loop_counter as f64 / elapsed;

                let predicted_freq = loop_rate * (freq_audio / rate_audio);

                println!(
                    "Engine: {:.3} MHz | Scope Freq: {:.3} kHz",
                    loop_rate / 1_000_000.0,
                    predicted_freq / 1_000.0
                );

                loop_counter = 0;
                last_report = now;
            }
        }
    }
}
