use dpw4::{DpwGain, Oscillator, Scalar};
use rppal::gpio::{Gpio, OutputPin};
use std::time::{Duration, Instant};

/// Observation Level B — Analog Reconstruction (RC Filter)
///
/// Observation levels describe measurement layers, not protocol layers.
/// This harness runs a "Spin-Locked" loop to approximate 48kSps
/// timing while driving a GPIO DAC adapter output (PDM) to the RC Filter.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Configuration
    const TARGET_FREQ: f64 = 440.0;
    const SAMPLE_RATE_HZ: u32 = 48_000;
    const SAMPLE_RATE: f64 = SAMPLE_RATE_HZ as f64;
    const TARGET_PERIOD: Duration = Duration::from_nanos(20_833); // 1/48000 sec

    // 2. Hardware Setup
    let gpio_result = Gpio::new();
    let mut pin: Option<OutputPin> = match gpio_result {
        Ok(gpio) => {
            println!("Hardware:   Detected Raspberry Pi");
            Some(gpio.get(18)?.into_output())
        }
        Err(e) => {
            println!("Hardware:   MOCK MODE ({})", e);
            None
        }
    };

    // 3. Core Setup (32-bit Reference)
    let mut osc = Oscillator::new_u32(SAMPLE_RATE_HZ);
    osc.frequency = Scalar::from_num(TARGET_FREQ);
    osc.sample_rate = Scalar::from_num(SAMPLE_RATE); // Explicitly ensure alignment

    // Gain: -3dB (0.707) to avoid PDM saturation
    let gain = DpwGain::new(6_521_722_912_506_847_232, 0, 0, 0);

    // 4. PDM State
    let mut pdm_error: i64 = 0;
    const PDM_MAX: i64 = 4_294_967_296; // 2^32

    // 5. Loop State
    let mut next_tick = Instant::now();

    println!("Observation Level B: Analog Reconstruction (RC Filter)");
    println!("---------------------------------------------------");
    println!("Signal:       Sawtooth (DPW4)");
    println!(
        "Target:       {:.1}Hz @ {:.1}kHz",
        TARGET_FREQ,
        SAMPLE_RATE / 1000.0
    );
    println!("Validation:   Probe 'Analog Out' on Reference RC Circuit");
    println!("---------------------------------------------------");

    loop {
        // A. Spin Lock (Force 48kHz)
        while Instant::now() < next_tick {
            std::hint::spin_loop();
        }
        next_tick += TARGET_PERIOD;

        // B. Generate 32-bit Sample
        let sample = osc.tick(0, &gain); // Shape 0 = Sawtooth

        // C. PDM Modulation (1-bit DAC)
        let sample_offset = sample as i64 + 2_147_483_648;
        pdm_error += sample_offset;

        if let Some(ref mut p) = pin {
            if pdm_error >= PDM_MAX {
                p.set_high();
                pdm_error -= PDM_MAX;
            } else {
                p.set_low();
            }
        }
    }
}
