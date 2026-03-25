/*
 * PRECISION TRAVERSAL: THE AUDIO WALKTHROUGH
 * Hardware Target: Raspberry Pi 3B
 * Output: GPIO 18 (Mono Fuzz with Capture Effect)
 *
 * SCENARIO:
 * - Beacons are STATIC (Lighthouses).
 * - Observer MOVES through the field (-5000m -> +5000m).
 * - Aggressive Gain Curve forces the "Capture Effect" on the closest beacon.
 */

use dpw4::{DpwGain, Oscillator};
use geom_signal::Scalar;
use geom_spatial::Vector3;
use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
use rppal::gpio::{Gpio, OutputPin};

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
    let mut gpio_pin: Option<()> = None;

    println!("═══════════════════════════════════════════════════════");
    println!(" PRECISION TRAVERSAL: Audio Field Walkthrough");
    println!(" Observer Path: (-5000, 0, 0) -> (+5000, 0, 0)");
    println!(" Mechanism:     Inverse-Square Gain Capture (Fuzz)");
    println!(" Output:        GPIO 18 (1-bit Quantized)");
    println!("═══════════════════════════════════════════════════════");

    // Sample Rate for the world clock
    let sample_rate = SAMPLE_RATE_HZ as f64;

    // Unity Gain for the oscillators (we will scale the results manually for the Capture Effect)
    let unit_gain = DpwGain::new(1u64 << 63, 0, 0, 0);
    let sine_shape: u32 = 4;

    // 2. The Soundfield (3 Static Beacons)
    // Beacon A (Left Entry): Low Tone (100 Hz)
    // Offset Y by 50m to avoid singularity at perfectly zero distance
    let beacon_a = Vector3::new(
        Scalar::from_num(-2500.0),
        Scalar::from_num(50.0),
        Scalar::ZERO,
    );
    let mut osc_a = Oscillator::new_u32(SAMPLE_RATE_HZ);
    osc_a.frequency = Scalar::from_num(100.0);

    // Beacon B (Center): Mid Tone (500 Hz)
    let beacon_b = Vector3::new(Scalar::ZERO, Scalar::from_num(50.0), Scalar::ZERO);
    let mut osc_b = Oscillator::new_u32(SAMPLE_RATE_HZ);
    osc_b.frequency = Scalar::from_num(500.0);

    // Beacon C (Right Exit): High Tone (1000 Hz)
    let beacon_c = Vector3::new(
        Scalar::from_num(2500.0),
        Scalar::from_num(50.0),
        Scalar::ZERO,
    );
    let mut osc_c = Oscillator::new_u32(SAMPLE_RATE_HZ);
    osc_c.frequency = Scalar::from_num(1000.0);

    // 3. The Observer (Starts far left)
    let mut observer_pos = Vector3::new(Scalar::from_num(-5000.0), Scalar::ZERO, Scalar::ZERO);

    // Physics Constants
    let dt = 1.0 / sample_rate;
    // Walk speed: 1000 m/s (Traversal takes 10 seconds for 10km)
    let walk_speed = 1000.0;

    // Telemetry Setup
    let mut total_samples: u64 = 0;
    let mut bench_counter: u64 = 0;
    let start_time = Instant::now();
    let mut last_report = start_time;

    println!(">>> WALKING STARTED. Listen for tone hand-offs.");

    loop {
        // --- A. OFFSET INJECTION (Move the Observer) ---
        let current_x = observer_pos.x.to_num::<f64>();

        if current_x < 5000.0 {
            // Move forward linearly
            let new_x = current_x + (walk_speed * dt);
            observer_pos.x = Scalar::from_num(new_x);
        } else {
            // Loop back to start
            observer_pos.x = Scalar::from_num(-5000.0);
            println!(">>> [LOOP] Resetting to Start (-5000m)");
        }

        // --- B. GEOMETRY (128-bit Euclidean Measurements) ---
        let d_a = observer_pos.distance(&beacon_a);
        let d_b = observer_pos.distance(&beacon_b);
        let d_c = observer_pos.distance(&beacon_c);

        // --- C. GAIN CALCULATION (The Selector) ---
        // Aggressive Gain Curve: 1000 / (Distance + 1)
        // This ensures that when an observer is at 0m from a beacon, its gain (1000)
        // is orders of magnitude higher than its neighbors 2500m away.
        let g_a = 1000.0 / (d_a.to_num::<f64>() + 1.0);
        let g_b = 1000.0 / (d_b.to_num::<f64>() + 1.0);
        let g_c = 1000.0 / (d_c.to_num::<f64>() + 1.0);

        // --- D. MIXDOWN ---
        // Generate raw 32-bit samples and scale by proximity gain
        let s_a = osc_a.tick(sine_shape, &unit_gain) as f64 * g_a;
        let s_b = osc_b.tick(sine_shape, &unit_gain) as f64 * g_b;
        let s_c = osc_c.tick(sine_shape, &unit_gain) as f64 * g_c;

        let mix = s_a + s_b + s_c;

        // --- E. OUTPUT (1-Bit Quantizer / Capture Effect) ---
        #[cfg(target_os = "linux")]
        if let Some(ref mut pin) = gpio_pin {
            if mix > 0.0 {
                pin.set_high();
            } else {
                pin.set_low();
            }
        }

        // --- F. TELEMETRY ---
        total_samples += 1;
        bench_counter += 1;

        if (bench_counter & 0xFFFF) == 0 {
            let now = Instant::now();
            let elapsed = now.duration_since(last_report);
            if elapsed >= Duration::from_millis(10) {
                let rate = bench_counter as f64 / elapsed.as_secs_f64();
                println!(
                    "Pos: {:6.0}m | Dist: A:{:5.0} B:{:5.0} C:{:5.0} | Dom: {} | Rate: {:.1} kHz",
                    current_x,
                    d_a.to_num::<f64>(),
                    d_b.to_num::<f64>(),
                    d_c.to_num::<f64>(),
                    if g_a > g_b && g_a > g_c {
                        "A (100Hz) "
                    } else if g_b > g_a && g_b > g_c {
                        "B (500Hz) "
                    } else {
                        "C (1kHz)  "
                    },
                    rate / 1000.0
                );
                bench_counter = 0;
                last_report = now;
            }
        }

        // Simulation exit: Traversal takes 10s at 1000m/s for 10km field
        if total_samples > 480000 {
            println!("\n[Simulation Exit: 10s traversal complete]");
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
