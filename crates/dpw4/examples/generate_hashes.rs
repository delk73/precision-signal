use dpw4::{tick_dpw4_raw, Dpw4State};
use sha2::{Digest, Sha256};

fn main() {
    // We generate 2 hashes for the Golden Lock:
    // 1. Saw-440Hz-10k
    // 2. Pulse-10%-440Hz-10k
    // These protect the RAW KERNEL and relational bit-alignment.

    println!("Generating Reference Hashes...");

    // 1. Saw-440Hz-10k
    let mut state = Dpw4State::new();
    state.reset();
    let mut phase: u64 = 0;
    let phase_inc = (libm::floor((440.0 / 44100.0) * (u64::MAX as f64) + 0.5)) as u64;

    let mut hasher = Sha256::new();
    for _ in 0..10000 {
        let s_q31 = ((phase >> 32) as i64).wrapping_sub(1i64 << 31);
        let raw = tick_dpw4_raw(&mut state, s_q31);
        hasher.update(raw.to_le_bytes());
        phase = phase.wrapping_add(phase_inc);
    }
    let hash = format!("{:x}", hasher.finalize());
    println!("Saw-440Hz-10k: {}", hash);

    // 2. Pulse-10%-440Hz-10k
    let mut state_a = Dpw4State::new();
    let mut state_b = Dpw4State::new();
    state_a.reset();
    state_b.reset();
    phase = 0;
    let duty_q32 = (libm::floor(0.1 * 4294967296.0 + 0.5)) as u32;
    let duty_ph = (duty_q32 as u64) << 32;
    let mut hasher_p = Sha256::new();
    for _ in 0..10000 {
        let s_a = ((phase >> 32) as i64).wrapping_sub(1i64 << 31);
        let s_b = ((phase.wrapping_sub(duty_ph) >> 32) as i64).wrapping_sub(1i64 << 31);
        let raw_a = tick_dpw4_raw(&mut state_a, s_a);
        let raw_b = tick_dpw4_raw(&mut state_b, s_b);
        let diff = raw_a.wrapping_sub(raw_b);
        hasher_p.update(diff.to_le_bytes());
        phase = phase.wrapping_add(phase_inc);
    }
    println!("Pulse-10%-440Hz-10k: {:x}", hasher_p.finalize());
}
