use crate::Scalar;

// 128-bit Constants (I64F64 bit-patterns)
pub const PI: Scalar = Scalar::from_bits(0x0000000000000003_243f6a8885a308d3i128);
pub const HALF_PI: Scalar = Scalar::from_bits(0x0000000000000001_921fb54442d1846ai128);
pub const TWO_PI: Scalar = Scalar::from_bits(0x0000000000000006_487ed5110b4611a6i128);
pub const K_GAIN: Scalar = Scalar::from_bits(0x0000000000000000_9b74eda8435e5b61i128);

// Reference ATAN Table: atan(2^-i) for i=0..31 (High Precision)
const ATAN_TABLE: [Scalar; 32] = [
    Scalar::from_bits(0x0000000000000000_c90fdaa22168c234i128), // atan(1)
    Scalar::from_bits(0x0000000000000000_76b19c159bf53371i128), // atan(0.5)
    Scalar::from_bits(0x0000000000000000_3eb6ebf232aa7703i128), // atan(0.25)
    Scalar::from_bits(0x0000000000000000_1fd5ba9aac2e600fi128),
    Scalar::from_bits(0x0000000000000000_0ffaadea24a4e441i128),
    Scalar::from_bits(0x0000000000000000_07ff55bb708461abi128),
    Scalar::from_bits(0x0000000000000000_03ffeaad6fbe7173i128),
    Scalar::from_bits(0x0000000000000000_01fff555bb72cf9ai128),
    Scalar::from_bits(0x0000000000000000_00fffebaaaaab551i128),
    Scalar::from_bits(0x0000000000000000_007fff555555bb73i128),
    Scalar::from_bits(0x0000000000000000_003fffff55555556i128),
    Scalar::from_bits(0x0000000000000000_001fffffeaaaaaaai128),
    Scalar::from_bits(0x0000000000000000_000ffffffff55555i128),
    Scalar::from_bits(0x0000000000000000_0007fffffffebaaai128),
    Scalar::from_bits(0x0000000000000000_0003fffffffffaabi128),
    Scalar::from_bits(0x0000000000000000_0001fffffffffff5i128),
    Scalar::from_bits(0x0000000000000000_0000fffffffffffei128),
    Scalar::from_bits(0x0000000000000000_0000800000000000i128), // atan(2^-17) approx
    Scalar::from_bits(0x0000000000000000_0000400000000000i128),
    Scalar::from_bits(0x0000000000000000_0000200000000000i128),
    Scalar::from_bits(0x0000000000000000_0000100000000000i128),
    Scalar::from_bits(0x0000000000000000_0000080000000000i128),
    Scalar::from_bits(0x0000000000000000_0000040000000000i128),
    Scalar::from_bits(0x0000000000000000_0000020000000000i128),
    Scalar::from_bits(0x0000000000000000_0000010000000000i128),
    Scalar::from_bits(0x0000000000000000_0000008000000000i128),
    Scalar::from_bits(0x0000000000000000_0000004000000000i128),
    Scalar::from_bits(0x0000000000000000_0000002000000000i128),
    Scalar::from_bits(0x0000000000000000_0000001000000000i128),
    Scalar::from_bits(0x0000000000000000_0000000800000000i128),
    Scalar::from_bits(0x0000000000000000_0000000400000000i128),
    Scalar::from_bits(0x0000000000000000_0000000200000000i128),
];

pub fn sin_cos(theta: Scalar) -> (Scalar, Scalar) {
    // 1. Range Reduction (Safe for massive theta)
    let mut z = theta % TWO_PI;
    if z > PI {
        z -= TWO_PI;
    } else if z < -PI {
        z += TWO_PI;
    }
    sin_cos_kernel(z)
}

/// Optimized sin_cos for pre-normalized inputs in range [0, 2PI].
/// Skips the expensive modulo operation for trusted paths (like Oscillators).
pub fn sin_cos_fast(theta_normalized: Scalar) -> (Scalar, Scalar) {
    let mut z = theta_normalized;
    // Cheaper range adjustment for [0, 2PI]
    if z > PI {
        z -= TWO_PI;
    } else if z < -PI {
        z += TWO_PI;
    }
    sin_cos_kernel(z)
}

/// Core CORDIC kernel logic (Internal)
#[inline(always)]
fn sin_cos_kernel(mut z: Scalar) -> (Scalar, Scalar) {
    let mut sign = 1i128;
    if z > HALF_PI {
        z -= PI;
        sign = -1;
    } else if z < -HALF_PI {
        z += PI;
        sign = -1;
    }

    // 2. CORDIC Rotation
    let mut x = K_GAIN;
    let mut y = Scalar::ZERO;

    for i in 0..64 {
        let tx = x;
        let ty = y;
        let atan_i = if i < 32 {
            ATAN_TABLE[i as usize]
        } else {
            Scalar::from_bits(1i128 << (64 - i))
        };

        if z >= 0 {
            x -= ty >> i;
            y += tx >> i;
            z -= atan_i;
        } else {
            x += ty >> i;
            y -= tx >> i;
            z += atan_i;
        }
    }
    (y * sign, x * sign)
}

pub fn sqrt(value: Scalar) -> Scalar {
    if value <= 0 {
        return Scalar::ZERO;
    }
    let mut op = value.to_bits() as u128;
    let mut res = 0u128;
    let mut one = 1u128 << 126;
    while one > op {
        one >>= 2;
    }
    while one != 0 {
        if op >= res + one {
            op -= res + one;
            res = (res >> 1) + one;
        } else {
            res >>= 1;
        }
        one >>= 2;
    }
    Scalar::from_bits((res << 32) as i128)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test sin_cos accuracy against ground truth
    #[test]
    fn test_sin_cos_accuracy() {
        let test_points: [f64; 6] = [0.0, 0.1, 0.5, 1.0, core::f64::consts::FRAC_PI_2, -0.5];
        let tolerance = 1e-6; // Reasonable for fixed-point CORDIC baseline

        for &p in test_points.iter() {
            let theta = Scalar::from_num(p);
            let (s, c) = sin_cos(theta);
            let s_truth = p.sin();
            let c_truth = p.cos();

            let s_val = s.to_num::<f64>();
            let c_val = c.to_num::<f64>();

            let s_diff = (s_val - s_truth).abs();
            let c_diff = (c_val - c_truth).abs();

            assert!(
                s_diff < tolerance,
                "Sine failed at {}: approx={}, truth={}, diff={}",
                p,
                s_val,
                s_truth,
                s_diff
            );
            assert!(
                c_diff < tolerance,
                "Cosine failed at {}: approx={}, truth={}, diff={}",
                p,
                c_val,
                c_truth,
                c_diff
            );
        }
    }

    /// Test Pythagorean identity: sin^2 + cos^2 = 1
    #[test]
    fn test_pythagorean_identity() {
        let test_points: [f64; 5] = [0.1, 0.5, 1.0, 1.5, 2.0];
        for &p in test_points.iter() {
            let theta = Scalar::from_num(p);
            let (s, c) = sin_cos(theta);
            let identity = (s * s + c * c).to_num::<f64>();
            let diff = (identity - 1.0).abs();
            assert!(
                diff < 1e-8,
                "Identity failed at {}: result={}, diff={}",
                p,
                identity,
                diff
            );
        }
    }

    /// Test sqrt accuracy
    #[test]
    fn test_sqrt_accuracy() {
        let test_points: [f64; 5] = [0.001, 1.0, 2.0, 100.0, 1000000.0];
        for &p in test_points.iter() {
            let val = Scalar::from_num(p);
            let res = sqrt(val).to_num::<f64>();
            let truth = p.sqrt();
            let diff = (res - truth).abs();
            assert!(
                diff < 1e-8,
                "Sqrt failed at {}: approx={}, truth={}, diff={}",
                p,
                res,
                truth,
                diff
            );
        }
    }

    /// Determinism check for sin_cos
    #[test]
    fn test_sin_cos_determinism() {
        let theta = Scalar::from_num(1.0);
        let (s, c) = sin_cos(theta);

        // Golden bits for sin(1.0) and cos(1.0)
        let s_bits: i128 = 15522404621367621153;
        let c_bits: i128 = 9966811009118392844;

        assert_eq!(s.to_bits(), s_bits, "Sin bit mismatch");
        assert_eq!(c.to_bits(), c_bits, "Cos bit mismatch");
    }
}
