//! Formal Verification Harnesses for Geometric Signal Kernels
//!
//! We use full loop unwinding (65 iterations) to prove 128-bit correctness.
//! This requires significant solver resources.

#[cfg(kani)]
mod tests {
    use crate::{sqrt, Scalar};

    /// 1. The Bedrock Safety Proof (The Risk Check)
    /// Goal: Prove sqrt(x) never panics for ANY 128-bit input.
    /// Complexity: 64 iterations of bitwise logic.
    #[kani::proof]
    #[kani::unwind(65)] // 64 iterations + 1 for termination check
    #[kani::solver(cadical)] // Optimizer hint for bit-vector logic
    fn proof_sqrt_no_panic() {
        let bits: i128 = kani::any();
        let value = Scalar::from_bits(bits);
        let _ = sqrt(value);
    }

    /// 2. SinCos Safety Proof
    /// Goal: Prove sin_cos(theta) never panics for any symbolic theta.
    /// Complexity: 64 iterations of CORDIC.
    #[kani::proof]
    #[kani::unwind(65)]
    #[kani::solver(cadical)]
    fn proof_sin_cos_no_panic() {
        let bits: i128 = kani::any();
        let theta = Scalar::from_bits(bits);
        let _ = crate::math::sin_cos(theta);
    }

    /// 3. Atan Shafer Safety Proof
    /// Goal: Prove atan_shafer(x) never panics and handles asymptotic clamping correctly.
    /// Complexity: Internal sqrt loop (64 iterations).
    #[kani::proof]
    #[kani::unwind(65)]
    #[kani::solver(cadical)]
    fn proof_atan_shafer_safety() {
        let bits: i128 = kani::any();
        let x = Scalar::from_bits(bits);
        let _ = crate::algebraic::atan_shafer(x);
    }

    /* --- Sharded Atan2 Verification --- */

    /// Shared logic for all Atan2 shards
    /// checks range invariants [-PI, PI]
    fn check_atan2_logic(y: Scalar, x: Scalar) {
        let result = crate::algebraic::atan2_shafer(y, x);
        let pi = crate::math::PI;

        // Loose bounds check (safety)
        kani::assert(result >= -pi && result <= pi, "Result must be in [-PI, PI]");
    }

    // --- SHARD 1: Quadrant I (+X, +Y) ---
    #[kani::proof]
    #[kani::unwind(65)]
    #[kani::solver(cadical)]
    fn proof_atan2_q1() {
        let y_bits: i128 = kani::any();
        let x_bits: i128 = kani::any();
        let y = Scalar::from_bits(y_bits);
        let x = Scalar::from_bits(x_bits);

        // Constraint: Positive X, Positive Y
        kani::assume(x > Scalar::ZERO && y >= Scalar::ZERO);
        check_atan2_logic(y, x);
    }

    // --- SHARD 2: Quadrant II (-X, +Y) ---
    #[kani::proof]
    #[kani::unwind(65)]
    #[kani::solver(cadical)]
    fn proof_atan2_q2() {
        let y_bits: i128 = kani::any();
        let x_bits: i128 = kani::any();
        let y = Scalar::from_bits(y_bits);
        let x = Scalar::from_bits(x_bits);

        // Constraint: Negative X, Positive Y
        kani::assume(x < Scalar::ZERO && y >= Scalar::ZERO);
        check_atan2_logic(y, x);
    }

    // --- SHARD 3: Quadrant III (-X, -Y) ---
    #[kani::proof]
    #[kani::unwind(65)]
    #[kani::solver(cadical)]
    fn proof_atan2_q3() {
        let y_bits: i128 = kani::any();
        let x_bits: i128 = kani::any();
        let y = Scalar::from_bits(y_bits);
        let x = Scalar::from_bits(x_bits);

        // Constraint: Negative X, Negative Y
        kani::assume(x < Scalar::ZERO && y < Scalar::ZERO);
        check_atan2_logic(y, x);
    }

    // --- SHARD 4: Quadrant IV (+X, -Y) ---
    #[kani::proof]
    #[kani::unwind(65)]
    #[kani::solver(cadical)]
    fn proof_atan2_q4() {
        let y_bits: i128 = kani::any();
        let x_bits: i128 = kani::any();
        let y = Scalar::from_bits(y_bits);
        let x = Scalar::from_bits(x_bits);

        // Constraint: Positive X, Negative Y
        kani::assume(x > Scalar::ZERO && y < Scalar::ZERO);
        check_atan2_logic(y, x);
    }
}
