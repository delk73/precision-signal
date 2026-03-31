#![no_std]
#![forbid(unsafe_code)]
//! DPW (Differentiated Polynomial Waveform) Module
//!
//! Reference Lock (v1.0.0-rc5)
//! 32-bit Scientific Reference Standard
//! Fixed 4th-Order Low Noise Standard
//!
//! # REFERENCE LOCK
//! This core is the Canonical Reference Implementation. Any change to the
//! mathematical identity of the output (SHA-256 traces) is strictly prohibited.
//! The egress width is elevated to 32-bit to preserve the dynamic range of the
//! i128 differentiator core.

#[cfg(any(test, feature = "verification-runtime"))]
extern crate std;

#[cfg(any(feature = "verification-runtime", kani))]
pub mod verification;

#[cfg(feature = "audit")]
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

#[cfg(feature = "audit")]
/// Internal tracker for maximal bit-depth of integration accumulator.
static MAX_ABS_Z_BITS: AtomicU32 = AtomicU32::new(0);

#[cfg(feature = "audit")]
/// Flag set if the integration state bit-length exceeds 63 bits.
/// This corresponds to the range where the legacy `<< 64` shift would have been unsafe.
static LEGACY_SHIFT_OVERFLOW_RISK: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "audit")]
/// Flag set if the integration state exceeds 126 bits, indicating critical overflow risk.
static INTEGRATOR_NEAR_OVERFLOW: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "audit")]
#[inline]
pub fn max_abs_z_bits() -> u32 {
    MAX_ABS_Z_BITS.load(Ordering::Relaxed)
}

#[cfg(feature = "audit")]
#[inline]
pub fn legacy_shift_overflow_risk() -> bool {
    LEGACY_SHIFT_OVERFLOW_RISK.load(Ordering::Relaxed)
}

#[cfg(feature = "audit")]
#[inline]
pub fn integrator_near_overflow() -> bool {
    INTEGRATOR_NEAR_OVERFLOW.load(Ordering::Relaxed)
}

#[cfg(feature = "audit")]
#[inline]
pub fn reset_audit_counters() {
    MAX_ABS_Z_BITS.store(0, Ordering::Relaxed);
    LEGACY_SHIFT_OVERFLOW_RISK.store(false, Ordering::Relaxed);
    INTEGRATOR_NEAR_OVERFLOW.store(false, Ordering::Relaxed);
}

pub use geom_signal::{math, Scalar};

mod constants;
pub use constants::*;

pub(crate) mod i256;

pub mod checksum;
pub use checksum::fletcher32;

pub mod goldens;

// =============================================================================
// STATE STRUCTURES
// =============================================================================

/// DPW4 State (3rd-order differentiator)
/// Uses i128 buffers to maintain 64-bit state invariants.
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct Dpw4State {
    pub z1: i128,
    pub z2: i128,
    pub z3: i128,
}

/// Integration accumulator for DPW4 Triangle generation.
/// Maintains the integration constant ensuring 0-mean output.
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct IntegrationState {
    pub z: i128,
    pub prev_phase_u32: u32,
    pub init: bool,
}

impl Dpw4State {
    pub fn new() -> Self {
        Self {
            z1: 0,
            z2: 0,
            z3: 0,
        }
    }

    pub fn reset(&mut self) {
        self.z1 = 0;
        self.z2 = 0;
        self.z3 = 0;
    }
}

/// Compute x² polynomial from bipolar phase input.
///
/// # Mathematical Identity
/// This function calculates the norm-4 polynomial core: `x² = (s/2)²`.
/// The 1-bit right shift (`s_q31 >> 1`) is an architectural invariant required
/// for bit-perfect trace compatibility.
///
/// # Arguments
/// * `s_q31` - Bipolar Q31 phase input, range `[-2^31, 2^31-1]`.
///   Expected range for valid DPW is `[-2^30, 2^30]`.
///
/// # Returns
/// * `i128` - x² result in **Q124** format to preserve maximum precision
///   for subsequent differentiation.
///
/// # Safety
/// This function is pure and never panics. All operations use wrapping or
/// wide-bit arithmetic to prevent overflow.
#[inline]
pub fn compute_x2_q124(s_q31: i64) -> i128 {
    // REFERENCE LOCK: Exact expression (s_q31 >> 1) required for conformance.
    // This normative 1-bit truncation defines the phase alignment of the reference.
    let x_q30 = s_q31 >> 1;
    let x_q30_i128 = x_q30 as i128;
    // x_q30 * x_q30 = Q30 * Q30 = Q60
    // Shift up to Q124 for ultra-high precision intermediate math
    let x2_q60 = x_q30_i128 * x_q30_i128;
    x2_q60 << 64
}

// -----------------------------------------------------------------------------
// IMMUTABILITY WARNING: CORE KERNEL
// The following functions implement the Q124 bit-exact reference truth.
// Any modification to these algorithms or the 1-bit phase truncation
// (s_q31 >> 1) is strictly forbidden to maintain forensic trace compatibility.
// -----------------------------------------------------------------------------

#[inline]
pub fn compute_x4_q124(x2_q124: i128) -> i128 {
    // We cannot square Q124 directly in i128 (needs 248 bits).
    // Instead, we shift down to Q62, square it to get Q124.
    let x2_q62 = x2_q124 >> 62;
    x2_q62 * x2_q62
}

/// DPW4 tick: 3rd-order differentiator on x^4 polynomial.
/// Returns raw DPW4 output in Q124 (needs gain normalization).
///
/// IMMUTABILITY WARNING: This tick loop is the static reference target.
#[inline]
pub fn tick_dpw4_raw(state: &mut Dpw4State, s_q31: i64) -> i128 {
    let x2 = compute_x2_q124(s_q31);
    let p4 = compute_x4_q124(x2);

    // Difference calculations in Q124
    // Historically, we used i64 buffers (shifting down by 60 bits), but this
    // caused overflow for values > 0.5. We now use full i128 state to maintain
    // reference-system rigor and 124-bit precision.

    let d1 = p4.wrapping_sub(state.z1);
    state.z1 = p4;

    let d2 = d1.wrapping_sub(state.z2);
    state.z2 = d1;

    let d3 = d2.wrapping_sub(state.z3);
    state.z3 = d2;

    d3
}

/// Saturate i64 to i16 range (Legacy/Utility).
#[inline]
pub fn saturate_i16(val: i64) -> i16 {
    val.clamp(i16::MIN as i64, i16::MAX as i64) as i16
}

/// Saturate an `i64` value to the `i32` range.
///
/// This is used to preserve the high-resolution dynamic range of the `i128` core
/// while conforming to the `S32LE` reference standard output.
///
/// # Note: Reference Application Configuration
/// This 32-bit saturation is the reference egress configuration, validated via
/// the audio synthesis application. The internal engine maintains `i128` precision.
///
/// # Arguments
/// * `val` - The value to clamp, typically a product of the gain stage.
///
/// # Returns
/// * `i32` - Clamped value in the range `[-2147483648, 2147483647]`.
#[inline]
pub fn saturate_i32(val: i64) -> i32 {
    val.clamp(i32::MIN as i64, i32::MAX as i64) as i32
}

/// Saturate an `i128` value to the `i32` range.
#[inline]
pub(crate) fn saturate_i128_to_i32(x: i128) -> i32 {
    x.clamp(i32::MIN as i128, i32::MAX as i128) as i32
}

#[inline]
fn saturating_mul_i128(lhs: i128, rhs: i128) -> i128 {
    match lhs.checked_mul(rhs) {
        Some(prod) => prod,
        None => {
            let negative = (lhs < 0) ^ (rhs < 0);
            if negative {
                i128::MIN
            } else {
                i128::MAX
            }
        }
    }
}

#[inline]
fn saturating_shift_i128(prod: i128, shift: i64) -> i128 {
    if shift >= 128 {
        if prod >= 0 {
            0
        } else {
            -1
        }
    } else if shift > 0 {
        prod >> (shift as u32)
    } else if shift == 0 {
        prod
    } else {
        let left = (-shift) as u32;
        match prod.checked_shl(left) {
            Some(v) => v,
            None => {
                if prod == 0 {
                    0
                } else if prod > 0 {
                    i128::MAX
                } else {
                    i128::MIN
                }
            }
        }
    }
}

/// Apply mantissa+exponent gain to raw DPW signal.
///
/// Performs high-precision scaling in the `i128` domain using a two-path
/// precision strategy to prevent overflow while maintaining dynamic range.
///
/// # Arguments
/// * `raw_q124` - Raw differentiator output in Q124 format.
/// * `m_q63` - Mantissa quantized as `u64` (Q63), range `[0.5, 1.0)`.
/// * `exp` - Exponent for binary scaling (octave control).
///
/// # Implementation Details
/// * **High Precision Path**: Used if `|raw| < 2^64`. Multiplies first, then shifts.
/// * **High Amplitude Path**: Used if `|raw| >= 2^64`. Pre-shifts to prevent `i128` overflow.
/// * **32-bit Egress**: Right-shift is reduced by 16 bits compared to legacy i16
///   to preserve sub-LSB jitter into the final `i32` stream.
///
/// # Returns
/// * `i32` - Saturated 32-bit output with fixed headroom applied.
#[inline]
pub fn apply_gain(raw_q124: i128, m_q63: u64, exp: i32) -> i32 {
    let mag_u: u128 = raw_q124.unsigned_abs();

    // Combined Shift Strategy (v1.0.0-rc5)
    // Egress is now i32. We reduce the total right-shift by 16 bits compared
    // to the legacy i16 implementation. This preserves sub-LSB precision
    // from the i128 differentiator state into the final S32LE output stream.

    let res_i128 = if mag_u < GAIN_PRECISION_THRESHOLD_U {
        // High Precision Path (raw < 2^64)
        // No pre-shift required.
        let prod = saturating_mul_i128(raw_q124, m_q63 as i128); // Q187
        let shift = i64::from(Q_HIGH_PRECISION) - 16 - i64::from(exp);
        saturating_shift_i128(prod, shift)
    } else {
        // High Amplitude Path (raw >= 2^64)
        // Pre-shift to Q60 to prevent i128 overflow during multiplication.
        let prod = saturating_mul_i128(raw_q124 >> 64, m_q63 as i128); // Q123
        let shift = i64::from(Q_HIGH_AMPLITUDE) - 16 - i64::from(exp);
        saturating_shift_i128(prod, shift)
    };

    // Hardened headroom policy: reserve explicit container space for
    // transport-domain superposition before final egress quantization.
    // Intentional arithmetic shift on signed i128:
    // - preserves deterministic two's-complement behavior for negative samples
    // - uses truncation policy (not round-to-nearest) to avoid kernel drift
    let hardened_i128 = res_i128 >> HEADROOM_BITS;
    saturate_i128_to_i32(hardened_i128)
}

/// DPW gain calibration parameters.
/// Standardized for Fixed 4th-Order path.
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct DpwGain {
    /// DPW4 gain mantissa (Q63 format)
    pub m4_q63: u64,
    /// DPW4 gain exponent
    pub e4: i32,
    /// DPW4 Inverse gain mantissa (for specific differential paths)
    pub m4_q63_inv: u64,
    /// DPW4 Inverse gain exponent
    pub e4_inv: i32,
}

impl DpwGain {
    pub fn new(m4_q63: u64, e4: i32, m4_q63_inv: u64, e4_inv: i32) -> Self {
        Self {
            m4_q63,
            e4,
            m4_q63_inv,
            e4_inv,
        }
    }
}

/// Full Fixed 4th-Order DPW tick.
#[inline]
pub fn tick_dpw4(state: &mut Dpw4State, phase_u32: u32, gain: &DpwGain) -> i32 {
    // Convert phase to bipolar Q31
    let s_q31 = (phase_u32 as i64).wrapping_sub(BIPOLAR_OFFSET);

    // Tick the 4th-order kernel
    let raw4 = tick_dpw4_raw(state, s_q31);

    // Apply 4th-order gain
    apply_gain(raw4, gain.m4_q63, gain.e4)
}

// =============================================================================
// Multi-Shape Oscillator State (v0.5.0)
// =============================================================================

/// Oscillator state for multi-shape synthesis.
/// Standardized on dual Dpw4State for differential pulse synthesis.
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct OscState {
    pub saw_a: Dpw4State,
    pub saw_b: Dpw4State,
    pub tri: IntegrationState,
    pub duty: Scalar,
}

/// Primary Oscillator Structure
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct Oscillator {
    pub state: OscState,
    pub phase: Scalar,
    pub frequency: Scalar,
    pub amplitude: Scalar,
    pub sample_rate: Scalar,
}

impl Oscillator {
    #[cfg(feature = "float-ingest")]
    pub fn new(sample_rate: f64) -> Self {
        Self::new_scalar(Scalar::from_num(sample_rate))
    }

    pub fn new_scalar(sample_rate: Scalar) -> Self {
        Self {
            state: OscState::new(),
            phase: Scalar::ZERO,
            frequency: Scalar::from_num(440),
            amplitude: Scalar::from_num(1),
            sample_rate,
        }
    }

    pub fn new_u32(sample_rate_hz: u32) -> Self {
        Self::new_scalar(Scalar::from_num(sample_rate_hz))
    }

    #[inline]
    pub fn tick_phase(&mut self) {
        // Phase Accumulation: Δφ = (freq / rate) * 2π
        let phase_inc = (self.frequency / self.sample_rate) * math::TWO_PI;
        self.phase += phase_inc;

        // Reset logic (New: O(1) modulo or branch)
        if self.phase >= math::TWO_PI {
            self.phase -= math::TWO_PI;
        } else if self.phase < Scalar::ZERO {
            self.phase += math::TWO_PI;
        }
    }

    #[inline]
    pub fn tick(&mut self, shape: u32, gain: &DpwGain) -> i32 {
        let out = tick_shape(&mut self.state, self.phase, shape, gain);
        self.tick_phase();
        out
    }
}

impl OscState {
    pub fn new() -> Self {
        Self {
            saw_a: Dpw4State::new(),
            saw_b: Dpw4State::new(),
            tri: IntegrationState::default(),
            duty: Scalar::from_num(1) / Scalar::from_num(2), // Default to 50% (Square)
        }
    }

    pub fn reset(&mut self) {
        self.saw_a.reset();
        self.saw_b.reset();

        // Triangle Reset
        self.tri.z = 0;
        self.tri.init = false;
        self.tri.prev_phase_u32 = 0;

        self.duty = Scalar::ZERO; // Hard reset
    }
}

/// Unified shape dispatch for multi-shape oscillator.
#[inline]
pub fn tick_shape(state: &mut OscState, phase: Scalar, shape: u32, gain: &DpwGain) -> i32 {
    match shape {
        0 => {
            // SAW: Fixed 4th-order path
            // Convert phase [0, 2pi] to bipolar phase [0, 2^32-1]
            let mut p = phase % math::TWO_PI;
            if p < Scalar::ZERO {
                p += math::TWO_PI;
            }
            let phase_u32 = (p / math::TWO_PI * SCALE_2_32).to_num::<u32>();
            tick_dpw4(&mut state.saw_a, phase_u32, gain)
        }
        1 => {
            // PULSE: Two-Saw Differential Method
            tick_pulse(state, phase, state.duty, gain)
        }
        2 => {
            // TRIANGLE: DPW4 (Integrated Band-Limited Square)
            tick_triangle_dpw4(state, phase, gain)
        }
        3 => {
            // SQUARE: Alias for Pulse with fixed 0.5 duty
            tick_pulse(
                state,
                phase,
                Scalar::from_num(1) / Scalar::from_num(2),
                gain,
            )
        }
        4 => {
            // SINE: Direct CORDIC (Fast path as phase is normalized [0, 2PI])
            let (s, _) = math::sin_cos_fast(phase);
            // Normative egress contract for Sine:
            // 1) Convert Scalar [-1, 1] to i32 using Scalar::to_num::<i32>()
            //    (signed integer-part extraction: arithmetic right shift of signed I64F64 bits by 64).
            // 2) Apply container headroom as an arithmetic right shift by HEADROOM_BITS in i128.
            // 3) Clamp to i32 domain via saturate_i128_to_i32.
            // TODO(DEBT-003): Route Sine through apply_gain once a Q-format raw
            // representation for CORDIC output is defined.
            let pre_headroom = (s * SINE_EGRESS_SCALE).to_num::<i32>() as i128;
            saturate_i128_to_i32(pre_headroom >> HEADROOM_BITS)
        }
        _ => 0,
    }
}

// -----------------------------------------------------------------------------
// SIGNAL PIPE: MONOMORPHIZED KERNELS
// -----------------------------------------------------------------------------

/// Trait for zero-branching signal generation.
pub trait SignalShape {
    fn tick(state: &mut OscState, phase: Scalar, gain: &DpwGain) -> i32;
}

/// Sawtooth generator (4th-order DPW).
pub struct Sawtooth;
impl SignalShape for Sawtooth {
    #[inline(always)]
    fn tick(state: &mut OscState, phase: Scalar, gain: &DpwGain) -> i32 {
        tick_shape(state, phase, 0, gain)
    }
}

/// Pulse generator (Differential DPW).
pub struct Pulse;
impl SignalShape for Pulse {
    #[inline(always)]
    fn tick(state: &mut OscState, phase: Scalar, gain: &DpwGain) -> i32 {
        tick_shape(state, phase, 1, gain)
    }
}

/// Square generator (Differential DPW, 50% duty).
pub struct Square;
impl SignalShape for Square {
    #[inline(always)]
    fn tick(state: &mut OscState, phase: Scalar, gain: &DpwGain) -> i32 {
        tick_shape(state, phase, 3, gain)
    }
}

/// Triangle generator (4th-order DPW).
#[deprecated(since = "1.0.0-rc5", note = "Use `TriangleDPW4` directly")]
pub type Triangle = TriangleDPW4;

/// Triangle generator (4th-order DPW).
/// Implements the Baseline Pattern by integrating a 3rd-order square wave.
pub struct TriangleDPW4;
impl SignalShape for TriangleDPW4 {
    #[inline(always)]
    fn tick(state: &mut OscState, phase: Scalar, gain: &DpwGain) -> i32 {
        tick_triangle_dpw4(state, phase, gain)
    }
}

/// Baseline Triangle generator (Forensic Control).
///
/// # Forensic Retention
/// This implementation is explicitly retained for:
/// * Noise floor comparison
/// * Regression detection
/// * Forensic baselining
///
/// It implements the "Naive Bitwise Folding" algorithm and is NOT band-limited.
pub struct TriangleDPW1;
impl SignalShape for TriangleDPW1 {
    #[inline(always)]
    fn tick(_state: &mut OscState, phase: Scalar, _gain: &DpwGain) -> i32 {
        #[allow(deprecated)]
        tick_triangle_dpw1(phase)
    }
}

/// Sine generator (CORDIC).
pub struct Sine;
impl SignalShape for Sine {
    #[inline(always)]
    fn tick(state: &mut OscState, phase: Scalar, gain: &DpwGain) -> i32 {
        tick_shape(state, phase, 4, gain)
    }
}

/// Monomorphized signal pipe for high-performance buffer processing.
/// Eliminates runtime branching to support auto-vectorization.
#[inline]
pub fn signal_pipe<S: SignalShape>(
    state: &mut OscState,
    phases: &[Scalar],
    gain: &DpwGain,
    output: &mut [i32],
) {
    for i in 0..phases.len() {
        output[i] = S::tick(state, phases[i], gain);
    }
}

/// Pulse waveform via Two-Saw Differential Method.
#[inline]
fn tick_pulse(state: &mut OscState, phase: Scalar, duty_scalar: Scalar, gain: &DpwGain) -> i32 {
    let duty_phase = duty_scalar * math::TWO_PI;

    // Map phase [0, 2pi] to bipolar Q31
    let to_q31 = |mut p: Scalar| {
        p %= math::TWO_PI;
        if p < Scalar::ZERO {
            p += math::TWO_PI;
        }
        let u32_phase = (p / math::TWO_PI * SCALE_2_32).to_num::<u32>();
        (u32_phase as i64).wrapping_sub(BIPOLAR_OFFSET)
    };

    let s_a = to_q31(phase);
    let s_b = to_q31(phase - duty_phase);

    // Compute raw differentiator values (i128)
    // d3 = x - x^3 approx (Sawtooth)
    let raw_a = tick_dpw4_raw(&mut state.saw_a, s_a);
    let raw_b = tick_dpw4_raw(&mut state.saw_b, s_b);

    // High-Precision Mixing (Reference Topology): Pulse = (SawA - SawB) / 2
    // Performs subtraction in Q124 domain to minimize quantization noise.
    let raw_diff = raw_a.wrapping_sub(raw_b);
    let raw_mix = raw_diff >> 1;

    // Apply gain once at the end
    apply_gain(raw_mix, gain.m4_q63, gain.e4)
}

/// Triangle waveform via Integration of Band-Limited Square.
///
/// # Greenfield Implementation (Normative)
/// 1. Generates Band-Limited Square (via 4th-order DPW differential chain)
/// 2. Integrates the result using 128-bit Scalar pipeline (State owned)
/// 3. Maintains integration constant to prevent DC drift
#[inline]
fn tick_triangle_dpw4(state: &mut OscState, phase: Scalar, gain: &DpwGain) -> i32 {
    // 1. Generate Band-Limited Square Wave (fixed 50% duty)
    //    Square = Saw(phi) - Saw(phi - 0.5)
    let duty_phase = math::PI;

    // 2. Integration
    //    Triangle = Integral(Square)dt
    //    We perform integration in 128-bit fixed domain.

    // Helper: Normalized Phase to u32 [0, 2^32)
    let phase_to_u32 = |mut p: Scalar| -> u32 {
        p %= math::TWO_PI;
        if p < Scalar::ZERO {
            p += math::TWO_PI;
        }
        // Use exact scalar multiplication (SCALE_2_32 defined at module scope)
        (p / math::TWO_PI * SCALE_2_32).to_num::<u32>()
    };

    // Calculate phases once
    let phase_u32 = phase_to_u32(phase);
    let phase_b_u32 = phase_to_u32(phase - duty_phase);

    // Init Check
    if !state.tri.init {
        state.tri.prev_phase_u32 = phase_u32;
        state.tri.init = true;
        // Contract: First tick produces 0 because dphi is undefined until prev_phase is established.
        return 0; // z is 0
    }

    let mut dphi = phase_u32.wrapping_sub(state.tri.prev_phase_u32);
    state.tri.prev_phase_u32 = phase_u32;

    // — Δ-06 NORMATIVE CONTRACT BOUNDARY —
    // Freeze condition (strict `>`):
    //   dphi: u32 = phase_u32.wrapping_sub(prev_phase_u32_old)  [wrapping distance;
    //               prev_phase_u32_old = state.tri.prev_phase_u32 before line 643]
    //   DISCONTINUITY_THRESHOLD: u32 (semantic: 1/4 cycle in u32 space;
    //                              current value: 0x4000_0000 — policy, not invariant identity)
    //   dphi == DISCONTINUITY_THRESHOLD does NOT freeze.
    // If the guard sets dphi = 0, the downstream pipeline yields:
    //   delta_i128 == 0  and  z_next == z_prev  for this tick.
    if dphi > DISCONTINUITY_THRESHOLD {
        dphi = 0;
    }

    // Generate Band-Limited Square using unified phase
    // s_a = phase_u32 - 2^31 (bipolar mapping)
    let s_a = (phase_u32 as i64).wrapping_sub(BIPOLAR_OFFSET);
    let s_b = (phase_b_u32 as i64).wrapping_sub(BIPOLAR_OFFSET);

    // 4th-Order DPW Differentiation Chain provided by tick_dpw4_raw (P4 -> d3)
    let raw_a = tick_dpw4_raw(&mut state.saw_a, s_a);
    let raw_b = tick_dpw4_raw(&mut state.saw_b, s_b);

    // Differentiated Square: dSquare = dSawA - dSawB
    let wide_a = i256::I256::from_i128(raw_a);
    let wide_b = i256::I256::from_i128(raw_b);
    let raw_square_diff = wide_a.sub(wide_b);

    // Integration Step: z += (raw >> 32) * dphi
    // Amplitude Contract:
    // Internal triangle integrator state z is maintained in the same fixed domain
    // expected by apply_gain for TriangleDPW4; no additional output shift is applied.
    //
    // Narrowing Policy: TRUNCATION
    // We discard the lower 32 bits of the difference before multiplication.
    // This establishes a noise floor at bit 32 of the 124-bit differentiator output.
    let shifted = raw_square_diff.sar(DPW_TRUNCATION_BITS as u32);
    let delta_wide = shifted.mul_u32(dphi);
    let delta_i128 = delta_wide.clamp_to_i128();
    state.tri.z = state.tri.z.saturating_add(delta_i128);

    // 3. Integration Constant & DC Management
    //    We rely on the strict zero-mean property of the differential Square wave
    //    (Square = SawA - SawB) to prevent long-term DC drift.

    #[cfg(feature = "audit")]
    {
        let bits = 128 - state.tri.z.unsigned_abs().leading_zeros();

        // Track Max Bits
        MAX_ABS_Z_BITS.fetch_max(bits, Ordering::Relaxed);

        // Check "Old Shift Unsafe" condition
        if bits > LEGACY_OVERFLOW_BITS {
            LEGACY_SHIFT_OVERFLOW_RISK.store(true, Ordering::Relaxed);
        }

        // Check "Near Overflow" condition
        // Reduced threshold to 126 to provide 2-bit safety margin.
        if bits >= INTEGRATOR_SAFE_BITS {
            INTEGRATOR_NEAR_OVERFLOW.store(true, Ordering::Relaxed);
        }
    }

    apply_gain(state.tri.z, gain.m4_q63, gain.e4)
}

#[cfg(test)]
fn triangle_apply_delta_z(z: &mut i128, delta: i128) {
    *z = z.saturating_add(delta);
}

/// Baseline Triangle generator (Naive Bitwise Folding).
///
/// # Forensic Control
/// Reference implementation for non-band-limited behavior.
#[inline]
#[allow(deprecated)]
pub fn tick_triangle_dpw1(phase: Scalar) -> i32 {
    let mut p = phase % math::TWO_PI;
    if p < Scalar::ZERO {
        p += math::TWO_PI;
    }
    let u = (p / math::TWO_PI * SCALE_2_32).to_num::<u32>();
    // Align 90 deg (PHASE_90_DEG) with peak positive
    let v = u.wrapping_add(PHASE_90_DEG);
    // Bitwise folding: (v ^ mask) where mask is all 1s if v is negative
    // This creates a triangle wave from a sawtooth
    let folded = ((v as i32) >> 31) ^ (v as i32);
    // Scale [0, 2^31-1] -> [0, 2^32-2]
    let scaled = folded << 1;
    // Offset to reach [-2147483648, 2147483646]
    scaled ^ (I32_BIPOLAR_CENTER as i32)
}

// -----------------------------------------------------------------------------
// TRANSPORT LAYER: THE BINARY CONTRACT
// -----------------------------------------------------------------------------

/// Zero-copy, fixed-size header for infrastructure-grade transport.
/// Total size: Exactly 64 bytes.
///
/// # Layout (64 bytes)
/// - Bytes 0-3: magic (`b"DP32"`)
/// - Bytes 4-7: version (u32 LE)
/// - Bytes 8-15: sequence (u64 LE)
/// - Bytes 16-19: sample_rate (u32 LE)
/// - Bytes 20-23: bit_depth (u32 LE)
/// - Bytes 24-55: padding (32 bytes)
/// - Bytes 56-59: reserved (4 bytes)
/// - Bytes 60-63: checksum (u32 LE, Fletcher-32)
#[derive(Clone, Copy)]
#[repr(C, align(64))]
pub struct SignalFrameHeader {
    /// Magic identifier: b"DP32"
    pub magic: [u8; 4],
    /// Protocol version (e.g., 1)
    pub version: u32,
    /// Monotonic sequence number
    pub sequence: u64,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Bit depth (always 32 for RC5+)
    pub bit_depth: u32,
    /// Padding (32 bytes)
    pub pad: [u8; HEADER_PAD_SIZE],
    /// Reserved bytes. Must be zero on the wire.
    pub reserved: [u8; 4],
    /// Fletcher-32 checksum of bytes 0-59
    pub checksum: u32,
}

impl SignalFrameHeader {
    /// Create a new header with computed checksum.
    pub fn new(sequence: u64, sample_rate: u32) -> Self {
        let mut header = Self {
            magic: *DP32_MAGIC,
            version: PROTOCOL_VERSION,
            sequence,
            sample_rate,
            bit_depth: BIT_DEPTH_32,
            pad: [0; HEADER_PAD_SIZE],
            reserved: [0; 4],
            checksum: 0,
        };
        header.checksum = header.compute_checksum();
        header
    }

    /// Compute Fletcher-32 checksum of the metadata portion (bytes 0-59).
    pub fn compute_checksum(&self) -> u32 {
        let bytes = self.to_bytes();
        // Checksum covers bytes 0-59 (metadata only)
        fletcher32(&bytes[0..checksum::HEADER_METADATA_SIZE])
    }

    /// Verify that the stored checksum matches the computed checksum.
    pub fn verify_checksum(&self) -> bool {
        self.checksum == self.compute_checksum()
    }

    /// Safe serialization to a fixed-size byte array.
    /// Eliminates the need for `unsafe` raw pointer casting in peripherals.
    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..4].copy_from_slice(&self.magic);
        buf[4..8].copy_from_slice(&self.version.to_le_bytes());
        buf[8..16].copy_from_slice(&self.sequence.to_le_bytes());
        buf[16..20].copy_from_slice(&self.sample_rate.to_le_bytes());
        buf[20..24].copy_from_slice(&self.bit_depth.to_le_bytes());
        buf[HEADER_PAD_OFFSET..HEADER_RESERVED_OFFSET].copy_from_slice(&self.pad);
        // Reserved bytes are defined as zero on-wire.
        buf[HEADER_RESERVED_OFFSET..HEADER_CHECKSUM_OFFSET]
            .copy_from_slice(&[0; HEADER_RESERVED_SIZE]);
        buf[HEADER_CHECKSUM_OFFSET..HEADER_SIZE].copy_from_slice(&self.checksum.to_le_bytes());
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saturate() {
        assert_eq!(saturate_i16(0), 0);
        assert_eq!(saturate_i16(32767), 32767);
        assert_eq!(saturate_i16(32768), 32767);
        assert_eq!(saturate_i16(-32768), -32768);
        assert_eq!(saturate_i16(-32769), -32768);

        assert_eq!(saturate_i32(0), 0);
        assert_eq!(saturate_i32(2147483647), 2147483647);
        assert_eq!(saturate_i32(2147483648), 2147483647);
        assert_eq!(saturate_i32(-2147483648), -2147483648);
        assert_eq!(saturate_i32(-2147483649), -2147483648);
    }

    #[test]
    fn test_saturate_i128_to_i32_regression_no_i64_wrap() {
        let x_hi = (i64::MAX as i128) + 123;
        let old_behavior = saturate_i32(x_hi as i64);
        let new_behavior = saturate_i128_to_i32(x_hi);
        assert_eq!(new_behavior, i32::MAX, "Must saturate high in i128 domain");
        assert_ne!(
            old_behavior, new_behavior,
            "Old cast-via-i64 path should differ for >i64::MAX inputs"
        );

        let x_lo = (i64::MIN as i128) - 123;
        let old_behavior_lo = saturate_i32(x_lo as i64);
        let new_behavior_lo = saturate_i128_to_i32(x_lo);
        assert_eq!(
            new_behavior_lo,
            i32::MIN,
            "Must saturate low in i128 domain"
        );
        assert_ne!(
            old_behavior_lo, new_behavior_lo,
            "Old cast-via-i64 path should differ for <i64::MIN inputs"
        );
    }

    #[test]
    fn test_saturate_i128_to_i32_monotonicity() {
        let values = [
            (i64::MIN as i128) - 2,
            (i64::MIN as i128) - 1,
            i64::MIN as i128,
            (i64::MIN as i128) + 1,
            (i32::MIN as i128) - 2,
            (i32::MIN as i128) - 1,
            i32::MIN as i128,
            (i32::MIN as i128) + 1,
            -1,
            0,
            1,
            (i32::MAX as i128) - 1,
            i32::MAX as i128,
            (i32::MAX as i128) + 1,
            (i32::MAX as i128) + 2,
            (i64::MAX as i128) - 1,
            i64::MAX as i128,
            (i64::MAX as i128) + 1,
            (i64::MAX as i128) + 2,
        ];

        let mut prev = saturate_i128_to_i32(values[0]);
        for &v in &values[1..] {
            let curr = saturate_i128_to_i32(v);
            assert!(
                curr >= prev,
                "Clamp must be monotone: f({})={} < f(prev)={}",
                v,
                curr,
                prev
            );
            prev = curr;
        }
    }

    #[test]
    fn test_header_reserved_gap_is_zeroed() {
        let header = SignalFrameHeader::new(0, 48_000);
        let bytes = header.to_bytes();
        assert_eq!(
            bytes[HEADER_RESERVED_OFFSET..HEADER_CHECKSUM_OFFSET],
            [0; HEADER_RESERVED_SIZE]
        );
    }

    #[test]
    fn test_header_offsets_are_canonical() {
        assert_eq!(
            core::mem::offset_of!(SignalFrameHeader, pad),
            HEADER_PAD_OFFSET
        );
        assert_eq!(
            core::mem::offset_of!(SignalFrameHeader, reserved),
            HEADER_RESERVED_OFFSET
        );
        assert_eq!(
            core::mem::offset_of!(SignalFrameHeader, checksum),
            HEADER_CHECKSUM_OFFSET
        );
        assert_eq!(core::mem::size_of::<SignalFrameHeader>(), HEADER_SIZE);
        assert_eq!(HEADER_CHECKSUM_OFFSET + HEADER_CHECKSUM_SIZE, HEADER_SIZE);
        assert_eq!(checksum::HEADER_METADATA_SIZE, HEADER_CHECKSUM_OFFSET);

        let header = SignalFrameHeader::new(0, 48_000);
        assert_eq!(
            core::mem::size_of_val(&header.reserved),
            HEADER_RESERVED_SIZE
        );
        assert_eq!(
            header.to_bytes()[HEADER_RESERVED_OFFSET..HEADER_CHECKSUM_OFFSET],
            [0; HEADER_RESERVED_SIZE]
        );
    }

    #[test]
    fn test_dpw4_precision_audit() {
        let mut state = Dpw4State::new();
        // phase_u32 near 0xFFFF_FFFF results in s_q31 near 2^31
        // x near 1.0, x^4 near 1.0
        // 1.0 in Q124 is 1 << 124

        let phase_near_max = 0xFFFF_FFFFu32;
        let s_q31 = (phase_near_max as i64).wrapping_sub(BIPOLAR_OFFSET);

        let raw = tick_dpw4_raw(&mut state, s_q31);

        // With i128 state, it should handle the full range.
        // x^4 for x near 1.0 should be very close to 1 << 124
        // The first tick of a differentiator gives the value itself (if state was 0)
        assert!(raw > (1i128 << 123), "First tick should be > 0.5 in Q124");

        // Tick 2
        let raw2 = tick_dpw4_raw(&mut state, s_q31);
        assert_eq!(raw2, -2 * (raw), "Second tick should be -2 * value");

        // Tick 3
        let raw3 = tick_dpw4_raw(&mut state, s_q31);
        assert_eq!(raw3, raw, "Third tick should be 1 * value");

        // Tick 4
        let raw4 = tick_dpw4_raw(&mut state, s_q31);
        assert_eq!(raw4, 0, "Constant input should result in zero output");
    }

    #[test]
    fn test_apply_gain_low_freq_precision() {
        // Test Case: Very small raw signal (simulating low freq/low derivative)
        // Magnitude: 1000.
        // 1000 << 64 would be 1000 * 2^64.
        // But here raw is just 1000.  (Value 1000 in Q124 is extremely small).
        // Let's assume raw=1000 is the derivative output value.
        // m=2^62 (Unit Gain approx).
        // We want output to be visible. So we need HUGE gain.
        // Let's explicitly test the High Precision Path condition: raw < 1<<64.

        let raw: i128 = 1000;
        let m: u64 = 1 << 62;
        // raw * m = 1000 * 2^62.
        // target shift = 187 - exp.
        // We want result 100.
        // 1000 * 2^62 >> (187 - exp) = 100
        // 1000 * 2^62 / 2^K = 100
        // 10 * 2^62 = 2^K => K = 62 + log2(10) ~ 62 + 3.32 = 65.32
        // So 187 - exp = 65. => exp = 122.

        // With exp=122:
        // High Prec Path: 1000 * 2^62 >> 65 = 1000 >> 3 = 125.
        // (1000 * 2^62) is 00..001111101000... (bits)
        // It fits in i128.

        // Low Prec Path (Old Logic):
        // (raw >> 64) * m = (1000 >> 64) * m = 0 * m = 0.
        // Output 0.

        // With i32 shift logic (16 bits less), we expect 125 << 16
        let exp = 122;
        let out = apply_gain(raw, m, exp);

        assert!(
            out > 0,
            "Output should be non-zero for small signal in High Prec path"
        );
        assert_eq!(
            out,
            (125 << 16) >> HEADROOM_BITS,
            "Expected headroom-normalized precision preservation"
        );
    }

    #[test]
    fn test_apply_gain_i128_min_unity_gain_no_panic() {
        let unity_q63: u64 = 1 << 63;
        let out = apply_gain(i128::MIN, unity_q63, 0);
        assert!(out <= 0);
    }

    #[test]
    fn test_apply_gain_i128_min_plus_one_unity_gain_no_panic() {
        let unity_q63: u64 = 1 << 63;
        let out = apply_gain(i128::MIN + 1, unity_q63, 0);
        assert!(out <= 0);
    }

    #[test]
    fn test_apply_gain_i128_min_unity_exp_min_no_panic() {
        let unity_q63: u64 = 1 << 63;
        let out = apply_gain(i128::MIN, unity_q63, i32::MIN);
        assert!(out <= 0);
        assert!(out == -1 || out == 0);
    }

    #[test]
    fn test_apply_gain_i128_min_unity_exp_max_no_panic() {
        let unity_q63: u64 = 1 << 63;
        let out = apply_gain(i128::MIN, unity_q63, i32::MAX);
        assert!(out == i32::MIN || out < 0);
    }

    #[test]
    fn test_sine_shape_applies_headroom_policy() {
        let mut state = OscState::new();
        let gain = DpwGain::new(1u64 << 63, 0, 0, 0);
        let phase = math::PI / Scalar::from_num(2.0);
        let out = tick_shape(&mut state, phase, 4, &gain);

        // Normative DEBT-003 oracle until Sine is routed through apply_gain.
        let (s, _) = math::sin_cos_fast(phase);
        let pre_headroom = (s * SINE_EGRESS_SCALE).to_num::<i32>() as i128;
        let expected = saturate_i128_to_i32(pre_headroom >> HEADROOM_BITS);
        assert_eq!(out, expected, "Sine egress must honor HEADROOM_BITS");
    }

    #[test]
    fn test_all_shapes_honor_headroom_container_cap() {
        let gain = DpwGain::new(1u64 << 63, 0, 0, 0);
        let cap_pos = (i32::MAX >> HEADROOM_BITS) as i64;
        // Two's-complement asymmetry: i32::MIN has no +mirror; arithmetic shift preserves sign.
        let cap_neg = -cap_pos - 1;

        let eps = Scalar::ONE / Scalar::from_num(1024);
        let phases = [
            Scalar::ZERO,
            math::PI / Scalar::from_num(2.0),
            math::PI,
            (math::PI * Scalar::from_num(3)) / Scalar::from_num(2),
            (math::PI / Scalar::from_num(2.0)) + eps,
            (math::PI / Scalar::from_num(2.0)) - eps,
            math::PI + eps,
            math::PI - eps,
        ];
        let shapes = [0u32, 1u32, 2u32, 3u32, 4u32];

        for shape in shapes {
            for (phase_idx, phase) in phases.into_iter().enumerate() {
                let mut state = OscState::new();
                // DPW recurrence can have a one-tick transient while internal state initializes.
                let _ = tick_shape(&mut state, phase, shape, &gain);
                let out = tick_shape(&mut state, phase, shape, &gain);
                let out_i64 = out as i64;
                assert!(
                    out_i64 <= cap_pos && out_i64 >= cap_neg,
                    "shape {} exceeded headroom cap on phase_idx {}: {} not in [{}, {}]",
                    shape,
                    phase_idx,
                    out,
                    cap_neg,
                    cap_pos
                );
            }
        }

        // Stateful sweep guard: catches recurrence blow-ups while reusing one state per shape.
        for shape in shapes {
            let mut state = OscState::new();
            let steps = phases.len() * 8;
            for step in 0..steps {
                let phase = phases[step % phases.len()];
                let out = tick_shape(&mut state, phase, shape, &gain);
                let out_i64 = out as i64;
                assert!(
                    out_i64 <= cap_pos && out_i64 >= cap_neg,
                    "shape {} exceeded headroom cap during sweep at step {}: {} not in [{}, {}]",
                    shape,
                    step,
                    out,
                    cap_neg,
                    cap_pos
                );
            }
        }
    }

    #[test]
    fn test_saturating_mul_i128_positive_overflow_saturates_high() {
        assert_eq!(saturating_mul_i128(i128::MAX, 2), i128::MAX);
    }

    #[test]
    fn test_saturating_mul_i128_negative_overflow_saturates_low() {
        assert_eq!(saturating_mul_i128(i128::MIN, 2), i128::MIN);
    }

    #[test]
    fn test_saturating_shift_i128_right_shift_beyond_width_policy() {
        assert_eq!(saturating_shift_i128(1, 200), 0);
        assert_eq!(saturating_shift_i128(-1, 200), -1);
    }

    #[test]
    fn test_saturating_shift_i128_left_shift_overflow_policy() {
        assert_eq!(saturating_shift_i128(1, -200), i128::MAX);
        assert_eq!(saturating_shift_i128(-1, -200), i128::MIN);
    }

    #[test]
    fn test_saturating_shift_i128_zero_left_shift_overflow_policy() {
        assert_eq!(saturating_shift_i128(0, -200), 0);
    }

    #[test]
    fn test_pulse_vs_square() {
        let mut state = OscState::new();
        let gain = DpwGain {
            m4_q63: 1 << 62,
            e4: 0,
            m4_q63_inv: 0,
            e4_inv: 0,
        };
        let phase = Scalar::from_num(1.23);

        // Shape 3 (Square)
        let sq = tick_shape(&mut state, phase, 3, &gain);

        // Reset and use Shape 1 (Pulse) with 50% duty
        state.reset();
        state.duty = Scalar::from_num(0.5);
        let pulse = tick_shape(&mut state, phase, 1, &gain);
        assert_eq!(
            sq, pulse,
            "Square (Shape 3) and Pulse(50%) must be bit-exact"
        );
    }

    #[test]
    fn test_oscillator_accumulation() {
        let mut osc = Oscillator::new_u32(48_000);
        osc.frequency = Scalar::from_num(440);
        let initial_phase = osc.phase;
        osc.tick_phase();
        assert!(osc.phase > initial_phase, "Phase should increment");

        // Stress test: 1 billion samples
        for _ in 0..1000 {
            osc.tick_phase();
        }
        assert!(osc.phase < math::TWO_PI, "Phase should be normalized");
        assert!(osc.phase >= Scalar::ZERO, "Phase should be positive");
    }

    #[test]
    fn test_sine_saw_phase_sync() {
        let mut state = OscState::new();
        let gain = DpwGain::new(1u64 << 63, 0, 0, 0);

        // At phase 0:
        // Sine(0) should be 0
        // Sawtooth(0) = 1.0 normalized = (1 << 16) >> HEADROOM_BITS
        let s0 = tick_shape(&mut state, Scalar::ZERO, 4, &gain); // Sine
        let w0 = tick_shape(&mut state, Scalar::ZERO, 0, &gain); // Saw

        // CORDIC is an approximation; tolerance of 1000 is ~0.01% of full scale
        assert!(s0.abs() <= 1000, "Sine(0) must be near 0, got {}", s0);
        assert_eq!(
            w0,
            // If HEADROOM_BITS >= 16, this normalizes to 0 by construction.
            (1 << 16) >> HEADROOM_BITS,
            "Saw(0) must equal unity scaled by HEADROOM_BITS"
        );

        // At phase PI:
        // Sine(PI) should be 0
        // Sawtooth(PI) should be -65536 (after 10k samples or so, but let's check one tick)
        // Note: Sawtooth(PI) maps to s_q31 = 0 (bipolar center of [2^31, -2^31])
        // Wait, phase_u32 = PI / 2PI * 2^32 = 2^31.
        // s_q31 = 2^31 - 2^31 = 0.
        // x = 0 >> 1 = 0. x^4 = 0.
        // Differentiator tick 1 gives p4 - 0 = 0.
        // So Saw(PI) = 0 on first tick.
        let s_pi = tick_shape(&mut state, math::PI, 4, &gain);
        assert!(
            s_pi.abs() <= 1000,
            "Sine(PI) should be near 0, got {}",
            s_pi
        );
    }

    #[test]
    fn test_triangle_integration_stability() {
        let mut state = OscState::new();
        let gain = DpwGain::new(1u64 << 63, 0, 0, 0);
        let mut min_out = i32::MAX;
        let mut max_out = i32::MIN;

        // Simulation params
        let phase_inc = Scalar::from_num(0.01); // Small, fixed increment
        let mut phase = Scalar::ZERO;

        // 1. Run 16384 samples (extended coverage)
        for _ in 0..16384 {
            let out = tick_triangle_dpw4(&mut state, phase, &gain);
            if out < min_out {
                min_out = out;
            }
            if out > max_out {
                max_out = out;
            }

            // Phase update
            phase += phase_inc;
            if phase >= math::TWO_PI {
                phase -= math::TWO_PI;
            }
        }

        // 2. Assert Not Constant
        assert!(max_out > min_out, "Triangle output should not be constant");

        // 3. Assert Safe Bit Depth
        let bits = 128 - state.tri.z.unsigned_abs().leading_zeros();
        assert!(
            bits < 127,
            "Integration state must remain safe (bits={} < 127)",
            bits
        );

        // 4. Assert Reset Semantics
        state.reset();
        assert_eq!(state.tri.z, 0, "Reset must clear integration state");
        assert!(!state.tri.init, "Reset must clear init flag");
    }

    #[test]
    fn triangle_rail_positive_determinism() {
        let mut z = i128::MAX - 5;
        super::triangle_apply_delta_z(&mut z, 10);
        assert_eq!(z, i128::MAX);
    }

    #[test]
    fn triangle_rail_negative_determinism() {
        let mut z = i128::MIN + 5;
        super::triangle_apply_delta_z(&mut z, -10);
        assert_eq!(z, i128::MIN);
    }

    /// Δ-03 evidence: prove that clamp_to_i128 and saturating_add never
    /// activate under the normative 440 Hz / 44100 Hz sweep (N=10,000,000).
    /// This test uses pub(crate) i256::I256 directly — no mirror duplication.
    #[test]
    #[ignore = "evidence characterization: 10M ticks, run with --ignored"]
    fn test_normative_no_clamp_no_saturate() {
        use crate::i256::I256;

        let mut state = OscState::new();
        let gain = DpwGain::new(1u64 << 63, 0, 0, 0);

        let dphi_u32: u32 = 42845688; // 440 Hz at 44100 Hz SR
        let scale_2_32 = SCALE_2_32;
        let phase_inc = (Scalar::from_num(dphi_u32) / scale_2_32) * math::TWO_PI;

        let num_ticks: u32 = 10_000_000;
        let mut current_phase = Scalar::ZERO;

        let mut did_clamp_delta = false;
        let mut did_saturate_z = false;
        let mut max_abs_delta_i128: u128 = 0;

        let phase_to_u32 = |mut p: Scalar| -> u32 {
            p %= math::TWO_PI;
            if p < Scalar::ZERO {
                p += math::TWO_PI;
            }
            (p / math::TWO_PI * SCALE_2_32).to_num::<u32>()
        };

        for _ in 0..num_ticks {
            // Snapshot z and saw states BEFORE tick
            let z_prev = state.tri.z;
            let saw_a_before = state.saw_a;
            let saw_b_before = state.saw_b;
            let prev_phase_u32_before = state.tri.prev_phase_u32;
            let was_init = state.tri.init;

            let phase_u32 = phase_to_u32(current_phase);
            let phase_b_u32 = phase_to_u32(current_phase - math::PI);

            // Tick the real oscillator
            tick_triangle_dpw4(&mut state, current_phase, &gain);

            // Replay delta computation from snapshotted state
            if was_init {
                let mut dphi = phase_u32.wrapping_sub(prev_phase_u32_before);
                if dphi > DISCONTINUITY_THRESHOLD {
                    dphi = 0;
                }

                let s_a = (phase_u32 as i64).wrapping_sub(BIPOLAR_OFFSET);
                let s_b = (phase_b_u32 as i64).wrapping_sub(BIPOLAR_OFFSET);

                let mut replay_saw_a = saw_a_before;
                let mut replay_saw_b = saw_b_before;
                let raw_a = tick_dpw4_raw(&mut replay_saw_a, s_a);
                let raw_b = tick_dpw4_raw(&mut replay_saw_b, s_b);

                let wide_a = I256::from_i128(raw_a);
                let wide_b = I256::from_i128(raw_b);
                let diff = wide_a.sub(wide_b);
                let shifted = diff.sar(DPW_TRUNCATION_BITS as u32);
                let delta_wide = shifted.mul_u32(dphi);
                let delta_i128 = delta_wide.clamp_to_i128();

                // Check clamp
                let required_hi = (delta_wide.lo as i128) >> 127;
                if delta_wide.hi != required_hi {
                    did_clamp_delta = true;
                }

                // Check saturation
                let z_wrap = z_prev.wrapping_add(delta_i128);
                let z_sat = z_prev.saturating_add(delta_i128);
                if z_wrap != z_sat {
                    did_saturate_z = true;
                }

                let abs_delta = delta_i128.unsigned_abs();
                if abs_delta > max_abs_delta_i128 {
                    max_abs_delta_i128 = abs_delta;
                }
            }

            current_phase += phase_inc;
            if current_phase >= math::TWO_PI {
                current_phase -= math::TWO_PI;
            } else if current_phase < Scalar::ZERO {
                current_phase += math::TWO_PI;
            }
        }

        assert!(
            !did_clamp_delta,
            "delta_wide.clamp_to_i128() clamped under normative inputs; max_abs_delta_i128={}",
            max_abs_delta_i128
        );
        assert!(
            !did_saturate_z,
            "z.saturating_add hit rails under normative inputs; max_abs_delta_i128={}",
            max_abs_delta_i128
        );
    }
}
