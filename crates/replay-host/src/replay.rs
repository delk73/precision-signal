use replay_core::artifact::EventFrame0;

use crate::artifact::{parse_replay_frames_legacy0, ParseError};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SutState0 {
    pub timer_sum: u64,
    pub sample_fold: u32,
    pub irq_count: u32,
}

pub fn step0(state: SutState0, frame: &EventFrame0) -> SutState0 {
    let timer_sum = state.timer_sum.wrapping_add(u64::from(frame.timer_delta));
    let sample_fold = state.sample_fold.rotate_left((frame.irq_id & 31) as u32)
        ^ (frame.input_sample as u32)
        ^ frame.frame_idx
        ^ ((u32::from(frame.flags)) << 24)
        ^ ((u32::from(frame.rsv)) << 8);
    let irq_count = state.irq_count.wrapping_add(1);

    SutState0 {
        timer_sum,
        sample_fold,
        irq_count,
    }
}

pub fn hash_state0(state: &SutState0) -> u64 {
    let mut x = state.timer_sum;
    x ^= (u64::from(state.sample_fold)) << 1;
    x ^= (u64::from(state.irq_count)) << 33;

    // SplitMix64 finalizer for stable integer-only diffusion.
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

pub fn replay_hashes0(frames: &[EventFrame0]) -> Vec<u64> {
    let mut out = Vec::with_capacity(frames.len());
    let mut state = SutState0::default();

    for frame in frames {
        state = step0(state, frame);
        out.push(hash_state0(&state));
    }

    out
}

/// Returns first differing index. If common prefix is equal but lengths differ,
/// divergence is reported at `min(a.len(), b.len())`.
pub fn first_divergence0(a: &[u64], b: &[u64]) -> Option<usize> {
    let min_len = a.len().min(b.len());

    for idx in 0..min_len {
        if a[idx] != b[idx] {
            return Some(idx);
        }
    }

    if a.len() == b.len() {
        None
    } else {
        Some(min_len)
    }
}

/// Parses both artifacts independently. If either artifact is malformed,
/// returns the first encountered ParseError.
/// For v1 artifacts, replay is currently defined by the legacy 16-byte
/// `EventFrame0` interpretation after strict container validation.
/// If both parse successfully, returns:
///   - Ok(None) if replay hash streams are identical,
///   - Ok(Some(idx)) where `idx` is the first differing frame index.
/// If lengths differ but common prefix hashes are equal,
/// divergence is reported at `min_len`.
pub fn diff_artifacts0(a_bytes: &[u8], b_bytes: &[u8]) -> Result<Option<usize>, ParseError> {
    let a_frames = parse_replay_frames_legacy0(a_bytes)?;
    let b_frames = parse_replay_frames_legacy0(b_bytes)?;

    let a_hashes = replay_hashes0(&a_frames);
    let b_hashes = replay_hashes0(&b_frames);

    Ok(first_divergence0(&a_hashes, &b_hashes))
}
