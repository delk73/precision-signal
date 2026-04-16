# Replay Wire Format v0

> **Status: HISTORICAL WIRE FORMAT**
> This document is retained for legacy `RPL0` v0 inspection only. It does not
> define the active STM32 capture authority. Use
> [docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md](INTERVAL_CAPTURE_CONTRACT_v1.md)
> for the live capture contract and
> [docs/verification/releases/index.md](../verification/releases/index.md) for
> historical verification routing.

This document defines the normative wire format for the RPL0 artifact.
It is the referenced wire-format specification for the capture contract in [docs/replay/FW_F446_CAPTURE_v0.md](FW_F446_CAPTURE_v0.md).
For the stabilized artifact contract with schema extension block and fixed 16-byte opaque frames, see [docs/spec/rpl0_artifact_contract.md](../spec/rpl0_artifact_contract.md).

## Endianness
All integer fields are encoded as little-endian bytes.

## Header0 Layout (16 bytes)

- `magic: [u8; 4]` at offset `0`
  - Required value: ASCII `RPL0` (`0x52 0x50 0x4c 0x30`)
- `version: u32` at offset `4`
  - Required value: `0`
- `frame_count: u32` at offset `8`
- `reserved: u32` at offset `12`

Constants in code:
- `MAGIC = b"RPL0"`
- `VERSION0 = 0`
- `HEADER0_SIZE = 16`

## EventFrame0 Layout (16 bytes)

- `frame_idx: u32` at offset `0`
- `irq_id: u8` at offset `4`
- `flags: u8` at offset `5`
- `rsv: u16` at offset `6`
- `timer_delta: u32` at offset `8`
- `input_sample: i32` at offset `12`

Constant in code:
- `EVENTFRAME0_SIZE = 16`

## Total Artifact Length Rule
A full artifact must satisfy:

`len == HEADER0_SIZE + frame_count * EVENTFRAME0_SIZE`

## Parser Validation Rules

`parse_header0(bytes)`:
- Fails with `InputTooShort` if `bytes.len() < HEADER0_SIZE`
- Fails with `BadMagic` if `magic != RPL0`
- Fails with `BadVersion` if `version != 0`
- Otherwise returns decoded `Header0`

`parse_frames0(bytes, header)`:
- Computes expected length from `header.frame_count`
- Fails with `FrameCountTooLarge` on conversion/overflow
- Fails with `LengthMismatch` unless `bytes.len() == expected_len`
- On success decodes all event frames field-by-field from LE bytes

## Important Note on Layout
The wire format is defined by explicit little-endian byte encoding/decoding of fields.
It is not defined by Rust struct memory layout.
