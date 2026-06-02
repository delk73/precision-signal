/*
 * PRECISION ANIMATOR: REAL-TIME WAVE PROPAGATION
 * Hardware Target: VT100 Terminal
 * Method: ANSI Escape Codes (Zero-Dependency TUI)
 *
 * SIMULATION:
 * - 3 Beacons with constructive interference.
 * - Ripples propagate outward (Time-varying phase).
 * - "Matrix-style" raw visualization of the 128-bit soundfield.
 */

use geom_signal::Scalar;
use geom_spatial::Vector3;
use std::io::{stdout, Write};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    // 1. Setup the Screen
    // Hide Cursor, Clear Screen
    print!("\x1b[?25l\x1b[2J");

    // 2. The Soundfield (Static Beacons)
    let beacon_a = Vector3::new(Scalar::from_num(-2500.0), Scalar::ZERO, Scalar::ZERO);
    let beacon_b = Vector3::new(Scalar::ZERO, Scalar::ZERO, Scalar::ZERO);
    let beacon_c = Vector3::new(Scalar::from_num(2500.0), Scalar::ZERO, Scalar::ZERO);

    // 3. Resolution & Physics
    let width = 80;
    let height = 40;
    let scale_x = 10000.0 / width as f64;
    let scale_y = 5000.0 / height as f64;
    let offset_x = 5000.0;
    let offset_y = 2500.0;
    let k_factor = 0.006; // Controls ripple tightness

    // 4. Animation Loop
    let start = Instant::now();
    let frame_duration = Duration::from_millis(33); // ~30 FPS

    loop {
        let now = Instant::now();
        let t = now.duration_since(start).as_secs_f64();

        // Move the cursor to Home (0,0) - No flickering!
        let mut buffer = String::with_capacity(width * height + 512);
        buffer.push_str("\x1b[H");

        // Header
        buffer.push_str(
            " ╔══════════════════════════════════════════════════════════════════════════════╗\n",
        );
        buffer.push_str(&format!(
            " ║ PRECISION RULER: T={:6.2}s                                                 ║\n",
            t
        ));

        // --- RENDER PASS ---
        for y in 0..height {
            buffer.push_str(" ║");
            for x in 0..width {
                // Map Coordinates
                let world_x = (x as f64 * scale_x) - offset_x;
                let world_y = (y as f64 * scale_y) - offset_y;
                let pos = Vector3::new(
                    Scalar::from_num(world_x),
                    Scalar::from_num(world_y),
                    Scalar::ZERO,
                );

                // Geometry
                let d_a = pos.distance(&beacon_a).to_num::<f64>();
                let d_b = pos.distance(&beacon_b).to_num::<f64>();
                let d_c = pos.distance(&beacon_c).to_num::<f64>();

                // Check for Hardware Impact
                if d_a < 150.0 || d_b < 150.0 || d_c < 150.0 {
                    buffer.push('●');
                    continue;
                }

                // Wave Physics (Propagating Phase)
                // - t * speed makes it move
                // - Gain decays with distance
                let g_a = 2000.0 / (d_a + 1.0);
                let g_b = 2000.0 / (d_b + 1.0);
                let g_c = 2000.0 / (d_c + 1.0);

                // Use slightly different frequencies for Moiré beating
                let s_a = (d_a * k_factor * 0.9 - (t * 10.0)).sin() * g_a;
                let s_b = (d_b * k_factor * 1.0 - (t * 10.0)).sin() * g_b;
                let s_c = (d_c * k_factor * 1.1 - (t * 10.0)).sin() * g_c;

                let mix = s_a + s_b + s_c;

                // Density Gradient (The "Fuzz" Visualizer)
                let char = if mix > 0.8 {
                    '#'
                }
                // Peak
                else if mix > 0.4 {
                    '='
                } else if mix > 0.1 {
                    '-'
                } else if mix > -0.1 {
                    ' '
                }
                // Zero-crossing (Silence)
                else if mix > -0.4 {
                    '.'
                } else if mix > -0.8 {
                    ':'
                } else {
                    '@'
                }; // Trough

                buffer.push(char);
            }
            buffer.push_str("║\n");
        }
        buffer.push_str(
            " ╚══════════════════════════════════════════════════════════════════════════════╝\n",
        );

        // Flush Buffer to Screen
        print!("{}", buffer);
        let _ = stdout().flush();

        // Frame Limiter
        let elapsed = now.elapsed();
        if elapsed < frame_duration {
            thread::sleep(frame_duration - elapsed);
        }

        // Optional: Break after some time for testing, but typically animators loop forever
        // For our case, we let it run until interrupted.
    }
}
