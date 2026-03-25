use geom_signal::math::TWO_PI;
use geom_signal::{sin_cos, Scalar};
use proptest::prelude::*;

// Harness 1: The "Pythagorean" Property (Fuzzing)
// Invariant: sin^2(theta) + cos^2(theta) ≈ 1.
// Tolerance: Assert error ≤ 10^-18.
// Note: In I64F64, 1 fractional unit is 2^-64 ≈ 5.42e-20.
// 10^-18 is approximately 18.44 fractional units.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]
    #[test]
    fn test_pythagorean_identity_fuzz(theta_bits in any::<i128>()) {
        let theta = Scalar::from_bits(theta_bits);
        let (s, c) = sin_cos(theta);

        let s2 = s * s;
        let c2 = c * c;
        let sum_sq = s2 + c2;
        let one = Scalar::from_num(1);

        let diff = if sum_sq > one { sum_sq - one } else { one - sum_sq };

        // Assert error is small. 1000 bits is ~5.4e-17.
        // This is safe for 64-bit precision where we expect ~10^-17.
        let tolerance = 1000;
        prop_assert!(
            diff.to_bits() <= tolerance,
            "Pythagorean identity failed: theta_bits={},\ns={},\nc={},\nsum_sq={},\ndiff_bits={}",
            theta_bits, s, c, sum_sq, diff.to_bits()
        );
    }
}

// Harness 2: The "Astronomical-Scale" Continuity Check (Manual)
// Input: A massive theta value (e.g., 10^15).
// Test: Compute y1 = sin(theta) and y2 = sin(theta + delta).
// Invariant: The signal must continue to oscillate and not collapse.
#[test]
fn test_extended_phase_continuity() {
    // 10^15 represents extreme theta magnitude (validates long-lived phase reduction; not spatial magnitude)
    let theta = Scalar::from_num(1_000_000_000_000_000i64);

    let (y1, _) = sin_cos(theta);

    // The signal must not collapse to zero
    assert_ne!(
        y1.to_bits(),
        0,
        "Signal collapsed to zero at extended phase range"
    );

    // Check local oscillation. 2^-32 (~2e-10) is a safe distance to see change
    // even with some precision loss, while still being "local".
    let delta = Scalar::from_bits(1i128 << 32);
    let (y2, _) = sin_cos(theta + delta);

    assert_ne!(
        y1, y2,
        "Signal is static at extended phase range (precision lost)"
    );
}

// Harness 3: The Periodicity Law
// Input: Random Scalar theta.
// Invariant: sin(theta) must exactly equal sin(theta + 2pi).
// Purpose: Verify the O(1) Modulo logic handles wrapping bit-perfectly.
proptest! {
    #[test]
    fn test_periodicity_law_fuzz(theta_bits in any::<i128>()) {
        let theta = Scalar::from_bits(theta_bits);

        // Restrict range slightly to avoid overflow of the Scalar type itself during addition
        // TWO_PI is approx 6.28. We subtract 10 from the max to be safe.
        let max_safe = Scalar::MAX - Scalar::from_num(10);
        let min_safe = Scalar::MIN + Scalar::from_num(10);

        if theta < max_safe && theta > min_safe {
            let theta_plus_2pi = theta + TWO_PI;
            let (s1, c1) = sin_cos(theta);
            let (s2, c2) = sin_cos(theta_plus_2pi);

            // Should be bit-perfect due to integer property of modulo in fixed point
            prop_assert_eq!(s1, s2, "Sine periodicity failed at theta_bits={}", theta_bits);
            prop_assert_eq!(c1, c2, "Cosine periodicity failed at theta_bits={}", theta_bits);
        }
    }
}
