# precision-signal Repository Mapping

This map documents the repository architecture from the Rust implementation paths named below. Links are relative to this document.

## A. RPL0 Binary Artifact Stream

Primary implementation: [`crates/replay-core/src/artifact.rs`](../../crates/replay-core/src/artifact.rs).

The artifact stream magic is the four-byte ASCII tag `RPL0`. The core artifact constants define `FRAME_SIZE = 16`, `FRAME_COUNT = 10_000`, and the v0 fixed artifact size as `160_016` bytes (`16` byte header plus `10_000 * 16` byte frames).

The v0 stream header is `Header0`, encoded little-endian into `HEADER_SIZE = 16` bytes:

| Offset | Size | Field | Type |
| --- | ---: | --- | --- |
| `0x00` | 4 | `magic` | `[u8; 4]` |
| `0x04` | 4 | `version` | `u32` |
| `0x08` | 4 | `frame_count` | `u32` |
| `0x0C` | 4 | `reserved` | `u32` |

The v1 stream header is `Header1`, encoded little-endian into `HEADER1_SIZE = V1_MIN_HEADER_SIZE = 0x98` bytes before the variable schema block:

| Offset | Size | Field | Type |
| --- | ---: | --- | --- |
| `0x00` | 4 | `magic` | `[u8; 4]` |
| `0x04` | 2 | `version` | `u16` |
| `0x06` | 2 | `header_len` | `u16` |
| `0x08` | 4 | `frame_count` | `u32` |
| `0x0C` | 2 | `frame_size` | `u16` |
| `0x0E` | 2 | `flags` | `u16` |
| `0x10` | 4 | `schema_len` | `u32` |
| `0x14` | 32 | `schema_hash` | `[u8; 32]` |
| `0x34` | 32 | `build_hash` | `[u8; 32]` |
| `0x54` | 32 | `config_hash` | `[u8; 32]` |
| `0x74` | 16 | `board_id` | `[u8; 16]` |
| `0x84` | 16 | `clock_profile` | `[u8; 16]` |
| `0x94` | 2 | `capture_boundary` | `u16` |
| `0x96` | 2 | `reserved` | `u16` |

Each replay frame uses the canonical `EventFrame0` layout. The record is exactly `16` bytes:

| Offset | Size | Field | Type |
| --- | ---: | --- | --- |
| `0x00` | 4 | `frame_idx` | `u32` |
| `0x04` | 1 | `irq_id` | `u8` |
| `0x05` | 1 | `flags` | `u8` |
| `0x06` | 2 | `rsv` | `u16` |
| `0x08` | 4 | `timer_delta` | `u32` |
| `0x0C` | 4 | `input_sample` | `i32` |

The host parser in [`crates/replay-host/src/artifact.rs`](../../crates/replay-host/src/artifact.rs) validates both v0 and v1 containers. For v1, replay currently uses strict container validation followed by legacy `EventFrame0` decoding of the frame region.

## B. Embedded Capture Firmware

Primary implementation: [`crates/replay-fw-f446/src/fw.rs`](../../crates/replay-fw-f446/src/fw.rs).

The firmware target is the STM32F446 PAC crate path `stm32f4::stm32f446`. The capture constants are:

| Name | Value |
| --- | --- |
| `FRAME_COUNT` | `10_000` |
| `IRQ_ID_TIM2` | `0x02` |
| `TIMER_DELTA_NOMINAL` | `1_000` |
| `CAPTURE_BOUNDARY_ISR` | `0` |
| `BOARD_ID` | `NUCLEO-F446RE\0\0\0` |
| `CLOCK_PROFILE` | `reset-16mhz-apb1` |

`fw_main` takes the device peripherals, configures GPIOA PA2 as `USART2_TX` on AF7, initializes USART2 for transmit at reset-clock 115200 baud settings, enables TIM2 on APB1, stores TIM2 and USART2 in interrupt-safe mutex slots, and calls `init_tim2_1khz`.

`init_tim2_1khz` configures TIM2 from the reset 16 MHz timer clock with `PSC = 15` and `ARR = 999`, producing a 1,000 Hz update interrupt. The firmware unmasks the TIM2 interrupt, waits with `wfi` until capture is complete, disables interrupts, masks TIM2, stops the timer, and then dumps the artifact.

`tim2_isr` is the deterministic capture loop. It clears the TIM2 update flag, reads the current write index, samples the selected signal model, advances the model state, writes the sample into the local `SAMPLES: [i32; 10_000]` buffer, increments `WRITE_IDX`, and sets `CAPTURE_DONE` after the final frame.

`dump_artifact` emits a v1 RPL0 payload over USART2. It writes a `Header1` with `version = 1`, `header_len = 0x98`, `frame_count = 10_000`, `frame_size = 16`, zero flags, schema metadata from [`crates/replay-fw-f446/src/artifact_metadata.rs`](../../crates/replay-fw-f446/src/artifact_metadata.rs), and the board/clock metadata above. It then writes `RPL0_SCHEMA` followed by 10,000 `EventFrame0` records where `frame_idx` is the sample index, `irq_id = 0x02`, `flags = 0`, `rsv = 0`, `timer_delta = 1_000`, and `input_sample` is the captured sample.

## C. Host Replay and Consistency Engine

Primary implementation: [`crates/replay-host/src/replay.rs`](../../crates/replay-host/src/replay.rs).

The replay state is `SutState0`:

| Field | Type |
| --- | --- |
| `timer_sum` | `u64` |
| `sample_fold` | `u32` |
| `irq_count` | `u32` |

`diff_artifacts0` parses both input byte slices through `parse_replay_frames_legacy0`, computes per-frame replay hashes for each artifact, and returns the first differing frame index.

The deterministic step loop is `replay_hashes0`: it starts from `SutState0::default()`, applies `step0` to each `EventFrame0`, and pushes `hash_state0` after every step. `step0` folds `timer_delta` into `timer_sum`, rotates and XORs `sample_fold` with `input_sample`, `frame_idx`, `flags`, and `rsv`, and increments `irq_count`. `hash_state0` then applies an integer-only SplitMix64 finalizer to the folded state.

`first_divergence0` returns `None` for identical hash streams. If a hash differs, it returns that zero-based frame index. If the common prefix is identical but lengths differ, it returns `min(a.len(), b.len())`.

## D. Operator Surface: `precision` CLI

Primary implementation: [`crates/dpw4/src/bin/precision/mod.rs`](../../crates/dpw4/src/bin/precision/mod.rs). Shared result block implementation: [`crates/dpw4/src/bin/common/mod.rs`](../../crates/dpw4/src/bin/common/mod.rs).

The command enum exposes four authoritative subcommands:

| Subcommand | Inputs | Implementation path |
| --- | --- | --- |
| `record` | `TARGET --mode <runtime_mode\|mock\|none>` | `run_record` |
| `replay` | `TARGET --mode <runtime_mode\|mock\|none>` | `run_replay` |
| `diff` | `TARGET_A TARGET_B --mode <runtime_mode\|mock\|none>` | `run_diff` |
| `envelope` | `TARGET --mode <runtime_mode\|mock\|none>` | `run_envelope` |

`record` accepts three source classes in `acquire_record_capture`: targets beginning with `fixture://`, CSV files with header `index,interval_us`, and serial targets captured through [`scripts/csv_capture.py`](../../scripts/csv_capture.py). CSV and fixture captures must produce `INTERVAL_ROW_COUNT = 138` interval rows. The intervals are converted into semantic trace nodes by `synthesize_semantic_trace`, which runs the DPW4 oscillator path with `Oscillator::new_u32(48_000)`, `DpwGain::new(1u64 << 63, 0, 0, 0)`, and shape `3`.

`record` also builds a transient v1 RPL0 payload with `TRANSIENT_FRAME_COUNT = 10_000`, `TRANSIENT_FRAME_SIZE = 16`, and `TRANSIENT_HEADER_SIZE = 0x98`. The first 138 frames carry the interval values as `input_sample`; remaining frames are zero-padded. The transient frame fields use `irq_id = 0x02`, `flags = 0`, `rsv = 0`, and `timer_delta = 1000`.

`replay` loads a published artifact directory containing `result.txt`, `trace.json`, and `meta.json`, validates the authoritative shape, synthesizes a replay trace from `signal_inputs`, compares captured and replay traces, and reports the first semantic divergence.

`diff` loads two published artifact directories, compares `artifact.signal_inputs` plus captured trace nodes, and emits a comparison-only artifact. `envelope` replays an artifact, extracts `dpw4.square`, converts it to cumulative `envelope.min` and `envelope.max` nodes, and compares captured and replay envelopes.

On success or failure, `ResultBlock::to_bytes` emits exactly seven newline-terminated lines to stdout and to the published `result.txt`:

```text
RESULT: <PASS|FAIL>
COMMAND: <record|replay|diff|envelope>
TARGET: <target>
MODE: <runtime_mode|mock|none>
EQUIVALENCE: <exact|diverged>
FIRST_DIVERGENCE: <none|step=N node=ID cause=CAUSE>
ARTIFACT: artifacts/<run_id>
```

The published artifact directory also contains `trace.json` and `meta.json`. `ResultBlock::canonicalized` derives `RESULT` from `EQUIVALENCE` and `FIRST_DIVERGENCE`: `PASS` requires `equivalence == "exact"` and `first_divergence == "none"`; all other cases canonicalize to `FAIL`.

## Support Math Workspace

The deterministic signal path is anchored by three workspace crates:

| Crate | Implementation | Role |
| --- | --- | --- |
| `dpw4` | [`crates/dpw4/src/lib.rs`](../../crates/dpw4/src/lib.rs) | Core DPW4 oscillator and reference signal processing kernel. It defines `Dpw4State`, `IntegrationState`, `DpwGain`, `Oscillator`, `tick_dpw4`, `tick_shape`, checksum utilities, and the `precision` CLI entry point gated by the `cli` feature in [`crates/dpw4/Cargo.toml`](../../crates/dpw4/Cargo.toml). |
| `geom-signal` | [`crates/geom-signal/src/lib.rs`](../../crates/geom-signal/src/lib.rs), [`crates/geom-signal/src/math.rs`](../../crates/geom-signal/src/math.rs) | Fixed-point signal math. It exports `Scalar = fixed::types::I64F64`, deterministic `sin_cos`, `sin_cos_fast`, `sqrt`, and algebraic atan helpers. |
| `geom-spatial` | [`crates/geom-spatial/src/lib.rs`](../../crates/geom-spatial/src/lib.rs) | Fixed-point spatial vector math over `geom_signal::Scalar`, including `Vector3`, checked/saturating magnitude, and distance calculations. |

These crates constrain the signal and geometry path to fixed-point arithmetic at the repository boundary where replay parity matters. Floating-point ingestion exists only behind explicit conversion points such as `float-ingest` constructors or CLI interval-to-frequency conversion; the reference state, replay folding, artifact frame layout, and geometry math use integer or fixed-point representations to avoid cross-platform drift.
