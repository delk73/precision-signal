# DPW Gain Specification
**Document revision:** v1.0.0-rc5  
**Applies to:** release 1.2.2 (content unchanged)  
**Status:** Normative (Contract Surface Lock)

## Versioning Terminology

- Document revision labels editorial history for this specification.
- Release versions identify the shipped software release.
- This document defines the gain-field contract; unchanged content remains applicable to release `1.2.2`.

## Scope
Defines the forward and inverse gain fields in `DpwGain`.

## Structure
`DpwGain` is defined as a `pub struct` in `crates/dpw4/src/lib.rs` with the following fields:

* Forward gain mantissa: `m4_q63`
* Forward gain exponent: `e4`
* Inverse gain mantissa: `m4_q63_inv`
* Inverse gain exponent: `e4_inv`

## Invariants

* **Forward fields** (`m4_q63`, `e4`) are used for normal gain application in `apply_gain` and `tick_dpw4` in `crates/dpw4/src/lib.rs`.
* **Inverse fields** (`m4_q63_inv`, `e4_inv`) are cached helpers and may be stale. Correctness must not depend on them unless a path explicitly opts in to inverse scaling.
* **No implied reciprocity**: The code does not guarantee any mathematical relationship between forward and inverse fields unless a caller explicitly enforces one. If a caller chooses to use inverse fields, it must define and validate its own relationship (for example, a fixed-point reciprocal with bounded error).
* **Forbidden Use**: Inverse fields must not be applied to the LE `i32` sample stream used for SHA-256 evidence unless the path explicitly documents inverse scaling as part of its normative definition.

## Call-Path Contract

* `tick_dpw4` applies forward gain at most once per call; callers must not re-apply gain to its output.
* `apply_gain` is the canonical forward-gain helper; any equivalent must be documented as such.

## Terms

* **RAW**: Pre-gain internal sample in the oscillator’s wide fixed-point accumulator domain.
* **Post-gain**: `i32` sample suitable for DP32/WAV emission and SHA-256 evidence.

## Descriptive (as of 53d06b9c28087b5a5e536e8f300eeebb573925f1)

* All DPW-based shapes — Saw (`shape=0`), Pulse/Square (`shape=1,3`), and TriangleDPW4 (`shape=2`) — route through
  `apply_gain`. Triangle calls `apply_gain(state.tri.z, gain.m4_q63, gain.e4)` as the final egress step of
  `tick_triangle_dpw4`.
* Sine (`shape=4`) is the **sole waveform that bypasses `apply_gain`**. It performs a fixed scalar scaling to i32 range
  (`Scalar * 2147483647.0 → i32`), followed by a `HEADROOM_BITS` arithmetic right-shift in i128, and final saturation to
  i32. This path is subject to `DEBT-003` (see code comment); routing Sine through `apply_gain` is deferred until a
  normative Q-format raw representation for the CORDIC output is defined.
* This describes call structure only; no amplitude equivalence or normalization is implied.

## Source (as of 53d06b9c28087b5a5e536e8f300eeebb573925f1)

Optional line anchors for the above statements can be verified in `crates/dpw4/src/lib.rs` at this commit.
