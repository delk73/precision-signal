# Oscillator API Specification
**Document revision:** v1.0.0-rc5  
**Applies to:** release 1.7.0 (content unchanged)  
**Status:** Normative (Contract Surface Lock)

## Versioning Terminology

- Document revision labels editorial history for this specification.
- Release versions identify the shipped software release.
- This document defines the public oscillator contract; unchanged content remains applicable to release `1.7.0`.

## Scope
This document specifies the public oscillator surface and shape dispatch contract as implemented in `crates/dpw4/src/lib.rs`. Items listed below are `pub` and part of the reference contract for external callers across the v1.x line.

## API Surface
The following items are part of the reference API surface:

* `OscState` in `crates/dpw4/src/lib.rs`
* `Oscillator` in `crates/dpw4/src/lib.rs`
* `tick_shape` in `crates/dpw4/src/lib.rs`
* `SignalShape` in `crates/dpw4/src/lib.rs`
* `signal_pipe` in `crates/dpw4/src/lib.rs`

## Shape ID Table (Normative)
The `tick_shape` shape IDs are **dispatch-stable** and must not change.

* `0` = Sawtooth
* `1` = Pulse
* `2` = Triangle
* `3` = Square
* `4` = Sine

## Stability Note
These numeric IDs are part of the reference dispatch contract and are stable across the v1.x line.

## Invalid IDs (Descriptive as of 53d06b9c28087b5a5e536e8f300eeebb573925f1)
IDs outside `0..=4` return `0` (DC-zero sample, treated as silence).

## Phase and Domain Contract (Normative)
`tick_shape` expects `phase` as `Scalar` radians, reduced modulo `2π`.

## Dispatch Contract (Normative)
* `SignalShape` provides a typed API for signal generation.
* Numeric shape IDs are the dispatch contract used by `tick_shape`.

## Source (as of 53d06b9c28087b5a5e536e8f300eeebb573925f1)

Optional line anchors for the above statements can be verified in `crates/dpw4/src/lib.rs` at this commit.
