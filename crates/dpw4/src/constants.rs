//! Domain Constants for Precision-DPW
//!
//! This module centralizes the magic numbers and architectural invariants
//! required for the bit-exact DPW4 reference implementation.

use crate::Scalar;

// =============================================================================
// WAVEFORM ARCHITECTURE
// =============================================================================

/// Scale factor for phase conversion (2^32).
/// SCALE_2_32 = 2^32 in real units; encoded as (2^32)<<64 = 1<<96 in Q64.64.
pub const SCALE_2_32: Scalar = Scalar::from_bits(0x00000001_00000000_00000000_00000000);

/// Bipolar Phase Offset (2^31).
/// Used to map [0, 2^32) phase space to [-2^31, 2^31) bipolar space.
pub const BIPOLAR_OFFSET: i64 = 1_i64 << 31;

/// 4th-Order DPW Normalization Factor (1 / 4!).
/// Users should multiply their amplitude by this factor to normalize gain.
pub const DPW4_GAIN_FACTOR: Scalar = Scalar::from_bits(0x0000000000000000_0aaaaaaaaaaaaaaabi128); // 1/24

/// Sine egress scale in Q31 units before container headroom.
///
/// Calibrated to quick-validate peak parity target (Δ-09B) against DPW saw/pulse in master sweep.
/// Applied only in the sine egress path (shape=4), then shifted by `HEADROOM_BITS`.
pub const SINE_EGRESS_SCALE_Q31: i32 = 284_692_382;
/// I64F64 representation of `SINE_EGRESS_SCALE_Q31`.
pub const SINE_EGRESS_SCALE: Scalar = Scalar::from_bits((SINE_EGRESS_SCALE_Q31 as i128) << 64);

// =============================================================================
// SIGNAL INTEGRITY & SAFETY
// =============================================================================

/// Discontinuity Threshold (1/4 cycle).
/// If phase dphi exceeds this, we assume a hard-sync or discontinuous jump.
pub const DISCONTINUITY_THRESHOLD: u32 = 0x4000_0000;

/// Bit-depth for DPW Trace Truncation.
/// Discards lower bits before integration to establish the noise floor.
pub const DPW_TRUNCATION_BITS: i32 = 32;

/// Safe Bit-length for Integration Accumulator.
/// Provides a 2-bit headroom before i128 overflow.
pub const INTEGRATOR_SAFE_BITS: u32 = 126;

/// Threshold for legacy 64-bit shift safety warnings.
pub const LEGACY_OVERFLOW_BITS: u32 = 63;

// =============================================================================
// GAIN STAGE CONFIGURATION
// =============================================================================

/// Threshold for switching between precision paths in the gain stage.
/// 1 << 64 ensures the product fits in i128 without pre-shifting.
pub const GAIN_PRECISION_THRESHOLD: i128 = 1 << 64;
/// Unsigned mirror of `GAIN_PRECISION_THRESHOLD` for `unsigned_abs` comparisons.
pub const GAIN_PRECISION_THRESHOLD_U: u128 = 1u128 << 64;

/// Q-Format Total Bits for High Precision Path (Q124 + Q63).
pub const Q_HIGH_PRECISION: i32 = 187;

/// Q-Format Total Bits for High Amplitude Path (Q60 + Q63).
pub const Q_HIGH_AMPLITUDE: i32 = 123;

/// Output headroom control (manual right shift before egress).
///
/// v1.0 baseline: 1 bit => 50% primary amplitude, 50% free headroom for
/// superposed reflections/echoes in the same fixed-point container.
pub const HEADROOM_BITS: u32 = 1;
// Safety contract: i128 right-shift count must remain within [0, 127].
// HEADROOM_BITS is a compile-time policy constant and must never be runtime/user controlled.
const _: () = assert!(HEADROOM_BITS <= 127);

// =============================================================================
// TRANSPORT & FORENSIC CONSTANTS
// =============================================================================

/// Expected Magic Identifier for DP32 files.
pub const DP32_MAGIC: &[u8; 4] = b"DP32";

/// Current Protocol Version.
pub const PROTOCOL_VERSION: u32 = 1;

/// Fixed Bit Depth for Reference Standard.
pub const BIT_DEPTH_32: u32 = 32;

/// Fixed Header Size (64 bytes).
pub const HEADER_SIZE: usize = 64;

/// Header Padding Size (reduced from 36 to 32 for checksum field).
pub const HEADER_PAD_SIZE: usize = 32;

/// Offset of the pad field in `SignalFrameHeader`.
pub const HEADER_PAD_OFFSET: usize = 24;

/// Offset of the reserved field in `SignalFrameHeader`.
pub const HEADER_RESERVED_OFFSET: usize = 56;

/// Size of the reserved field in `SignalFrameHeader`.
pub const HEADER_RESERVED_SIZE: usize = 4;

/// Offset of the checksum field in `SignalFrameHeader`.
pub const HEADER_CHECKSUM_OFFSET: usize = 60;

/// Size of the checksum field in `SignalFrameHeader`.
pub const HEADER_CHECKSUM_SIZE: usize = 4;

/// Bipolar Center for i32 (0x8000_0000).
pub const I32_BIPOLAR_CENTER: u32 = 0x8000_0000;

/// 90-degree Phase Offset in u32 space (1/4 cycle).
pub const PHASE_90_DEG: u32 = 0x4000_0000;
