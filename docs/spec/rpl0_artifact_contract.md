# RPL0 Artifact Contract (Normative)

**Document revision:** 1.4.0  
**Applies to:** release 1.7.0 (content unchanged)

This document defines the normative binary artifact format for the RPL0 `version = 1` header path.
It is sufficient to implement a compliant parser and deterministic artifact hasher without guessing.

## Versioning Terminology

- `RPL0` identifies the artifact format.
- `vX.Y.Z` identifies the software release.
- The `version` field in the header selects the parsing path and is part of the artifact format.
- The term "current release surface" refers to the set of capabilities classified as Release in [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md).
- Document revisions are editorial and do not version the artifact format.
- Software release versions do not imply artifact format changes.

## Scope and non-goals

This contract defines artifact-level structure only:

- Header encoding and validation
- Schema block placement and integrity
- Frame data placement and framing
- Canonical artifact hash definition
- Forward-compatibility behavior via `header_len`

This contract does not define frame payload semantics. Frame payload bytes are opaque to the artifact format.

This contract does not change firmware capture logic.

## Hard invariants

1. `FRAME_SIZE` is fixed at `16` bytes.
2. Frame payload bytes are opaque to this artifact contract.
3. Schema interpretation is artifact-level only.
4. Artifact identity hash is deterministic: `SHA256(header + schema_block + frame_data)`.
5. Unknown header fields are skippable using `header_len`.
6. Backward-incompatible artifact layout changes require a header `version` increment.
7. Firmware capture logic is out of scope and unchanged by this contract.

## Byte order

All multi-byte integer fields are little-endian.
All integer fields use unsigned two's-complement binary encoding (`u16`, `u32` as specified in this document).

## Version Dispatch Semantics (Normative)

This dispatch rule is intentional, frozen, and part of the compatibility boundary.

- RPL0 v0 uses the legacy fixed header layout.
- A compliant parser MUST first read a 32-bit little-endian value at offset `0x04`.
- If that `u32` value equals `0`, the artifact MUST be interpreted as v0.
- Otherwise, the artifact MUST be interpreted using the extended-header parsing path.
- In the v1 parsing path, the 16-bit little-endian value at offset `0x04` MUST equal `1`.
- Any other value is invalid and MUST cause parse failure.

Changing this dispatch rule requires a format version break and corresponding compatibility update.

## Artifact layout

Artifacts are encoded as:

`[HEADER][SCHEMA BLOCK][FRAME DATA]`

- `HEADER`: fixed normative fields below.
- `SCHEMA BLOCK`: starts at offset `header_len`, length `schema_len`.
- `FRAME DATA`: immediately follows schema block, length `frame_count * frame_size`.

## Header layout (v1)

For `version = 1`, the minimum header size is `0x98` (`152`) bytes.

| Offset | Size | Field | Type | Notes |
|---|---:|---|---|---|
| `0x00` | 4 | `magic` | `[u8;4]` | MUST be ASCII `RPL0` |
| `0x04` | 2 | `version` | `u16` | MUST be `1` for this contract |
| `0x06` | 2 | `header_len` | `u16` | Full header size in bytes |
| `0x08` | 4 | `frame_count` | `u32` | Number of frames in frame data |
| `0x0C` | 2 | `frame_size` | `u16` | MUST be `16` for v1 |
| `0x0E` | 2 | `flags` | `u16` | Artifact-level flags; opaque to framing |
| `0x10` | 4 | `schema_len` | `u32` | Schema block length in bytes |
| `0x14` | 32 | `schema_hash` | `[u8;32]` | `SHA256(schema_block)` |
| `0x34` | 32 | `build_hash` | `[u8;32]` | Producer build identity (opaque bytes) |
| `0x54` | 32 | `config_hash` | `[u8;32]` | Capture config identity (opaque bytes) |
| `0x74` | 16 | `board_id` | `[u8;16]` | Board identity (opaque bytes) |
| `0x84` | 16 | `clock_profile` | `[u8;16]` | Clock profile identity (opaque bytes) |
| `0x94` | 2 | `capture_boundary` | `u16` | Capture boundary selector (enumerated, implementation-defined) |
| `0x96` | 2 | `reserved` | `u16` | Reserved |

### `header_len` rule

- For `version = 1`, `header_len` MUST be `>= 0x98`.
- In future compatible revisions, header extension fields MAY be appended after offset `0x98`; readers use `header_len` to skip unknown trailing header bytes.
- A reader MUST reject artifacts where `header_len < 0x98`.
- Current producer implementations set `header_len = 0x98`.

### `capture_boundary` semantics

`capture_boundary` is an implementation-defined enumerated selector. Known values:

- `0`: ISR boundary
- `1`: DMA ingress
- `2`: Peripheral bus ingress

Readers SHOULD preserve/report unknown values without reinterpretation.

## Schema block

- Start offset: `schema_offset = header_len`
- Length: `schema_len`
- End offset: `schema_end = schema_offset + schema_len`

The schema defines interpretation metadata for frame payload bytes (for example channel definitions, signal identifiers, numeric encoding, units, scaling, sensor type, and endian rules). The artifact format does not interpret frame payload itself.

Schema integrity field:

- `schema_hash = SHA256(schema_block)`
- `schema_hash` MUST be computed over exactly `schema_len` bytes beginning at `schema_offset`.

## Frame data block

- Start offset: `frames_offset = header_len + schema_len`
- Length: `frame_count * frame_size`
- End offset: `frames_end = frames_offset + frame_count * frame_size`

For `version = 1`:

- `frame_size` MUST equal `16`.
- `frame_count` MAY be `0`.
- Frame bytes are opaque at artifact-format level.
- Frame boundaries are located mechanically by fixed-width stepping:
  - `frame_n_offset = frames_offset + n * 16`, for `0 <= n < frame_count`.

## Total artifact length rule

Let:

- `expected_len = header_len + schema_len + frame_count * frame_size`

A compliant artifact MUST satisfy:

- `file_len == expected_len`
- Readers MUST compute `expected_len` using integer arithmetic that rejects overflow.

No trailing or missing bytes are allowed.

## Canonical artifact identity hash

The canonical deterministic artifact identity is:

- `artifact_hash = SHA256(canonical artifact bytes: header + schema_block + frame_data)`

Where:

- `header` is exactly the first `header_len` bytes from the artifact.
- `schema_block` is exactly the next `schema_len` bytes.
- `frame_data` is exactly the remaining `frame_count * frame_size` bytes.

### Hash mode vs strict verification (normative tooling behavior)

- Strict verification/compare paths MUST reject trailing bytes (`file_len` must equal computed canonical length).
- Canonical hash mode MAY ignore trailing bytes and hash only the canonical artifact prefix
  (`header + schema_block + frame_data`).
- This behavior difference is intentional:
  - `verify`/`compare` enforce strict structural validity.
  - `hash` computes portable artifact identity from the canonical prefix.

## Invalid artifact conditions (MUST reject)

A reader MUST reject when any of the following holds:

- `magic != "RPL0"`
- `version` unsupported by the reader
- `header_len < 0x98`
- `frame_size != 16` (for `version = 1`)
- `schema_offset + schema_len > file_len`
- `schema_hash != SHA256(schema_block)`
- `file_len != header_len + schema_len + frame_count * frame_size`

## Forward compatibility

- Readers MUST parse `header_len` and use it to locate schema and frame data.
- Readers MUST safely skip unknown trailing header bytes if present in a supported version.
- Backward-incompatible layout changes MUST use a new `version`.

## Reference parser procedure (normative algorithm)

1. Read bytes at offsets `0x00..0x98`; fail if file shorter than `0x98`.
2. Decode little-endian header fields.
3. Validate `magic`, `version`, `header_len`, and `frame_size`.
4. Compute `expected_len` and validate exact file length.
5. Slice `schema_block` and validate `schema_hash`.
6. Slice `frame_data` as `frame_count` contiguous `16`-byte frames.
7. Compute canonical artifact hash over `[header][schema_block][frame_data]`.

This procedure is sufficient to build a compliant parser and deterministic comparison pipeline.
