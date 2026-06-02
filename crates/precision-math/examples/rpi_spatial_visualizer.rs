/*
 * PRECISION CONSOLE: THE WAVE SCANNER
 * Hardware Target: Any Terminal (Standard Output)
 * Resolution: 80x40 characters (Low-Res/High-Clarity)
 *
 * PURPOSE:
 * - Visual proof of 128-bit Geometric Synthesis.
 * - Shows the interference pattern (ripples) of the 3 beacons.
 * - Corrected spatial frequency to avoid aliasing on text screens.
 */

use geom_signal::Scalar;
use geom_spatial::Vector3;
use std::io::Write;

fn main() {
    println!("════════════════════════════════════════════════════════════════");
    println!(" PRECISION SCANNER: 128-bit Interference Pattern");
    println!(" [A]=Left  [B]=Center  [C]=Right");
    println!(" Rendering 10km x 5km field...");
    println!("════════════════════════════════════════════════════════════════");

    // 1. The Soundfield (Static Beacons)
    // We space them out to ensure their wave pools barely touch
    let beacon_a = Vector3::new(Scalar::from_num(-2500.0), Scalar::ZERO, Scalar::ZERO);
    let beacon_b = Vector3::new(Scalar::ZERO, Scalar::ZERO, Scalar::ZERO);
    let beacon_c = Vector3::new(Scalar::from_num(2500.0), Scalar::ZERO, Scalar::ZERO);

    // 2. Console Canvas Setup
    let width = 80;
    let height = 40;

    // Map the world to the console window
    // X: -5000m to +5000m
    // Y: -2500m to +2500m
    let scale_x = 10000.0 / width as f64;
    let scale_y = 5000.0 / height as f64;
    let offset_x = 5000.0;
    let offset_y = 2500.0;

    // 3. Visualization Tuning
    // K-Factor: Controls the "tightness" of the ripples.
    // k = 2*PI / Wavelength.
    // We want a Wavelength of ~1000m to look good in ASCII.
    // k = 6.28 / 1000 = 0.00628
    let k_factor = 0.006;

    // 4. The Scan Loop
    for y in 0..height {
        let mut line = String::with_capacity(width);

        for x in 0..width {
            // Screen -> World Coordinates
            let world_x = (x as f64 * scale_x) - offset_x;
            let world_y = (y as f64 * scale_y) - offset_y;
            let pos = Vector3::new(
                Scalar::from_num(world_x),
                Scalar::from_num(world_y),
                Scalar::ZERO,
            );

            // A. Geometry (128-bit Distance)
            let d_a = pos.distance(&beacon_a);
            let d_b = pos.distance(&beacon_b);
            let d_c = pos.distance(&beacon_c);

            // Check for "Impact" (Is this pixel literally inside a beacon?)
            let dist_a_f = d_a.to_num::<f64>();
            let dist_b_f = d_b.to_num::<f64>();
            let dist_c_f = d_c.to_num::<f64>();

            // Character to represent a beacon hardware unit
            if dist_a_f < 200.0 || dist_b_f < 200.0 || dist_c_f < 200.0 {
                line.push('@');
                continue;
            }

            // B. Gain (Inverse Square)
            let g_a = 2000.0 / (dist_a_f + 1.0);
            let g_b = 2000.0 / (dist_b_f + 1.0);
            let g_c = 2000.0 / (dist_c_f + 1.0);

            // C. Phase Calculation (The Ripple)
            // We use slightly different frequencies to create "Beating" interference
            let s_a = (dist_a_f * k_factor * 0.9).sin() * g_a;
            let s_b = (dist_b_f * k_factor * 1.0).sin() * g_b;
            let s_c = (dist_c_f * k_factor * 1.1).sin() * g_c;

            // Sum the energy
            let mix = s_a + s_b + s_c;

            // D. ASCII Quantizer
            // We map the Wave Height to a character gradient
            // to show peaks and troughs.
            let c = if mix > 0.8 {
                '#' // Peak (High Energy)
            } else if mix > 0.4 {
                '=' // Rising
            } else if mix > 0.0 {
                '-' // Zero Crossing (+)
            } else if mix > -0.4 {
                '.' // Zero Crossing (-)
            } else {
                ' ' // Trough (Silence)
            };

            line.push(c);
        }
        println!("{}", line);
        let _ = std::io::stdout().flush();
    }
    println!("════════════════════════════════════════════════════════════════");
}
