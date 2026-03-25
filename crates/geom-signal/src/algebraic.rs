//! High-Throughput Algebraic Phase Logic
//!
//! This module provides ILP-optimized alternatives to CORDIC-based trigonometric
//! functions using the Modified Shafer-Fink Approximation.
//!
//! # Motivation
//!
//! CORDIC achieves high precision but suffers from serial iteration latency.
//! Modern ALUs can exploit instruction-level parallelism (ILP) in algebraic
//! expressions, making these approximations faster for high-throughput paths.
//!
//! Based on the implementation approach advocated by **Inigo Quilez** for efficient
//! algebraic trigonometric approximations.
//!
//! # Mathematical Basis
//!
//! Shafer-Fink approximation for arctan:
//! ```text
//! atan(x) ≈ (π² · x) / (4 + √(34 + 4π²x²))
//! ```

use crate::Scalar;

// Import constants from math module
use crate::math::{HALF_PI, PI};

// Scalar constants for algebraic computation
const C_34: Scalar = Scalar::from_bits(34i128 << 64);
const C_4: Scalar = Scalar::from_bits(4i128 << 64);

/// Asymptotic guard threshold: inputs beyond this magnitude short-circuit to ±π/2.
/// This prevents overflow in the squared term while preserving the arctangent
/// asymptotic behavior (atan(±∞) = ±π/2).
const SAFE_LIMIT: Scalar = Scalar::from_bits(1_000_000i128 << 64);

/// Fixed-point sqrt using the integer square root algorithm.
/// Duplicated here to avoid circular dependencies with math module.
#[inline]
fn sqrt_local(value: Scalar) -> Scalar {
    if value <= Scalar::ZERO {
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

/// Computes arctan(x) using the Modified Shafer-Fink approximation.
///
/// # Algorithm
///
/// Uses the algebraic approximation:
/// ```text
/// atan(x) ≈ (π² · x) / (4 + √(34 + 4π²x²))
/// ```
///
/// This form exploits ILP in modern ALUs, avoiding the serial iteration
/// latency of CORDIC while maintaining acceptable precision for DSP paths.
///
/// This specific algebraic optimization for fixed-point/GPU contexts is based
/// on the work and optimizations popularized by **Inigo Quilez**.
///
/// # Asymptotic Guard ("The Trim")
///
/// For |x| > SAFE_LIMIT (1,000,000), the function short-circuits to ±π/2.
/// This prevents overflow in the squared term while correctly modeling
/// the asymptotic behavior of arctan.
///
/// # Arguments
///
/// * `x` - Input value in Scalar (I64F64) format
///
/// # Returns
///
/// Approximation of atan(x) in radians, range [-π/2, π/2]
pub fn atan_shafer(x: Scalar) -> Scalar {
    let abs_x = x.saturating_abs();

    // 1. Asymptotic Trim (Soft-Clip)
    // For large inputs, atan(x) → ±π/2
    if abs_x > SAFE_LIMIT {
        return if x.is_negative() { -HALF_PI } else { HALF_PI };
    }

    // Handle zero explicitly
    if x == Scalar::ZERO {
        return Scalar::ZERO;
    }

    // 2. Algebraic Core
    // atan(x) ≈ (π² · x) / (4 + √(34 + 4π²x²))
    let pi_sq = PI * PI;
    let x_sq = x * x;
    let term_sq = (pi_sq * C_4) * x_sq;
    let denom = C_4 + sqrt_local(C_34 + term_sq);

    (pi_sq * x) / denom
}

/// Computes atan2(y, x) using the Shafer-Fink approximation with quadrant correction.
///
/// This is a drop-in replacement for CORDIC-based atan2, providing the same
/// interface with improved throughput for high-frequency DSP paths.
///
/// # Quadrant Handling
///
/// | Condition      | Result                    |
/// |----------------|---------------------------|
/// | x > 0          | atan(y/x)                 |
/// | x < 0, y >= 0  | atan(y/x) + π             |
/// | x < 0, y < 0   | atan(y/x) - π             |
/// | x = 0, y > 0   | +π/2                      |
/// | x = 0, y < 0   | -π/2                      |
/// | x = 0, y = 0   | 0 (undefined, convention) |
///
/// # Arguments
///
/// * `y` - Y coordinate (numerator)
/// * `x` - X coordinate (denominator)
///
/// # Returns
///
/// Angle in radians, range [-π, π]
pub fn atan2_shafer(y: Scalar, x: Scalar) -> Scalar {
    // Handle special cases on the axes
    if x == Scalar::ZERO {
        if y > Scalar::ZERO {
            return HALF_PI;
        } else if y < Scalar::ZERO {
            return -HALF_PI;
        } else {
            // Both zero: undefined, return 0 by convention
            return Scalar::ZERO;
        }
    }

    // 1. Robust Ratio Calculation
    // We must prevent (y / x) from overflowing the 128-bit Scalar.
    // Since atan(inf) = PI/2, we short-circuit if |y| > |x| * SAFE_LIMIT.
    let abs_y = y.saturating_abs();
    let abs_x = x.saturating_abs();

    let atan_ratio = if abs_y > abs_x.saturating_mul(SAFE_LIMIT) {
        if y.is_negative() == x.is_negative() {
            HALF_PI
        } else {
            -HALF_PI
        }
    } else {
        atan_shafer(y / x)
    };

    // 2. Quadrant correction
    if x > Scalar::ZERO {
        // Quadrants I and IV: no correction needed
        atan_ratio
    } else if y >= Scalar::ZERO {
        // Quadrant II: add π
        atan_ratio + PI
    } else {
        // Quadrant III: subtract π
        atan_ratio - PI
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 1: Verification of "The Trim" (Asymptotic Guard)
    /// Prove that inputs beyond SAFE_LIMIT (1,000,000) hard-clamp to exactly ±π/2.
    #[test]
    fn test_asymptotic_trim() {
        let epsilon = Scalar::from_bits(1);

        // Case: Scalar::MAX -> HALF_PI
        assert_eq!(atan_shafer(Scalar::MAX), HALF_PI);

        // Case: Large Negative (Avoiding MIN overflow which is un-negatable)
        let large_neg = Scalar::from_bits(-(10_000_000i128 << 64));
        assert_eq!(atan_shafer(large_neg), -HALF_PI);

        // Case: SAFE_LIMIT + epsilon -> HALF_PI
        assert_eq!(atan_shafer(SAFE_LIMIT + epsilon), HALF_PI);
        assert_eq!(atan_shafer(-(SAFE_LIMIT + epsilon)), -HALF_PI);

        // Case: SAFE_LIMIT - epsilon -> Calculated value (very close to HALF_PI)
        let near_limit = atan_shafer(SAFE_LIMIT - epsilon);
        let diff = (HALF_PI - near_limit).abs();
        assert!(
            diff < Scalar::from_num(0.00001),
            "Should be extremely close to PI/2 at limit"
        );
    }

    /// Test 2: Quadrant Logic (atan2_shafer)
    /// Verify the 4-quadrant logic matches standard atan2 behavior.
    #[test]
    fn test_quadrants_exhaustive() {
        let one = Scalar::from_bits(1i128 << 64);
        let neg_one = -one;
        let zero = Scalar::ZERO;

        // Q1 (+, +) -> (0, PI/2)
        let q1 = atan2_shafer(one, one);
        assert!(q1 > zero && q1 < HALF_PI);

        // Q2 (-, +) -> (PI/2, PI)
        let q2 = atan2_shafer(one, neg_one);
        assert!(q2 > HALF_PI && q2 <= PI);

        // Q3 (-, -) -> (-PI, -PI/2)
        let q3 = atan2_shafer(neg_one, neg_one);
        assert!(q3 >= -PI && q3 < -HALF_PI);

        // Q4 (+, -) -> (-PI/2, 0)
        let q4 = atan2_shafer(neg_one, one);
        assert!(q4 > -HALF_PI && q4 < zero);

        // Axes
        assert_eq!(atan2_shafer(zero, one), zero); // Positive X
        assert_eq!(atan2_shafer(zero, neg_one), PI); // Negative X
        assert_eq!(atan2_shafer(one, zero), HALF_PI); // Positive Y
        assert_eq!(atan2_shafer(neg_one, zero), -HALF_PI); // Negative Y

        // Origin
        assert_eq!(atan2_shafer(zero, zero), zero); // Return 0 by convention
    }

    /// Test 3: Parity & Symmetry
    #[test]
    fn test_parity_invariant() {
        let points = [0.1, 0.5, 1.0, 2.0, 10.0, 100.0];
        for &p in points.iter() {
            let x = Scalar::from_num(p);
            // Invariant: atan_shafer(-x) == -atan_shafer(x)
            assert_eq!(
                atan_shafer(-x),
                -atan_shafer(x),
                "Parity failed for x={}",
                p
            );

            // Invariant: atan2_shafer(-y, x) == -atan2_shafer(y, x) (for x > 0)
            let y = x;
            let x_pos = Scalar::from_num(1.0);
            assert_eq!(
                atan2_shafer(-y, x_pos),
                -atan2_shafer(y, x_pos),
                "Atan2 parity failed for y={}",
                p
            );
        }
    }

    /// Test 4: Accuracy vs. Ground Truth
    /// Measure error against std::f64::atan.
    #[test]
    fn test_accuracy_vs_std() {
        let test_points: [f64; 8] = [0.0, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 100.0];
        let tolerance = 0.005;

        for &p in test_points.iter() {
            let x = Scalar::from_num(p);
            let approx = atan_shafer(x).to_num::<f64>();
            let truth = p.atan();
            let error = (approx - truth).abs();

            assert!(
                error <= tolerance,
                "Accuracy failed at {}: approx={}, truth={}, error={}",
                p,
                approx,
                truth,
                error
            );
        }
    }

    /// Test 5: Determinism Check
    /// Ensure bit-exact reproduction against a Golden Value.
    #[test]
    fn test_determinism_golden() {
        // Input bit pattern: 0x0000000000000001_0000000000000000 (1.0 in I64F64)
        let x = Scalar::from_bits(1i128 << 64);
        let result = atan_shafer(x);

        // Golden value for atan_shafer(1.0)
        // Calculated locally: (PI^2 * 1) / (4 + sqrt(34 + 4*PI^2))
        // PI^2 ≈ 9.8696044
        // sqrt(34 + 4*9.8696044) = sqrt(34 + 39.4784176) = sqrt(73.4784176) ≈ 8.5719553
        // Result ≈ 9.8696044 / (4 + 8.5719553) = 9.8696044 / 12.5719553 ≈ 0.7850493
        // Actual PI/4 ≈ 0.7853981

        let golden_bits: i128 = 14481603076265524838; // Bit pattern for 0.78504...
        assert_eq!(
            result.to_bits(),
            golden_bits,
            "Determinism failed: bit pattern mismatch"
        );
    }
}
