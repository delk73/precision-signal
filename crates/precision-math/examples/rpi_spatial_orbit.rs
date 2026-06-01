/*
 * PRECISION ORBIT: DYNAMIC GRAVITATIONAL AUDIO
 * Hardware Target: VT100 Terminal
 *
 * SIMULATION:
 * - Beacons A & C orbit the center (Binary Star).
 * - Beacon B oscillates vertically (Unstable Core).
 * - Interference patterns rotate and shear in real-time.
 */

use geom_signal::Scalar;
use geom_spatial::Vector3;
use std::io::{stdout, Write};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    // 1. Setup
    print!("\x1b[?25l\x1b[2J"); // Hide cursor, Clear

    // 2. Canvas Resolution
    let width = 80;
    let height = 40;

    // Viewport: 10km wide, 5km tall
    let scale_x = 10000.0 / width as f64;
    let scale_y = 5000.0 / height as f64;
    let offset_x = 5000.0;
    let offset_y = 2500.0;

    let k_factor = 0.006; // Ripple tightness

    let start = Instant::now();
    let frame_duration = Duration::from_millis(33);

    // 3. The Infinite Loop
    loop {
        let now = Instant::now();
        let t = now.duration_since(start).as_secs_f64();

        // --- PHYSICS UPDATE (Orbital Mechanics) ---

        // Beacons A & C: Counter-Clockwise Orbit (0.5 rad/s)
        let orbit_radius = 2000.0;
        let orbit_speed = 0.5;

        let a_x = orbit_radius * (t * orbit_speed).cos();
        let a_y = orbit_radius * (t * orbit_speed).sin();

        // C is exactly opposite to A (+PI)
        let c_x = orbit_radius * (t * orbit_speed + std::f64::consts::PI).cos();
        let c_y = orbit_radius * (t * orbit_speed + std::f64::consts::PI).sin();

        // Beacon B: Vertical Oscillation (The "Piston")
        let b_y = 1500.0 * (t * 1.5).sin();

        // Apply positions
        let beacon_a = Vector3::new(Scalar::from_num(a_x), Scalar::from_num(a_y), Scalar::ZERO);
        let beacon_b = Vector3::new(Scalar::ZERO, Scalar::from_num(b_y), Scalar::ZERO);
        let beacon_c = Vector3::new(Scalar::from_num(c_x), Scalar::from_num(c_y), Scalar::ZERO);

        // --- RENDERER ---

        let mut buffer = String::with_capacity(width * height + 512);
        buffer.push_str("\x1b[H"); // Cursor Home

        buffer.push_str(
            " ╔══════════════════════════════════════════════════════════════════════════════╗\n",
        );
        buffer.push_str(&format!(
            " ║ PRECISION ORBIT: T={:6.2}s  [Orbiting Beacons Active]                      ║\n",
            t
        ));

        for y in 0..height {
            buffer.push_str(" ║");
            for x in 0..width {
                // Screen -> World
                let world_x = (x as f64 * scale_x) - offset_x;
                let world_y = (y as f64 * scale_y) - offset_y;
                let pos = Vector3::new(
                    Scalar::from_num(world_x),
                    Scalar::from_num(world_y),
                    Scalar::ZERO,
                );

                // 128-bit Distance Checks
                let d_a = pos.distance(&beacon_a).to_num::<f64>();
                let d_b = pos.distance(&beacon_b).to_num::<f64>();
                let d_c = pos.distance(&beacon_c).to_num::<f64>();

                // Render Hardware (The Dots)
                if d_a < 150.0 || d_c < 150.0 {
                    buffer.push('O'); // Orbiters
                    continue;
                }
                if d_b < 150.0 {
                    buffer.push('+'); // The Core
                    continue;
                }

                // Wave Physics
                let g_a = 2000.0 / (d_a + 1.0);
                let g_b = 2000.0 / (d_b + 1.0);
                let g_c = 2000.0 / (d_c + 1.0);

                // Ripples propagate outward (-t * 10.0)
                let s_a = (d_a * k_factor * 0.9 - (t * 10.0)).sin() * g_a;
                let s_b = (d_b * k_factor * 1.0 - (t * 10.0)).sin() * g_b;
                let s_c = (d_c * k_factor * 1.1 - (t * 10.0)).sin() * g_c;

                let mix = s_a + s_b + s_c;

                // Density Map
                let char = if mix > 0.8 {
                    '█'
                } else if mix > 0.5 {
                    '▓'
                } else if mix > 0.2 {
                    '▒'
                } else if mix > -0.2 {
                    '░'
                } else if mix > -0.5 {
                    '·'
                } else {
                    ' '
                };

                buffer.push(char);
            }
            buffer.push_str("║\n");
        }
        buffer.push_str(
            " ╚══════════════════════════════════════════════════════════════════════════════╝\n",
        );

        print!("{}", buffer);
        let _ = stdout().flush();

        let elapsed = now.elapsed();
        if elapsed < frame_duration {
            thread::sleep(frame_duration - elapsed);
        }
    }
}
