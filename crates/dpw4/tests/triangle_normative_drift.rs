use dpw4::math;
use dpw4::{DpwGain, OscState, Scalar};
use dpw4::{SignalShape, TriangleDPW4};

// TEST-ONLY I256/U256; not shared with crate::i256; used for sums/bounds only.
#[derive(Clone, Copy)]
struct U256 {
    hi: u128,
    lo: u128,
}

// TEST-ONLY I256/U256; not shared with crate::i256; used for sums/bounds only.
#[derive(Clone, Copy)]
struct I256 {
    hi: i128,
    lo: u128,
}

impl I256 {
    fn zero() -> Self {
        Self { hi: 0, lo: 0 }
    }

    fn add_i128(self, v: i128) -> Self {
        let v_hi = (v >> 127) as i128;
        let (lo, overflow) = self.lo.overflowing_add(v as u128);
        let mut hi = self.hi.wrapping_add(v_hi);
        if overflow {
            hi = hi.wrapping_add(1);
        }
        Self { hi, lo }
    }

    fn negate(self) -> Self {
        let (lo, overflow) = (!self.lo).overflowing_add(1);
        let mut hi = !self.hi;
        if overflow {
            hi = hi.wrapping_add(1);
        }
        Self { hi, lo }
    }

    fn unsigned_abs(self) -> U256 {
        if self.hi < 0 {
            let neg = self.negate();
            U256 {
                hi: neg.hi as u128,
                lo: neg.lo,
            }
        } else {
            U256 {
                hi: self.hi as u128,
                lo: self.lo,
            }
        }
    }
}

fn sub(a: I256, b: I256) -> I256 {
    let nb = b.negate();
    let (lo, overflow) = a.lo.overflowing_add(nb.lo);
    let mut hi = a.hi.wrapping_add(nb.hi);
    if overflow {
        hi = hi.wrapping_add(1);
    }
    I256 { hi, lo }
}

impl U256 {
    fn le(&self, other: &U256) -> bool {
        if self.hi < other.hi {
            true
        } else if self.hi == other.hi {
            self.lo <= other.lo
        } else {
            false
        }
    }
}

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
#[ignore = "evidence: 10M ticks; run manually with --ignored"]
fn test_triangle_normative_drift() {
    let mut state = OscState::new();
    let gain = DpwGain::new(1 << 63, 0, 0, 0); // Unity gain

    // Non-dividing frequency (440 Hz at 44100 Hz SR)
    let dphi_u32: u32 = 42845688;
    let scale_2_32: Scalar = Scalar::from_bits(1_i128 << 96);
    let phase_inc = (Scalar::from_num(dphi_u32) / scale_2_32) * math::TWO_PI;

    let num_ticks: u32 = 10_000_000;
    let window_size: u32 = 100_000;

    let mut current_phase = Scalar::ZERO;

    let mut max_abs_z: u128 = 0;

    let mut sum_z = I256::zero();
    let mut sum_out: i128 = 0;
    let mut max_abs_out: u32 = 0;
    let mut sum_window = I256::zero();

    let mut sum_first_window = I256::zero();
    let mut sum_last_window = I256::zero();

    let mut window_k: u32 = 0;
    let mut windows_seen: u32 = 0;

    for _ in 0..num_ticks {
        let out = TriangleDPW4::tick(&mut state, current_phase, &gain);

        let z = state.tri.z;
        let abs_z: u128 = z.unsigned_abs();

        if abs_z > max_abs_z {
            max_abs_z = abs_z;
        }

        let abs_out = out.unsigned_abs();
        if abs_out > max_abs_out {
            max_abs_out = abs_out;
        }
        sum_out = sum_out.wrapping_add(out as i128);

        let z_i128 = z as i128;

        sum_z = sum_z.add_i128(z_i128);
        sum_window = sum_window.add_i128(z_i128);

        window_k += 1;

        if window_k == window_size {
            if windows_seen == 0 {
                sum_first_window = sum_window;
            }
            sum_last_window = sum_window;
            sum_window = I256::zero();
            window_k = 0;
            windows_seen += 1;
        }

        current_phase += phase_inc;
        if current_phase >= math::TWO_PI {
            current_phase -= math::TWO_PI;
        } else if current_phase < Scalar::ZERO {
            current_phase += math::TWO_PI;
        }
    }

    let threshold = max_abs_z / 100;
    let bound_global = mul_u128_u32_to_u256(threshold, num_ticks);
    let abs_sum = sum_z.unsigned_abs();

    println!("Normative Drift Test Results:");
    println!("N: {}", num_ticks);
    println!("Window size: {}", window_size);
    println!("Max |z|: {}", max_abs_z);
    println!("Threshold (1% max |z|): {}", threshold);

    println!(
        "Global sum_z.unsigned_abs(): hi={}, lo={}",
        abs_sum.hi, abs_sum.lo
    );
    println!(
        "Global boundary: hi={}, lo={}",
        bound_global.hi, bound_global.lo
    );

    assert!(
        abs_sum.le(&bound_global),
        "Global |sum_z| exceeded 1% of max |z| * N"
    );

    assert!(windows_seen >= 2, "Expected at least two completed windows");
    let diff = sub(sum_last_window, sum_first_window);
    let abs_diff = diff.unsigned_abs();
    let bound_window = mul_u128_u32_to_u256(threshold, window_size);

    println!(
        "Window diff.unsigned_abs(): hi={}, lo={}",
        abs_diff.hi, abs_diff.lo
    );
    println!(
        "Window boundary: hi={}, lo={}",
        bound_window.hi, bound_window.lo
    );

    assert!(
        abs_diff.le(&bound_window),
        "Window drift proxy exceeded 1% of max |z| * window_size"
    );

    let threshold_out = (max_abs_out as u128) / 100;
    let bound_out = threshold_out * num_ticks as u128;
    let abs_sum_out = if sum_out < 0 {
        sum_out.wrapping_neg() as u128
    } else {
        sum_out as u128
    };

    println!("Output max_abs_out: {}", max_abs_out);
    println!("Output threshold_out: {}", threshold_out);
    println!("Output abs_sum_out: {}", abs_sum_out);
    println!("Output bound_out: {}", bound_out);

    assert!(
        abs_sum_out <= bound_out,
        "Output DC exceeded 1% envelope bound"
    );
}
