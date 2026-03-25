use dpw4::{math, signal_pipe, DpwGain, OscState, Sawtooth, Scalar};
use rppal::gpio::Gpio;
use std::time::Instant;

/// Observation Level A — Digital Adapter (GPIO)
///
/// Observation levels describe measurement layers, not protocol layers.
/// PWM is an adapter-level choice on Raspberry Pi hardware and is not part
/// of the Precision-DPW reference protocol.
///
/// This harness runs the core "flat out" (no sleep) to validate the
/// logical gearing ratio of the Phase Accumulator under maximum load.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. The Fixed "Gear Ratio" (Calculated for 440Hz @ 48kHz)
    // We keep the ratio fixed even if the loop runs faster, so we can
    // measure the "Speedup Factor" on the scope.
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

    // 3. Initialize Core (New API)
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

    println!("Observation Level A: Digital Adapter (GPIO)");
    println!("---------------------------------------------------");
    println!("Target Ratio: 440Hz / 48kSps");
    println!("Pin Output:   GPIO 18 (GPIO DAC adapter output - PWM or PDM)");
    println!("Note:         Frequency will scale with CPU speed.");
    println!("---------------------------------------------------");

    loop {
        // A. Drive Hardware (Phase MSB - Digital Square Wave)
        if let Some(ref mut p) = pin {
            // Scalar phase in [0, TWO_PI). MSB is set if phase >= PI.
            if phase >= math::PI {
                p.set_high();
            } else {
                p.set_low();
            }
        }

        // B. Drive Core (Math Manifestation - Load Simulation)
        phase_buf[0] = phase;
        signal_pipe::<Sawtooth>(&mut osc, &phase_buf, &gain, &mut out_buf);

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

                // If we run faster than 48k, the pitch goes up.
                // This predicts what the scope sees.
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
