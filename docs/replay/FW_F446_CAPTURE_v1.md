# F446 Capture v1 — Normative Capture Contract

**Document revision:** 1.2.2  
**Applies to:** release 1.2.2 (content unchanged)

## Versioning Terminology

- Document revision labels editorial history for this capture contract.
- Release versions identify the shipped software release.
- `v1` in this document name and `version = 1` in emitted headers identify the artifact/header version, not the software release version.

> **Status: NORMATIVE**
> This document defines the current NUCLEO-F446RE capture contract for `replay-fw-f446`.
> Firmware capture emits RPL0 `version = 1` container artifacts using `[HEADER][SCHEMA BLOCK][FRAME DATA]`.
> Replay frame payload semantics remain the legacy 16-byte `EventFrame0` interpretation.

---

## Scope

This contract defines the active operator-facing firmware capture path for the
NUCLEO-F446RE board workflow, UART artifact egress, and host verification flow.

---

## Signal Contract

The canonical replay signal remains `phase8`:

```
sample = frame_idx & 0xFF
```

Mechanism: 32-bit phase accumulator with `STEP = 0x0100_0000`, read-then-advance ordering, output `(phase >> 24) as i32`.

---

## Artifact Layout

Artifacts are emitted as:

```
[HEADER][SCHEMA BLOCK][FRAME DATA]
```

Current emitted header fields:

- `magic = "RPL0"`
- `version = 1`
- `header_len = 152` (`0x98`, which is also the minimum v1 header size)
- `frame_count = 10000`
- `frame_size = 16`
- `flags = 0`
- `schema_len = 91`
- `schema_hash = SHA256(schema_block)`
- `build_hash`: deterministic producer identity bytes
- `config_hash`: deterministic capture-config identity bytes
- `board_id`: deterministic board identity bytes
- `clock_profile`: deterministic clock identity bytes
- `capture_boundary = 0`
- `reserved = 0`

The normative artifact-format definition remains [docs/spec/rpl0_artifact_contract.md](../spec/rpl0_artifact_contract.md).

---

## Schema Block

The current firmware emits a fixed embedded schema block immediately after the header.

- Schema bytes are deterministic and have no runtime variability.
- `schema_hash` is the SHA-256 digest of exactly the emitted `schema_len` bytes.
- Host verification MUST reject artifacts whose schema bytes do not match the stored `schema_hash`.

Current schema bytes describe the legacy `EventFrame0` payload fields; they do not change replay semantics.

---

## Frame Contract

- `frame_count = 10000`
- `frame_size = 16`
- Frame payload semantics remain legacy `EventFrame0`:
  - `frame_idx`: `0..9999`
  - `irq_id = 0x02` (TIM2)
  - `flags = 0`
  - `rsv = 0`
  - `timer_delta = 1000`
  - `input_sample = (phase >> 24) as i32`

No schema-aware replay behavior is introduced by this contract.

---

## Capture Boundary

Current firmware emits `capture_boundary = 0`, which maps to the ISR boundary for the active firmware path.

---

## Canonical Reference Artifact

```
path:         artifacts/baseline.bin
sha256:       f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765
schema_hash:  8c6e82b4f9c80de029775d26da900a655686fd93038013ca759155ff02a68721
signal_model: phase8
```

Metadata is retained in `artifacts/baseline.json` and `artifacts/baseline.sha256`.

---

## Determinism Contract

Identical firmware binary and capture configuration MUST produce byte-identical
RPL0 v1 artifacts. No nondeterministic fields are permitted in the active capture path.

---

## Operator Verification Expectations

Active operator path:

```bash
make replay-check
make replay-repeat-auto REPLAY_REPEAT_RUNS=3
make firmware-release-check
```

Host tooling expectations for the active path:

- `scripts/artifact_tool.py capture` reads the v1 container from UART
- `scripts/artifact_tool.py verify` enforces strict structure and signal-model checks
- `scripts/artifact_tool.py compare` requires byte-identical match against `artifacts/baseline.bin`
- repeat-capture manifests are written as `replay_manifest_v1.txt` with `contract_version=rpl0_capture_v1`

v0 artifacts remain supported for historical inspection only; they are not the active capture/release path.
