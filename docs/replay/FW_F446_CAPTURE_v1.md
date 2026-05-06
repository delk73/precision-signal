# F446 Capture v1 — RPL0 Capture Contract

**Document revision:** 1.7.0  
**Status:** NORMATIVE  
**Applies to:** `crates/replay-fw-f446`

## Versioning Terminology

- Document revision labels editorial history for this capture contract.
- Release versions identify the shipped software release.
- `v1` in this document name and `version = 1` in emitted headers identify the RPL container version, not the software release version.

---

## Scope

This document is the active operator capture contract for `crates/replay-fw-f446` — the
bare-metal RPL0-emitting firmware on STM32F446RE. It covers the TIM2 update-interrupt
capture path that produces a hardware-backed RPL file over USART2.

The STM32 self-stimulus timing characterization fixture is a separate crate
(`crates/replay-fw-f446-timing`) governed by
[docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md](INTERVAL_CAPTURE_CONTRACT_v1.md).

Historical release-record routing lives in
[docs/verification/releases/index.md](../verification/releases/index.md).

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

The normative RPL format definition remains [docs/spec/rpl0_format_contract.md](../spec/rpl0_format_contract.md).

---

## Schema Block

The retained historical firmware path emitted a fixed embedded schema block immediately after the header.

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

The retained historical firmware path emitted `capture_boundary = 0`, which maps to the ISR boundary for that capture record.

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
RPL0 v1 artifacts. No nondeterministic fields were permitted in that retained capture path.

---

## Verification Expectations

Active operator path:

```bash
make rpl0-replay-check
make rpl0-replay-repeat-auto REPLAY_REPEAT_RUNS=3
```

Host tooling expectations:

- `scripts/artifact_tool.py capture` reads the v1 container from UART
- `scripts/artifact_tool.py verify` enforces strict structure and signal-model checks
- `scripts/artifact_tool.py compare` requires byte-identical match against `artifacts/baseline.bin`
- repeat-capture manifests are written as `replay_manifest_v1.txt` with `contract_version=rpl0_capture_v1`
