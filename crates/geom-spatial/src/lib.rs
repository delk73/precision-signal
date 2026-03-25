#![forbid(unsafe_code)]
use geom_signal::{sqrt, Scalar};
use std::ops::Sub;

/// A 3D spatial vector with 128-bit fixed-point coordinates.
///
/// # Range and Precision
///
/// - **Safe Range**: Up to $\approx 2.8 \times 10^{14}$ meters (astronomical-scale, with formal saturation guarantees)
/// - **Precision**: $2^{-64}$ meters ($\approx 5.4 \times 10^{-20}$ m)
/// - **Overflow Behavior**: Saturates to `Scalar::MAX` instead of wrapping
///
/// # Implementation Details
///
/// The `magnitude()` calculation uses a **32-bit shift-scaling algorithm** to prevent
/// overflow when computing $\sqrt{x^2 + y^2 + z^2}$ at extreme scales:
///
/// 1. Detect the maximum coordinate component
/// 2. Calculate required right-shift (0-32 bits)
/// 3. Apply uniform shift to all components
/// 4. Compute magnitude in safe range
/// 5. Shift result back up
///
/// This extends the operational range from $\approx 10^9$ m (naive) to $\approx 2.8 \times 10^{14}$ m
/// while maintaining bit-exact determinism.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector3 {
    pub x: Scalar,
    pub y: Scalar,
    pub z: Scalar,
}

impl Vector3 {
    /// Create a Vector3 from Scalar components (bit-exact, deterministic).
    ///
    /// This is the standard constructor for internal use where coordinates are
    /// already in `Scalar` representation. It has zero conversion overhead.
    ///
    /// # Example
    /// ```
    /// use geom_spatial::Vector3;
    /// use geom_signal::Scalar;
    ///
    /// let v = Vector3::new(
    ///     Scalar::from_num(3),
    ///     Scalar::from_num(4),
    ///     Scalar::from_num(0)
    /// );
    /// ```
    pub fn new(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Self { x, y, z }
    }

    /// Create a Vector3 from `Scalar` values (bit-exact, deterministic).
    #[inline]
    pub fn from_scalar(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Self { x, y, z }
    }

    /// Create a Vector3 from f64 values (non-deterministic ingestion).
    ///
    /// # Determinism Warning
    ///
    /// Floating-point conversions are platform-dependent and may vary across
    /// architectures. For bit-exact reproducibility, use `new()` with pre-converted
    /// `Scalar` values.
    ///
    /// # Example
    /// ```
    /// use geom_spatial::Vector3;
    ///
    /// let v = Vector3::from_f64(3.0, 4.0, 0.0);
    /// ```
    #[cfg(feature = "float-ingest")]
    pub fn from_f64(x: f64, y: f64, z: f64) -> Self {
        Self {
            x: Scalar::from_num(x),
            y: Scalar::from_num(y),
            z: Scalar::from_num(z),
        }
    }

    /// Compute the magnitude with overflow detection (returns `None` on overflow).
    ///
    /// This is the error-propagating variant for contexts where overflow must be
    /// explicitly handled.
    ///
    /// # Returns
    /// - `Some(magnitude)` if calculation succeeds
    /// - `None` if intermediate values would overflow (coordinates $> \approx 2.8 \times 10^{14}$ m)
    pub fn magnitude_checked(&self) -> Option<Scalar> {
        // Find maximum absolute coordinate to determine scaling
        let abs_x = self.x.checked_abs()?;
        let abs_y = self.y.checked_abs()?;
        let abs_z = self.z.checked_abs()?;

        let max_coord = abs_x.max(abs_y).max(abs_z);

        // Calculate required shift to prevent overflow
        // For I64F64, we want max_coord^2 to fit, so max_coord should be < 2^32
        let shift = calculate_shift(max_coord);

        // If shift > 32, we're beyond our extended range
        if shift > 32 {
            return None;
        }

        // Apply shift-scaling
        let x_scaled = self.x >> shift;
        let y_scaled = self.y >> shift;
        let z_scaled = self.z >> shift;

        // Compute in safe range using checked operations
        let x2 = x_scaled.checked_mul(x_scaled)?;
        let y2 = y_scaled.checked_mul(y_scaled)?;
        let z2 = z_scaled.checked_mul(z_scaled)?;

        let sum_sq = x2.checked_add(y2)?.checked_add(z2)?;
        let mag_scaled = sqrt(sum_sq);

        // Scale back up: sqrt(x^2) where x was shifted by N means we shift result by N
        // Because sqrt((x >> N)^2) = (x >> N), and we want x, so shift left by N
        // But wait, (x >> N)^2 >> (2N) in value, so sqrt brings it back to >> N
        // Therefore mag_scaled is already at the >> N scale, we shift by N to get back
        Some(mag_scaled.checked_shl(shift).unwrap_or(Scalar::MAX))
    }

    /// Compute the magnitude (Euclidean norm) of the vector.
    ///
    /// Uses the verified 128-bit CORDIC sqrt with automatic shift-scaling to
    /// support coordinates up to $\approx 2.8 \times 10^{14}$ meters.
    ///
    /// # Overflow Behavior
    ///
    /// If coordinates exceed the extended safe range ($> \approx 2.8 \times 10^{14}$ m), the result
    /// **saturates to `Scalar::MAX`** instead of wrapping. This ensures graceful
    /// degradation rather than catastrophic failure.
    ///
    /// # Example
    /// ```
    /// use geom_spatial::Vector3;
    /// use geom_signal::Scalar;
    ///
    /// // Pythagorean triple: 3-4-5
    /// let v = Vector3::from_scalar(
    ///     Scalar::from_num(3),
    ///     Scalar::from_num(4),
    ///     Scalar::from_num(0)
    /// );
    /// assert_eq!(v.magnitude(), Scalar::from_num(5));
    /// ```
    pub fn magnitude(&self) -> Scalar {
        self.magnitude_checked().unwrap_or(Scalar::MAX)
    }

    /// Compute the Euclidean distance between two vectors.
    ///
    /// # Example
    /// ```
    /// use geom_spatial::Vector3;
    /// use geom_signal::Scalar;
    ///
    /// let source = Vector3::from_scalar(
    ///     Scalar::from_num(3),
    ///     Scalar::from_num(4),
    ///     Scalar::from_num(0)
    /// );
    /// let observer = Vector3::from_scalar(
    ///     Scalar::from_num(0),
    ///     Scalar::from_num(0),
    ///     Scalar::from_num(0)
    /// );
    ///
    /// // Distance should be 5 (Pythagorean theorem)
    /// let dist = source.distance(&observer);
    /// assert_eq!(dist, Scalar::from_num(5));
    /// ```
    pub fn distance(&self, other: &Vector3) -> Scalar {
        (*self - *other).magnitude()
    }
}

/// Calculate the number of bits to right-shift coordinates to prevent overflow.
///
/// # Algorithm
///
/// For I64F64 (64.64 fixed point), squaring a value requires twice the integer bits.
/// To prevent overflow, we need `max_coord^2 < 2^64`, which means `max_coord < 2^32`.
///
/// Additionally, we need to ensure that after sqrt, shifting back left doesn't overflow.
/// We use a conservative threshold of 2^48 (bit 112) to ensure safety.
///
/// # Returns
/// Number of bits to shift (0-32 for normal range, >32 indicates overflow)
/// Threshold for spatial coordinate scaling (bits).
///
/// When the most significant bit of a coordinate exceeds bit 80:
/// - **Trigger Value**: $2^{17}$ in the integer part of I64F64 (~131 km)
/// - **Shift Activation**: Coordinates are right-shifted to prevent overflow in $x^2 + y^2 + z^2$
/// - **Precision Trade-off**: 1 LSB quantization noise introduced per shift bit
///
/// For AU-scale operations (~$10^{11}$ m), shift will be ~47 bits.
const SPATIAL_SCALE_THRESHOLD: u32 = 80;

#[inline]
fn calculate_shift(max_coord: Scalar) -> u32 {
    let bits = max_coord.to_bits();
    let abs_bits = if bits < 0 {
        (-bits) as u128
    } else {
        bits as u128
    };

    if abs_bits == 0 {
        return 0;
    }

    // Find the position of the most significant bit
    // For I64F64, the integer part is in bits [64..127]
    let msb_pos = 127 - abs_bits.leading_zeros();

    // We want to keep the value safe for squaring AND for shift-back after sqrt
    // Use a more conservative threshold: if msb > SPATIAL_SCALE_THRESHOLD, start shifting
    // This gives us range up to 2^48 (281 trillion) with safety margin
    msb_pos.saturating_sub(SPATIAL_SCALE_THRESHOLD)
}

impl Sub for Vector3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pythagorean_tuning() {
        // Place a "Source" at (3, 4, 0)
        let source = Vector3::from_scalar(
            Scalar::from_num(3),
            Scalar::from_num(4),
            Scalar::from_num(0),
        );
        // Place an "Observer" at (0, 0, 0)
        let observer = Vector3::from_scalar(
            Scalar::from_num(0),
            Scalar::from_num(0),
            Scalar::from_num(0),
        );

        // Assert: source.distance(&observer) is exactly 5
        let dist = source.distance(&observer);
        let expected = Scalar::from_num(5);

        // We use bit-identical comparison as 3^2 + 4^2 = 25 is exact in fixed point
        assert_eq!(
            dist, expected,
            "Pythagorean distance failed: expected {}, got {}",
            expected, dist
        );
    }

    // ===== Boundary Stress Tests: Astronomical-Scale Magnitude =====

    #[test]
    fn test_safe_range() {
        // 10^9 meters: should compute with minimal or no shift
        let large = Scalar::from_num(1_000_000_000);
        let v = Vector3::new(large, large, large);
        let mag = v.magnitude();

        // Expected: sqrt(3) * 10^9 ≈ 1.732050808 * 10^9
        // With shift-scaling, there may be minor precision differences
        let three = Scalar::from_num(3);
        let large_sq = large * large;
        let expected = sqrt(three * large_sq);

        // Allow for minor precision differences due to shift-scaling
        let diff = if mag > expected {
            mag - expected
        } else {
            expected - mag
        };
        let tolerance = Scalar::from_num(0.001); // 1mm tolerance at 1000km scale

        assert!(
            diff < tolerance,
            "Safe range calculation mismatch: expected {}, got {}, diff {}",
            expected,
            mag,
            diff
        );
    }

    #[test]
    fn test_shift_transition() {
        // 2^31 meters: shift becomes active
        // In I64F64, 2^31 is represented as 2^31 << 64 = 0x0000_0000_8000_0000__0000_0000_0000_0000
        let coord = Scalar::from_bits(1i128 << 95); // 2^31 in I64F64
        let v = Vector3::new(coord, Scalar::ZERO, Scalar::ZERO);
        let mag = v.magnitude();

        // For a vector (x, 0, 0), magnitude should equal abs(x)
        assert_eq!(mag, coord, "Shift transition failed");
    }

    #[test]
    fn test_astronomical_stability() {
        // 10^12 meters (~10 light-hours): Astronomical scale test
        // This is the "Astronomical-Scale" domain - well beyond normal usage
        // Old implementation would overflow; new one handles it perfectly
        let large = Scalar::from_num(1_000_000_000_000i64); // 1 trillion meters

        let v = Vector3::new(large, Scalar::ZERO, Scalar::ZERO);

        // The key test: This should NOT panic (the old implementation would panic/overflow)
        let mag = v.magnitude();

        // Should be exact for 1D vector
        assert_eq!(mag, large, "Astronomical magnitude incorrect");

        // Verify magnitude_checked also works
        assert_eq!(
            v.magnitude_checked(),
            Some(large),
            "magnitude_checked failed at astronomical scale"
        );

        // Test a full 3D vector at this scale to ensure shift-scaling works
        let v3d = Vector3::new(large, large, large);
        let mag3d = v3d.magnitude();

        // Expected: sqrt(3 * large^2) = sqrt(3) * large
        // We can't compute this directly due to overflow, so we verify it's reasonable
        let sqrt3 = Scalar::from_num(1.732050807568877);
        let expected_approx = large * sqrt3;

        // Allow some precision loss due to shift-scaling at this extreme range
        let diff = if mag3d > expected_approx {
            mag3d - expected_approx
        } else {
            expected_approx - mag3d
        };
        let tolerance = Scalar::from_num(1_000_000); // 1000km tolerance at trillion-meter scale

        assert!(
            diff < tolerance,
            "3D astronomical magnitude out of range: expected ~{}, got {}",
            expected_approx,
            mag3d
        );
    }

    #[test]
    fn test_hard_saturation() {
        // 10^19 meters: beyond I64F64 capacity
        // This will saturate to MAX
        let v = Vector3::new(Scalar::MAX, Scalar::MAX, Scalar::MAX);
        let mag = v.magnitude();

        // Should saturate, not wrap
        assert_eq!(mag, Scalar::MAX, "Failed to saturate at extreme scale");

        // magnitude_checked should return None
        assert_eq!(
            v.magnitude_checked(),
            None,
            "magnitude_checked should return None at extreme scale"
        );
    }
}
