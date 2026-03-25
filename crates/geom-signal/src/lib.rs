#![no_std]
#![forbid(unsafe_code)]

pub mod algebraic;
pub mod math;

// Kani Verification Module
#[cfg(any(feature = "verification", kani))]
pub mod verification;

pub use algebraic::{atan2_shafer, atan_shafer};
pub use math::{sin_cos, sin_cos_fast, sqrt};
pub type Scalar = fixed::types::I64F64;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_large_theta_normalized() {
        // Scalar::from_num(1_000_000_000_000)
        let theta = Scalar::from_bits(1_000_000_000_000i128 << 64);
        let (s, c) = sin_cos(theta);

        // Results should be normalized between -1 and 1
        assert!((-1..=1).contains(&s), "Sine out of range: {}", s);
        assert!((-1..=1).contains(&c), "Cosine out of range: {}", c);

        // Verify consistency
        let (s2, c2) = sin_cos(theta);
        assert_eq!(s, s2);
        assert_eq!(c, c2);
    }
}
