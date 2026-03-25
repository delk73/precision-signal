use dpw4::math;
use dpw4::{DpwGain, OscState, Scalar};
use dpw4::{SignalShape, TriangleDPW4};
use std::time::Instant;

// TEST-ONLY I256/U256; not shared with crate::i256; used for sums/bounds only.
#[derive(Clone, Copy, Default)]
struct I256 {
    hi: i128,
    lo: u128,
}

// TEST-ONLY I256/U256; not shared with crate::i256; used for sums/bounds only.
#[derive(Clone, Copy, Default)]
struct U256 {
    hi: u128,
    lo: u128,
}

impl I256 {
    #[inline(always)]
    fn add_i128(&mut self, x: i128) {
        let x_lo = x as u128;
        let x_hi = x >> 127; // sign extension

        let (lo, carry) = self.lo.overflowing_add(x_lo);
        self.lo = lo;
        self.hi = self.hi.wrapping_add(x_hi).wrapping_add(carry as i128);
    }

    #[inline(always)]
    fn unsigned_abs(&self) -> U256 {
        if self.hi < 0 {
            let (inv_lo, carry) = (!self.lo).overflowing_add(1);
            let inv_hi = (!self.hi).wrapping_add(carry as i128);
            U256 {
                hi: inv_hi as u128,
                lo: inv_lo,
            }
        } else {
            U256 {
                hi: self.hi as u128,
                lo: self.lo,
            }
        }
    }
}

impl U256 {
    #[inline(always)]
    fn check_le(&self, rhs: &U256) -> bool {
        if self.hi < rhs.hi {
            true
        } else if self.hi == rhs.hi {
            self.lo <= rhs.lo
        } else {
            false
        }
    }
}

// Multiplies u128 by u32 and returns a U256 value.
#[inline(always)]
fn mul_u128_u32_to_u256(x: u128, n: u32) -> U256 {
    let n = n as u64;

    let x0 = (x & 0xFFFF_FFFF_FFFF_FFFF) as u64;
    let x1 = (x >> 64) as u64;

    let mut lo0 = x0 as u128 * n as u128;
    let mut lo1 = x1 as u128 * n as u128;

    let carry = lo0 >> 64;
    lo0 &= 0xFFFF_FFFF_FFFF_FFFF;

    lo1 += carry;

    let lo = lo0 | ((lo1 & 0xFFFF_FFFF_FFFF_FFFF) << 64);
    let hi = lo1 >> 64;

    U256 { hi: hi as u128, lo }
}

#[test]
#[ignore = "runtime probe: manual benchmark only"]
fn test_d03_runtime_probe() {
    let dphi_u32: u32 = 42845688;
    let scale_2_32: Scalar = Scalar::from_bits(1_i128 << 96);
    let phase_inc = (Scalar::from_num(dphi_u32) / scale_2_32) * math::TWO_PI;
    let gain = DpwGain::new(1 << 63, 0, 0, 0);

    let num_ticks = 10_000_000;
    const MEAN_SHIFT: u32 = 32;

    // --- Variant 1: Shifted (baseline) ---
    let mut state1 = OscState::new();
    let mut phase1 = Scalar::ZERO;
    let mut sum_z_shifted: i128 = 0;

    let mut overflowed = false;

    let t0 = Instant::now();
    for _ in 0..num_ticks {
        TriangleDPW4::tick(&mut state1, phase1, &gain);
        let (new_sum, of) = sum_z_shifted.overflowing_add((state1.tri.z as i128) >> MEAN_SHIFT);
        overflowed |= of;
        sum_z_shifted = new_sum;
        phase1 += phase_inc;
        clip_phase(&mut phase1);
    }
    let elapsed1 = t0.elapsed();

    // --- Variant 2: Exact sum only (I256) ---
    let mut state2 = OscState::new();
    let mut phase2 = Scalar::ZERO;
    let mut sum_exact = I256::default();

    let t1 = Instant::now();
    for _ in 0..num_ticks {
        TriangleDPW4::tick(&mut state2, phase2, &gain);
        sum_exact.add_i128(state2.tri.z as i128);
        phase2 += phase_inc;
        clip_phase(&mut phase2);
    }
    let elapsed2 = t1.elapsed();

    // --- Variant 3: Exact sum + end check (no division) ---
    let mut state3 = OscState::new();
    let mut phase3 = Scalar::ZERO;
    let mut sum_exact_chk = I256::default();
    let mut max_abs_z: u128 = 0;

    let t2 = Instant::now();
    for _ in 0..num_ticks {
        TriangleDPW4::tick(&mut state3, phase3, &gain);
        let z = state3.tri.z;
        let abs_z = z.unsigned_abs();
        if abs_z > max_abs_z {
            max_abs_z = abs_z;
        }

        sum_exact_chk.add_i128(z as i128);

        phase3 += phase_inc;
        clip_phase(&mut phase3);
    }

    let threshold = max_abs_z / 100;
    let bound = mul_u128_u32_to_u256(threshold, num_ticks);
    let check_ok = sum_exact_chk.unsigned_abs().check_le(&bound);

    let elapsed3 = t2.elapsed();

    // Prevent optimizations
    std::hint::black_box(sum_z_shifted);
    std::hint::black_box(sum_exact.hi);
    std::hint::black_box(sum_exact.lo);
    std::hint::black_box(sum_exact_chk.hi);
    std::hint::black_box(sum_exact_chk.lo);
    std::hint::black_box(bound.hi);
    std::hint::black_box(bound.lo);
    std::hint::black_box(check_ok);

    // Format helpers
    let format_ns = |dur: std::time::Duration| {
        let n = dur.as_nanos();
        let w = n / num_ticks as u128;
        let f = ((n % num_ticks as u128) * 100) / num_ticks as u128;
        format!("{}.{:02}", w, f)
    };

    let p_overhead = |base: std::time::Duration, target: std::time::Duration| {
        let n_base = base.as_nanos();
        let n_target = target.as_nanos();
        if n_target >= n_base {
            let diff = n_target - n_base;
            let w = (diff * 100) / n_base;
            let f = ((diff * 10000) / n_base) % 100;
            format!("{}.{:02}", w, f)
        } else {
            let diff = n_base - n_target;
            let w = (diff * 100) / n_base;
            let f = ((diff * 10000) / n_base) % 100;
            format!("-{}.{:02}", w, f)
        }
    };

    println!(
        "shifted: {} ms ({} ns/tick)",
        elapsed1.as_millis(),
        format_ns(elapsed1)
    );
    println!(
        "i256_sum: {} ms ({} ns/tick)",
        elapsed2.as_millis(),
        format_ns(elapsed2)
    );
    println!(
        "i256_sum+check: {} ms ({} ns/tick)",
        elapsed3.as_millis(),
        format_ns(elapsed3)
    );
    println!("i256 overhead: {}%", p_overhead(elapsed1, elapsed2));
    println!("i256+check overhead: {}%", p_overhead(elapsed1, elapsed3));
    println!("shifted overflowed: {}", overflowed);
}

#[inline(always)]
fn clip_phase(phase: &mut Scalar) {
    if *phase >= math::TWO_PI {
        *phase -= math::TWO_PI;
    } else if *phase < Scalar::ZERO {
        *phase += math::TWO_PI;
    }
}
