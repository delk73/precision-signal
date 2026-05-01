# Header Layout Addendum (DP32)
**Document revision:** v1.0.0-rc5  
**Applies to:** release 1.7.0 (content unchanged)  
**Status:** Normative (Contract Surface Lock)

## Versioning Terminology

- Document revision labels editorial history for this specification.
- Release versions identify the shipped software release.
- The `version` field described below is the artifact/header contract version, not the software release version.
- Unchanged content remains applicable to release `1.7.0`.

## Scope
This addendum clarifies the canonical layout and alignment rules for `SignalFrameHeader`.

## Normative Layout
`SignalFrameHeader` is defined as `#[repr(C, align(64))]` in `crates/dpw4/src/lib.rs`.

* Alignment to 64 bytes is **normative**.
* The struct is required to occupy **exactly 64 bytes**.
* **Requirement**: `size_of::<SignalFrameHeader>() == 64` (enforced by `test_header_offsets_are_canonical`).
* **Evolution**: Any new fields added must reduce the `pad` array accordingly to maintain the 64-byte invariant.

## Authoritative Byte-Offset Table

| Offset | Size | Field | Constant |
|---|---|---|---|
| 0 | 4 | `magic: [u8; 4]` | — |
| 4 | 4 | `version: u32` | `PROTOCOL_VERSION` |
| 8 | 8 | `sequence: u64` | — |
| 16 | 4 | `sample_rate: u32` | — |
| 20 | 4 | `bit_depth: u32` | `BIT_DEPTH_32` |
| 24 | 32 | `pad: [u8; 32]` | `HEADER_PAD_OFFSET = 24`, `HEADER_PAD_SIZE = 32` |
| 56 | 4 | `reserved: [u8; 4]` | `HEADER_RESERVED_OFFSET = 56`, `HEADER_RESERVED_SIZE = 4` |
| 60 | 4 | `checksum: u32` | `HEADER_CHECKSUM_OFFSET = 60`, `HEADER_CHECKSUM_SIZE = 4` |
| **Total** | **64** | | `HEADER_SIZE = 64` |

All constants are defined in `crates/dpw4/src/constants.rs`.

## Field Map
Defined fields in `crates/dpw4/src/lib.rs`:

* `magic: [u8; 4]`
* `version: u32`
* `sequence: u64`
* `sample_rate: u32`
* `bit_depth: u32`
* `pad: [u8; 32]`
* `reserved: [u8; 4]` — must be zero on wire; enforced by `to_bytes()`
* `checksum: u32` (LE) — Fletcher-32 of bytes 0–59

## Arithmetic
`4 + 4 + 8 + 4 + 4 + 32 + 4 + 4 = 64 bytes` — exactly fills the 64-byte aligned struct with no compiler padding.

## Padding vs Alignment
`#[repr(C, align(64))]` enforces 64-byte alignment and rounds the struct size up to a multiple of 64. Because the sum of all fields is exactly 64, no implicit compiler padding is added.

## Endianness and Packing

* All integer fields are serialized in little-endian order via `to_bytes()`.
* `magic` bytes are serialized verbatim (no endianness conversion).
* `#[repr(C)]` forbids field reordering; no `packed` layout is used.
* `reserved` bytes are always written as `[0; 4]` by `to_bytes()`, regardless of the struct field value.

## Resolution (Δ-05)
**DIV-07 CLOSED.** Earlier versions of this document stated `pad: [u8; 36]` (no `reserved`, no `checksum`) based on a pre-checksum layout. The code was updated to add `reserved[4]` and `checksum[4]`, splitting the old pad region. This document and `VERIFICATION_GUIDE.md §7.1` now match the emitted bytes exactly. Tests `test_header_offsets_are_canonical` and `test_header_reserved_gap_is_zeroed` are the normative byte-layout assertions.
