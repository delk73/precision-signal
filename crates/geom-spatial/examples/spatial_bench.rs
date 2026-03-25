use geom_signal::Scalar;
use geom_spatial::Vector3;
use std::time::Instant;

/// Protocol Level 0: Spatial Validation Harness
///
/// This harness benchmarks the 128-bit Vector3 distance calculation.
/// It represents the "Cost of Space" in the geometric paradigm.
fn main() {
    // 1. Setup Simulation Data
    let observer = Vector3::new(Scalar::ZERO, Scalar::ZERO, Scalar::ZERO);

    // 2. Instrumentation Setup
    let mut loop_counter: u64 = 0;
    let mut last_report = Instant::now();
    let report_interval = std::time::Duration::from_secs(1);

    // Movement Step (Pre-converted to Scalar)
    let step = geom_signal::Scalar::from_bits(0x100000000);
    let mut x_scalar = geom_signal::Scalar::ZERO;

    // Architecture Detection for Honest Reporting
    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64 / Intel/AMD"
    } else if cfg!(target_arch = "aarch64") {
        "AARCH64 / ARM (Raspberry Pi/SBC)"
    } else if cfg!(target_arch = "arm") {
        "ARMv7 / 32-bit (Legacy Pi)"
    } else {
        "Unknown Architecture"
    };

    println!("Protocol Level 0: Spatial Validation Harness (Ultra-Lean)");
    println!("---------------------------------------------------");
    println!("Operation:    Vector3::distance (128-bit Sqrt)");
    println!("Architecture: {}", arch);
    println!("---------------------------------------------------");

    loop {
        // A. Update Movement (Pure Scalar Addition - Zero Cost)
        x_scalar += step;
        let source = Vector3 {
            x: x_scalar,
            y: geom_signal::Scalar::ZERO,
            z: geom_signal::Scalar::ZERO,
        };

        // B. Logic Under Test (Geometric Spatial Solving)
        // This exercises the refactored .max().max() logic in Vector3
        let _dist = source.distance(&observer);

        // C. Instrumentation
        loop_counter += 1;

        // Report every 64k iterations to minimize timing overhead
        if (loop_counter & 0xFFFF) == 0 {
            let now = Instant::now();
            if now.duration_since(last_report) >= report_interval {
                let elapsed = now.duration_since(last_report).as_secs_f64();
                let loop_rate = loop_counter as f64 / elapsed;

                println!(
                    "Spatial Engine: {:.3} MHz | Ops/Sec: {:.0}",
                    loop_rate / 1_000_000.0,
                    loop_rate
                );

                loop_counter = 0;
                last_report = now;
            }
        }
    }
}
