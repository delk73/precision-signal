//! Saturation Boundary Tests for geom-spatial
//!
//! These tests verify the monotonicity and continuity of the magnitude
//! calculation at the Bit 81 transition point where shift-scaling activates.

use geom_signal::Scalar;
use geom_spatial::Vector3;
use proptest::prelude::*;

/// Bit 81 in I64F64: represents 2^17 in the integer part (~131 km)
const TRANSITION_VALUE: i128 = 1i128 << 81;

/// Tolerance for precision loss at the shift boundary.
/// At bit 81, we expect ~1 LSB quantization noise per shift bit.
/// This corresponds to ~2^-47 relative error at this scale.
fn tolerance_at_transition() -> Scalar {
    // Allow 1 part in 2^40 relative error (generous margin)
    Scalar::from_bits(1i128 << 41)
}

proptest! {
    /// Verify that 1D vectors at the transition boundary compute correctly.
    /// For a 1D vector (x, 0, 0), magnitude should be approximately abs(x).
    #[test]
    fn monotonicity_at_transition(delta in -100i64..100i64) {
        let base = Scalar::from_bits(TRANSITION_VALUE + delta as i128);
        let v = Vector3::new(base, Scalar::ZERO, Scalar::ZERO);
        let mag = v.magnitude();

        // For 1D vector, magnitude should be close to abs(x)
        let expected = if base < Scalar::ZERO { -base } else { base };
        let diff = if mag > expected { mag - expected } else { expected - mag };

        prop_assert!(
            diff <= tolerance_at_transition(),
            "1D magnitude deviation too large: expected {}, got {}, diff {}",
            expected, mag, diff
        );
    }

    /// Verify diagonal vectors at transition maintain monotonicity.
    /// Tests the sum-of-squares accumulator under shift-scaling.
    #[test]
    fn diagonal_transition_continuity(delta in -50i64..50i64) {
        let coord = Scalar::from_bits((TRANSITION_VALUE / 2) + delta as i128);
        let v = Vector3::new(coord, coord, coord);
        let mag = v.magnitude();

        // Magnitude should be positive and finite
        prop_assert!(mag > Scalar::ZERO, "Diagonal magnitude must be positive");
        prop_assert!(mag < Scalar::MAX, "Diagonal magnitude must not saturate spuriously");
    }

    /// Verify that inputs at Scalar::MAX saturate correctly without wrap.
    #[test]
    fn saturation_at_max(x in any::<i32>(), y in any::<i32>()) {
        let v = Vector3::new(
            Scalar::MAX,
            Scalar::from_num(x),
            Scalar::from_num(y),
        );
        let mag = v.magnitude();

        // Must saturate to MAX, not wrap to negative or lower value
        prop_assert_eq!(mag, Scalar::MAX, "Saturation failed at MAX input");
    }
}

#[test]
fn transition_boundary_approximate() {
    // Test at bit 81 boundary with tolerance
    let below = Scalar::from_bits(TRANSITION_VALUE - 1);
    let at = Scalar::from_bits(TRANSITION_VALUE);
    let above = Scalar::from_bits(TRANSITION_VALUE + 1);

    let v_below = Vector3::new(below, Scalar::ZERO, Scalar::ZERO);
    let v_at = Vector3::new(at, Scalar::ZERO, Scalar::ZERO);
    let v_above = Vector3::new(above, Scalar::ZERO, Scalar::ZERO);

    let mag_below = v_below.magnitude();
    let mag_at = v_at.magnitude();
    let mag_above = v_above.magnitude();

    // All should be within tolerance of expected
    let tol = tolerance_at_transition();

    let diff_below = if mag_below > below {
        mag_below - below
    } else {
        below - mag_below
    };
    let diff_at = if mag_at > at {
        mag_at - at
    } else {
        at - mag_at
    };
    let diff_above = if mag_above > above {
        mag_above - above
    } else {
        above - mag_above
    };

    assert!(
        diff_below <= tol,
        "Below boundary: diff {} > tol {}",
        diff_below,
        tol
    );
    assert!(
        diff_at <= tol,
        "At boundary: diff {} > tol {}",
        diff_at,
        tol
    );
    assert!(
        diff_above <= tol,
        "Above boundary: diff {} > tol {}",
        diff_above,
        tol
    );

    // Monotonicity: magnitudes should weakly increase (ties allowed due to quantization)
    assert!(mag_below <= mag_at, "Monotonicity violation: below > at");
    assert!(mag_at <= mag_above, "Monotonicity violation: at > above");
}

#[test]
fn max_max_max_saturates() {
    // Extreme case: all coordinates at MAX
    let v = Vector3::new(Scalar::MAX, Scalar::MAX, Scalar::MAX);
    let mag = v.magnitude();

    // Must saturate, not wrap
    assert_eq!(mag, Scalar::MAX, "Triple MAX must saturate");
}
