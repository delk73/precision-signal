# Host Replay (RPL0) Semantics (WIP / Experimental)

This document describes current `replay-host` replay/diff semantics.
All logic is integer-only and deterministic.
Current support is limited to:

- RPL0 v0 parsing and replay
- RPL0 `version = 1` container parsing
- RPL0 format version 1 replay only under the legacy 16-byte `EventFrame0` interpretation

This document does not claim generic schema-aware RPL0 format version 1 replay semantics.

## State: `SutState0`

Fields:
- `timer_sum: u64`
- `sample_fold: u32`
- `irq_count: u32`

Note: in current implementation `irq_count` increments once per processed frame.

## Step Function: `step0(state, frame)`

Given `frame: EventFrame0`, update state as:

- `timer_sum = state.timer_sum.wrapping_add(frame.timer_delta as u64)`
- `sample_fold = state.sample_fold.rotate_left((frame.irq_id & 31) as u32)`
  then XOR with:
  - `frame.input_sample as u32`
  - `frame.frame_idx`
  - `(frame.flags as u32) << 24`
  - `(frame.rsv as u32) << 8`
- `irq_count = state.irq_count.wrapping_add(1)`

## State Hash: `hash_state0`

`hash_state0(state) -> u64` computes a deterministic, non-cryptographic digest:

1. Pack/mix fields into `x`:
- `x = timer_sum`
- `x ^= (sample_fold as u64) << 1`
- `x ^= (irq_count as u64) << 33`

2. Apply SplitMix64-style finalizer:
- `x ^= x >> 30`
- `x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9)`
- `x ^= x >> 27`
- `x = x.wrapping_mul(0x94d0_49bb_1331_11eb)`
- `x ^= x >> 31`

This hash is deterministic and intended for replay comparison, not cryptographic security.

## Replay Hash Stream: `replay_hashes0`

`replay_hashes0(frames)`:
- Starts from `SutState0::default()`
- Applies `step0` for each frame in order
- Emits one hash per frame after each step
- Output length equals input frame length

## First Divergence: `first_divergence0`

`first_divergence0(a, b)` returns:
- `Some(i)` for first index `i` where `a[i] != b[i]`
- `None` if lengths and all values are equal
- `Some(min_len)` if common prefix matches but lengths differ

## Artifact Diff Flow: `diff_artifacts0`

`diff_artifacts0(a_bytes, b_bytes)`:
- Parses both artifacts independently via strict container validation, then
  decodes replay frames under the legacy `EventFrame0` interpretation
- Returns the first encountered `ParseError` if parsing fails
- If both parse, replays both frame streams and compares hashes
- Returns:
  - `Ok(None)` if replay hash streams match exactly
  - `Ok(Some(idx))` where `idx` is first divergence index
  - If lengths differ but prefix hashes match, divergence is at `min_len`
