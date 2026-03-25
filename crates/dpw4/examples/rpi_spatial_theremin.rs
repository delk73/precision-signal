use dpw4::{DpwGain, Oscillator};
use geom_signal::Scalar;
use geom_spatial::Vector3;

#[cfg(target_os = "linux")]
use rppal::gpio::{Gpio, OutputPin};

use std::time::Instant;

/// The Spatial Theremin: First "World-Aware" Audio Application
///
/// This example demonstrates real-time 3D spatial audio synthesis by mapping
/// Euclidean distance to frequency and gain. A virtual "Drone" orbits an
/// observer in an eccentric elliptical path, generating continuous audio
/// feedback based on its distance.
///
/// ## The Game Design
///
/// - **Observer**: Fixed at origin (0, 0, 0)
/// - **Drone**: Orbits with radius varying from 100m to 10,000m (eccentric ellipse)
/// - **Distance → Frequency**: `Freq = 100.0 + (Distance / 10.0)` Hz
/// - **Distance → Gain**: `Gain = 1.0 / (1.0 + Distance/100.0)` (inverse-square simulation)
///
/// ## Validation Goal
///
/// Prove that Raspberry Pi 3B can compute:
/// 1. 3D Euclidean Distance (128-bit CORDIC sqrt)
/// 2. CORDIC Sine Wave generation
///
/// ...within a single 20μs audio window (48kHz sample rate) without dropouts.
///
/// ## Hardware Output
///
/// - **GPIO 18**: PWM/Square wave (MSB of phase)
/// - **Stdout**: Telemetry (distance, frequency, gain, throughput)
fn main() {
    const SAMPLE_RATE_HZ: u32 = 48_000;

    // Initialize GPIO for hardware output (Pi-specific)
    #[cfg(target_os = "linux")]
    let mut gpio_pin = match init_gpio() {
        Ok(pin) => Some(pin),
        Err(e) => {
            println!("[Warning: GPIO initialization failed: {}]", e);
            println!("[Running in simulation mode without hardware output]\n");
            None
        }
    };

    #[cfg(not(target_os = "linux"))]
    let gpio_pin: Option<()> = None;

    // Spatial Setup
    let observer = Vector3::new(Scalar::ZERO, Scalar::ZERO, Scalar::ZERO);

    // Oscillator Setup (CORDIC Sine path)
    let sample_rate = SAMPLE_RATE_HZ as f64; // 48kHz
    let mut osc = Oscillator::new_u32(SAMPLE_RATE_HZ);

    // Use shape index 4 for Sine (see tick_shape in lib.rs)
    let sine_shape: u32 = 4;

    // Orbit Parameters
    let orbit_period = 10.0; // Complete orbit every 10 seconds
    let orbit_freq = 1.0 / orbit_period; // 0.1 Hz
    let _min_radius = Scalar::from_num(100.0); // 100 meters (documentary)
    let _max_radius = Scalar::from_num(10_000.0); // 10 km (documentary)

    // Telemetry & Timing
    let mut bench_counter: u64 = 0;
    let mut total_samples: u64 = 0;
    let mut last_report = Instant::now();
    let report_interval = std::time::Duration::from_secs(1);

    println!("═══════════════════════════════════════════════════════");
    println!("  THE SPATIAL THEREMIN: First World-Aware Audio Demo");
    println!("═══════════════════════════════════════════════════════");
    println!("Observer:     (0, 0, 0)");
    println!("Drone Orbit:  100m → 10,000m (eccentric ellipse)");
    println!("Sample Rate:  48 kHz");
    println!("Audio Path:   128-bit CORDIC Sine");
    println!("Hardware:     GPIO 18 (Square Wave Output)");
    println!("═══════════════════════════════════════════════════════\n");

    loop {
        // 1. Compute Drone Position (Eccentric Elliptical Orbit)
        // Use total_samples for a persistent simulation clock
        let time = total_samples as f64 / sample_rate;
        let orbit_phase = 2.0 * std::f64::consts::PI * orbit_freq * time;

        // Eccentric orbit: radius varies sinusoidally
        let radius_variation = (orbit_phase.sin() + 1.0) / 2.0; // 0.0 → 1.0
        let radius_f64 = 100.0 + (10_000.0 - 100.0) * radius_variation;
        let radius = Scalar::from_num(radius_f64);

        // Position on circular path (XY plane)
        let x = radius * Scalar::from_num(orbit_phase.cos());
        let y = radius * Scalar::from_num(orbit_phase.sin());
        let z = Scalar::ZERO;

        let drone = Vector3::new(x, y, z);

        // 2. Compute 3D Euclidean Distance (THE KEY OPERATION)
        let distance = drone.distance(&observer);

        // 3. Map Distance → Frequency
        // Freq = 100.0 + (Distance / 10.0)
        let distance_f64 = distance.to_num::<f64>();
        let frequency = 100.0 + (distance_f64 / 10.0);

        // 4. Map Distance → Gain (Inverse-Square Simulation)
        // Gain = 1.0 / (1.0 + Distance/100.0)
        let gain_f64 = 1.0 / (1.0 + distance_f64 / 100.0);

        // Create DPW gain (simplified - use mantissa at Q63)
        // For m4_q63: 1.0 in Q63 = 1 << 63
        // We scale gain_f64 by 2^63
        let m4_q63 = (gain_f64 * (1u64 << 63) as f64) as u64;
        let gain = DpwGain::new(m4_q63, 0, 0, 0);

        // 5. Update oscillator frequency
        osc.frequency = Scalar::from_num(frequency);

        // 6. Generate Audio Sample
        let _sample = osc.tick(sine_shape, &gain);

        // 7. Hardware Output (GPIO Toggle based on Phase MSB)
        #[cfg(target_os = "linux")]
        if let Some(ref mut pin) = gpio_pin {
            // Access phase directly from oscillator
            let phase_bits = osc.phase.to_bits();
            let phase_msb = (phase_bits >> 127) & 1;
            if phase_msb != 0 {
                pin.set_high();
            } else {
                pin.set_low();
            }
        }

        // 8. Telemetry & Counters
        bench_counter += 1;
        total_samples += 1;

        if (bench_counter & 0xFFFF) == 0 {
            let now = Instant::now();
            if now.duration_since(last_report) >= report_interval {
                let elapsed = now.duration_since(last_report).as_secs_f64();
                let sample_rate_actual = bench_counter as f64 / elapsed;

                println!(
                    "Distance: {:8.1} m | Freq: {:6.1} Hz | Gain: {:.3} | Rate: {:.1} kHz | Time: {:.1}s",
                    distance_f64,
                    frequency,
                    gain_f64,
                    sample_rate_actual / 1000.0,
                    time
                );

                bench_counter = 0;
                last_report = now;
            }
        }

        // Prevent runaway loop in simulation mode
        #[cfg(not(target_os = "linux"))]
        if total_samples > 480_000 {
            println!("\n[Simulation mode: Audio Time {:.1}s reached]", time);
            break;
        }
    }
}

#[cfg(target_os = "linux")]
fn init_gpio() -> Result<OutputPin, String> {
    let gpio = Gpio::new().map_err(|e| format!("{}", e))?;
    let pin = gpio.get(18).map_err(|e| format!("{}", e))?;
    Ok(pin.into_output())
}
