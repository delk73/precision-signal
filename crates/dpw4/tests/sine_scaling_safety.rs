//! Sine Scaling Safety — Deterministic Validation (Δ-04 / OQ-5 / OQ-6)
//!
//! Closes OQ-6: establishes that `|s| ≤ 1.0` for all CORDIC outputs and that
//! `s * SINE_EGRESS_SCALE` never overflows I64F64 or i32.
//!
//! Closes OQ-5: confirms `.to_num::<i32>()` performs signed integer-part extraction — sign symmetry,
//! monotonicity near zero, and integer-part bit equivalence.
//!
//! No floats. No proptest. Deterministic. Does NOT modify production code.

use dpw4::math;
use dpw4::Scalar;
use dpw4::{SINE_EGRESS_SCALE, SINE_EGRESS_SCALE_Q31};

/// Calibrated sine egress scale constant used by shape=4.
const SINE_SCALE: Scalar = SINE_EGRESS_SCALE;
const ONE: Scalar = Scalar::ONE;

// =============================================================================
// OQ-6: Scaling Safety — Phase Sweep
// =============================================================================

/// Sweep 4097 phases across [0, 2π) and verify:
///   (a) |s| ≤ 1.0  (CORDIC bound, per sin_cos_fast contract)
///   (b) `s * SINE_SCALE` does not panic (no I64F64 integer overflow)
///   (c) `.to_num::<i32>()` result ∈ [−SINE_EGRESS_SCALE_Q31, SINE_EGRESS_SCALE_Q31]
#[test]
fn test_sine_scaling_safety_sweep() {
    // N+1 points so we include both endpoints of representative coverage.
    let n: u32 = 4096;

    // max_abs_s_bits tracks the maximum |s| seen expressed as I64F64 bits,
    // so we can assert the empirical ceiling without float conversion.
    let mut max_abs_s_bits: u128 = 0;

    for i in 0..=n {
        // Phase as exact rational i/n * 2π, computed entirely in fixed-point.
        // Scalar::from_num(i) * (TWO_PI / Scalar::from_num(n)) — fully fixed.
        let phase = Scalar::from_num(i) * (math::TWO_PI / Scalar::from_num(n));

        let (s, _c) = math::sin_cos_fast(phase);

        // (a) OQ-6 bound: |s| ≤ 1.0
        assert!(
            s >= -ONE && s <= ONE,
            "CORDIC bound violated at i={}: s bits = {:#x}",
            i,
            s.to_bits()
        );

        // Track empirical maximum |s| in bit-repr (unsigned abs of I64F64 bits).
        let abs_bits = s.to_bits().unsigned_abs();
        if abs_bits > max_abs_s_bits {
            max_abs_s_bits = abs_bits;
        }

        // (b) Multiply: must not panic (I64F64 has 64 integer-bit headroom; result ≤ 2^31)
        let scaled: Scalar = s * SINE_SCALE;

        // (c) Convert: result must be in calibrated range.
        let q: i32 = scaled.to_num::<i32>();
        assert!(
            (-SINE_EGRESS_SCALE_Q31..=SINE_EGRESS_SCALE_Q31).contains(&q),
            "to_num::<i32>() out of range at i={}: q={}, s bits={:#x}",
            i,
            q,
            s.to_bits()
        );
    }

    // Empirical ceiling: max_abs_s ≤ 1.0 exactly.
    // ONE in I64F64 bits = 1 << 64 = 0x0000_0001_0000_0000_0000_0000_0000_0000
    let one_bits: u128 = ONE.to_bits().unsigned_abs();
    assert!(
        max_abs_s_bits <= one_bits,
        "max |s| exceeds 1.0: max_abs_s_bits={:#x}, one_bits={:#x}",
        max_abs_s_bits,
        one_bits
    );
}

// =============================================================================
// OQ-5: Quantizer Semantics — Sign Symmetry
// =============================================================================

/// sin(θ + π) ≈ −sin(θ) for all θ.
/// Scaled results must be sign-symmetric (exact or differing by at most 1 ULP
/// from the CORDIC approximation).
///
/// For our purposes: sign(q(θ)) and sign(q(θ+π)) must be opposite,
/// or both zero (near-zero crossing).
#[test]
fn test_sine_scaling_sign_symmetry() {
    let n: u32 = 4096;

    for i in 0..n {
        let phase_a = Scalar::from_num(i) * (math::TWO_PI / Scalar::from_num(n));
        // θ + π
        let phase_b = phase_a + math::PI;

        let (s_a, _) = math::sin_cos_fast(phase_a);
        let (s_b, _) = math::sin_cos_fast(phase_b);

        let q_a: i32 = (s_a * SINE_SCALE).to_num::<i32>();
        let q_b: i32 = (s_b * SINE_SCALE).to_num::<i32>();

        // Sign symmetry: signs must be opposite or both zero.
        // Both zero is allowed (near zero-crossing; CORDIC rounding may land on same side).
        let both_zero = q_a == 0 && q_b == 0;
        let opposite_sign = (q_a > 0 && q_b < 0) || (q_a < 0 && q_b > 0);

        // Near-zero: allow |q| ≤ 1 discrepancy (CORDIC approximation + integer-part extraction near zero crossing)
        let near_zero_crossing = q_a.unsigned_abs() <= 1 || q_b.unsigned_abs() <= 1;

        assert!(
            both_zero || opposite_sign || near_zero_crossing,
            "sign symmetry violated at i={}: q_a={}, q_b={} (expected opposite signs)",
            i,
            q_a,
            q_b
        );
    }
}

// =============================================================================
// OQ-5: Quantizer Semantics — Integer-Part Extraction (I64F64 >> 64)
// =============================================================================

/// Confirms `.to_num::<i32>()` extracts the signed integer part of an I64F64 value.
///
/// In two's-complement I64F64, the integer part is the upper 64 bits interpreted
/// as a signed i64. For fractional values, this means:
///   - Positive `n.f` → integer part = n  (discards fractional bits downward)
///   - Negative `-(n.f)` in two's complement → integer part = -(n+1) for f ≠ 0
///     (equivalently: floor toward negative infinity)
///
/// This is exactly what the `fixed` 1.30.0 documentation specifies:
/// for signed fixed-point, conversion to integer targets is equivalent to
/// extracting the signed integer part (upper 64 bits of I64F64) within range.
/// The Kani harness
/// `proof_sine_to_i32_in_range` independently verifies bit-extraction equivalence
/// holds for all |s| ≤ 1.0. This test is a concrete deterministic check.
#[test]
fn test_sine_scaling_integer_part_extraction() {
    // Case A: positive fractional → integer part = n
    // Construct I64F64 = 1073741823.5 via direct bit pattern.
    // Upper 64 bits = 1073741823 (integer part), lower 64 bits = 1<<63 (= 0.5 fractional).
    let pos_n: i32 = 1073741823;
    let pos_bits: i128 = ((pos_n as i128) << 64) | (1_i128 << 63);
    let scaled_pos = Scalar::from_bits(pos_bits);
    let q_pos: i32 = scaled_pos.to_num::<i32>();
    assert_eq!(
        q_pos, pos_n,
        "positive fractional: integer part must be {} (got {})",
        pos_n, q_pos
    );

    // Case B: negative fractional → integer part = -(n+1) in two's-complement
    // Construct I64F64 = -(pos_n + 1) + 0.5 = -1073741823.5 in two's-complement.
    // Two's-complement negation of (pos_n.5):
    //   positive repr = (1073741823 << 64) | (1<<63)
    //   negation      = -((1073741823 << 64) | (1<<63))
    // The upper 64 bits of this i128 = -(1073741823+1) = -1073741824.
    let neg_bits: i128 = -pos_bits;
    let scaled_neg = Scalar::from_bits(neg_bits);
    let q_neg: i32 = scaled_neg.to_num::<i32>();
    // For negative values with nonzero fractional part, integer-part extraction
    // yields -(n+1) (floor toward negative infinity). This is defined behavior.
    assert_eq!(
        q_neg,
        -(pos_n + 1),
        "negative fractional: integer part must be {} (got {})",
        -(pos_n + 1),
        q_neg
    );

    // Case C: confirm bit-extraction equivalence for the same values —
    // cross-check that to_num equals the upper-64-bit signed integer part.
    let int_pos: i32 = (scaled_pos.to_bits() >> 64) as i32;
    let int_neg: i32 = (scaled_neg.to_bits() >> 64) as i32;
    assert_eq!(q_pos, int_pos, "pos: to_num must equal bit-extraction");
    assert_eq!(q_neg, int_neg, "neg: to_num must equal bit-extraction");
}

// =============================================================================
// OQ-5: Quantizer Semantics — Equivalence to Signed Integer-Part Bit Extraction
// =============================================================================

/// `.to_num::<i32>()` equals the signed integer-part extracted from the upper
/// 64 bits of the I64F64 128-bit representation, cast to i32.
/// This is the mechanical grounding of the "signed integer-part extraction" claim.
#[test]
fn test_sine_scaling_bit_extraction_equivalence() {
    let n: u32 = 4096;

    for i in 0..=n {
        let phase = Scalar::from_num(i) * (math::TWO_PI / Scalar::from_num(n));
        let (s, _) = math::sin_cos_fast(phase);

        let scaled: Scalar = s * SINE_SCALE;

        // Method A: standard API
        let via_to_num: i32 = scaled.to_num::<i32>();

        // Method B: integer-part bit extraction.
        // I64F64 = fixed<i128, 64>: upper 64 bits are the signed integer part.
        // to_bits() returns i128; shift right 64 gives i64 integer part.
        let int_part_i64: i64 = (scaled.to_bits() >> 64) as i64;

        // Integer part must fit i32 given |s| ≤ 1 and SINE_SCALE < 2^31.
        assert!(
            int_part_i64 >= i32::MIN as i64 && int_part_i64 <= i32::MAX as i64,
            "integer part out of i32 range at i={}: int_part_i64={}",
            i,
            int_part_i64
        );

        let via_bits: i32 = int_part_i64 as i32;

        assert_eq!(
            via_to_num, via_bits,
            "bit extraction mismatch at i={}: via_to_num={}, via_bits={}",
            i, via_to_num, via_bits
        );
    }
}
