# Deterministic Execution Analysis Infrastructure

## Architecture Whitepaper — precision-signal Replay Architecture

---

## Abstract

This document describes the architecture of the current replay-facing system
implemented in the `precision-signal` repository. The system captures execution
artifacts from physical hardware, encodes them in a canonical binary format,
replays those artifacts through deterministic tooling, and performs structured
divergence analysis on the resulting hash streams.

The central architectural contribution is the **artifact boundary**: a
well-defined serialization contract that separates runtime capture from offline
analysis. This boundary enables execution comparison without execution
reproduction. Two executions can be compared by replaying their artifacts
independently; the original runtime conditions need not be recreated.

The system is implemented across bare-metal firmware (STM32F446RE), a Rust
workspace of `no_std` and host-side crates, Python replay tooling, and a
multi-tier verification architecture including formal proofs (Kani), hash-locked
reference outputs, and deterministic build enforcement.

This document is architectural, not a release classifier. Command-surface
classification is routed to [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md); verification authority is
routed to [VERIFICATION_GUIDE.md](../../VERIFICATION_GUIDE.md); released replay operator tooling is
routed to [docs/replay/tooling.md](../replay/tooling.md). Experimental workspace components and
[docs/wip/](../wip/) material are present in the workspace but are not classified as released
unless promoted by [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md).

---

## Terminology

| Term | Definition |
|------|-----------|
| **Artifact** | A canonical binary file encoding a header, optional schema block, and a sequence of execution frames in the RPL0 wire format. Size is determined by header fields. |
| **RPL0** | The versioned binary artifact format. Magic bytes: `RPL0`. Two format versions exist: format version 0 (fixed-layout, legacy) and format version 1 (variable-layout with schema). |
| **Frame** | A 16-byte record capturing one interrupt-driven execution sample. |
| **Replay** | Deterministic reconstruction of execution state from an artifact's frame stream. |
| **State hash** | A 64-bit value computed from accumulated replay state after each frame via the SplitMix64 finalizer. |
| **Hash stream** | The ordered sequence of state hashes produced by replaying an artifact. |
| **Divergence** | The first frame index at which two hash streams differ. |
| **Artifact boundary** | The serialization contract separating runtime capture from offline analysis. |
| **DPW4** | 4th-order Differentiated Polynomial Waveform synthesis; the reference oscillator used to generate deterministic test signals. |
| **Scalar** | 128-bit fixed-point type (`I64F64`) used throughout the math core. |
| **CORDIC** | Coordinate Rotation Digital Computer; iterative trigonometric algorithm used for sine/cosine. |
| **Golden hash** | A SHA-256 hash of deterministic output frozen at a specific version. |
| **Normative** | A contract or document that defines required behavior. |
| **components classified as released** | Components that [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md) classifies as released. |

---

## 1. Problem Statement

Embedded and signal-processing systems produce execution traces that must be
compared across runs, hardware units, firmware revisions, and toolchain
versions. Traditional approaches require either reproducing the original
runtime environment or comparing raw outputs without structured localization of
differences.

Three requirements motivate the architecture:

1. **Deterministic comparison.** Two executions must be comparable without
   reproducing the physical conditions of either execution.

2. **Divergence localization.** When executions differ, the system must identify
   the first frame of divergence, the affected data region, the shape of the
   divergence, and how the divergence evolves over subsequent frames.

3. **Verification without ambiguity.** Determinism claims must be backed by
   formal proofs, hash-locked reference outputs, and reproducible builds. No
   claim may exceed what the verification infrastructure demonstrates.

---

## 2. Architecture Overview

The system implements a four-stage pipeline:

```
capture → artifact → replay → analysis
```

### Pipeline Diagram

```
┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐
│     CAPTURE      │    │     ARTIFACT      │    │      REPLAY      │    │     ANALYSIS     │
│                  │    │                   │    │                  │    │                  │
│  STM32F446RE     │───>│  RPL0 binary      │───>│  State machine   │───>│  Hash stream     │
│  TIM2 ISR        │    │  Header + Frames  │    │  SutState0       │    │  comparison      │
│  Phase accum.    │    │  16B header       │    │  step0() per     │    │  Divergence      │
│  USART2 emit     │    │  16B × N frames   │    │  frame           │    │  localization    │
│                  │    │                   │    │  hash_state0()   │    │  Classification  │
└──────────────────┘    └──────────────────┘    └──────────────────┘    └──────────────────┘
        │                        │                       │                       │
        │                        │                       │                       │
   physical time            artifact boundary        deterministic           structured
   ISR-driven               (wire contract)          state replay            explanation
```

The **artifact boundary** is the architectural invariant. Everything to its left
is runtime-dependent. Everything to its right is deterministic and reproducible
from the artifact alone.

### Artifact Lifecycle

```
          ┌─────────────────────┐
          │       CAPTURE       │  STM32F446RE / TIM2 ISR
          │  (physical, lossy)  │
          └─────────┬───────────┘
                    │ USART2 serial stream
                    ▼
          ┌─────────────────────┐
          │       STORAGE       │  .rpl binary on host
          │  (canonical, inert) │
          └─────────┬───────────┘
                    │ read + parse
                    ▼
          ┌─────────────────────┐
          │       REPLAY        │  step0() × N frames
          │  (deterministic)    │  → hash stream
          └─────────┬───────────┘
                    │ pair-wise comparison
                    ▼
          ┌─────────────────────┐
          │        DIFF         │  first_divergence0()
          │  (frame-level)      │
          └─────────┬───────────┘
                    │ region + shape + evolution
                    ▼
          ┌─────────────────────┐
          │   CLASSIFICATION    │  artifact_diff.py
          │  (structured)       │
          └─────────────────────┘
```

### Architectural Contribution

The artifact boundary enables:

```
execution comparison without execution reproduction
```

Given two artifacts A and B:

1. Replay A through the deterministic state machine. Produce hash stream H_A.
2. Replay B through the deterministic state machine. Produce hash stream H_B.
3. Compare H_A and H_B element-wise.

The comparison requires only the artifacts. The original hardware, timing,
clocking, and environmental conditions are not needed.

---

## 3. System Layers

The system is organized into five layers.

### Layer 1: Fixed-Point Math Core (`geom-signal`)

A `no_std` library providing deterministic arithmetic primitives. Its
architecture-relevant properties are:

- **Scalar type**: `I64F64` (128-bit fixed-point). All oscillator math
  operates on this type, avoiding floating-point nondeterminism.
- **Trigonometry**: 64-iteration CORDIC for `sin`/`cos`; Shafer-Fink `atan`
  with asymptotic clamping. No floating-point operations anywhere in the path.
- **Verification coverage**: Kani proofs cover `sqrt`, `sin_cos`, `atan_shafer`,
  and all four `atan2` quadrants (see Section 9.1).

Evidence:
- `crates/geom-signal/src/lib.rs`: Scalar type alias (line 13)
- `crates/geom-signal/src/math.rs`: `sin_cos_kernel()` (lines 71–103), `sqrt()` (lines 107–122)
- `crates/geom-signal/src/algebraic.rs`: `atan_shafer()` (lines 89+), `atan2_shafer()` (lines 137+)

### Layer 2: Reference Oscillator (`dpw4`)

A 4th-order DPW oscillator producing deterministic waveforms (sawtooth,
pulse, triangle, sine) with SHA-256-verified outputs. Its
architecture-relevant properties are:

- **Signal determinism**: All computation is integer or fixed-point. Phase
  is a 32-bit wrapping accumulator; the DPW4 polynomial operates in Q124
  via a 3rd-order differentiator. Final output is i32 after gain scaling
  and 1-bit headroom saturation.
- **Extended precision**: Triangle integration uses I256 (256-bit signed
  integer with modular arithmetic) to prevent accumulator overflow.
- **Hash-locked outputs**: Six normative scenarios produce frozen SHA-256
  hashes. Golden byte-array hashes are independently verified in forensic
  audit tests (see Section 9.2).

Evidence:
- `crates/dpw4/src/lib.rs`: `Dpw4State` (line 90), `compute_x2_q124()` (line 141), `tick_dpw4_raw()` (line 172), `apply_gain()` (line 287)
- `crates/dpw4/src/i256.rs`: `I256` struct and operations (lines 2–78)
- `crates/dpw4/src/constants.rs`: `HEADROOM_BITS` (line 71), `DISCONTINUITY_THRESHOLD` (line 38)

### Layer 3: Capture Firmware (`replay-fw-f446`, `replay-embed`)

Bare-metal firmware on STM32F446RE capturing interrupt-driven execution
samples into a fixed-size buffer and emitting them as an RPL0 artifact over
USART2.

- **Timer configuration**: TIM2 at 1 kHz update interrupt (PSC=15, ARR=999
  from 16 MHz APB1 clock).
- **ISR capture**: Phase accumulator advances by `STEP = 0x0100_0000` per
  interrupt. Sample is extracted as `(phase >> 24) as i32`. Stored into a
  static `[i32; 10_000]` buffer protected by Cortex-M critical sections.
- **Artifact emission**: After 10,000 frames are captured, firmware disables
  interrupts and emits the complete RPL0 artifact (16-byte header + 10,000
  × 16-byte frames) over USART2 at 115200 baud.
- **Demo perturbation modes**: Feature flags `demo-divergence` and
  `demo-persistent-divergence` inject controlled perturbations at frame 4096
  for divergence analysis demonstration.

Evidence:
- `crates/replay-fw-f446/src/fw.rs`: `tim2_isr()` (lines 95–149), `init_tim2_1khz()` (lines 182+), `dump_artifact()` (lines 221+), `write_header0()` (line 251), `write_event_frame0()` (line 259)
- `crates/replay-fw-f446/src/main.rs`: TIM2 interrupt binding
- `crates/replay-fw-f446/memory.x`: FLASH 512K at 0x08000000, RAM 128K at 0x20000000

### Layer 4: Replay Engine (`replay-host`, `replay-core`)

A deterministic state machine replaying artifact frame streams and producing
hash streams for comparison. This Rust replay path remains experimental: it
supports RPL0 format version 0 replay, RPL0 format version 1 container parsing,
and current replay only under the legacy 16-byte `EventFrame0` interpretation.

- **State structure**: `SutState0 { timer_sum: u64, sample_fold: u32, irq_count: u32 }`.
  Zero-initialized.
- **State transition** (`step0`): Each frame advances `timer_sum` via
  wrapping addition, mixes `sample_fold` via rotate-left-by-`(irq_id & 31)`
  XOR'd with sample, frame index, flags, and reserved field. Increments
  `irq_count`.
- **Hash function** (`hash_state0`): Merges all state fields at different
  bit offsets, then applies the SplitMix64 finalizer (two multiply-XOR-shift
  rounds) for avalanche diffusion.
- **Hash stream** (`replay_hashes0`): Iterates all frames, producing one
  cumulative hash per frame.
- **First-divergence detection** (`first_divergence0`): Linear scan comparing
  two hash vectors element-wise. Returns first mismatching index. Handles
  length differences by reporting divergence at `min(len_a, len_b)`.
- **Artifact comparison** (`diff_artifacts0`): Parses both artifacts, replays
  both, compares hash streams.

Evidence:
- `crates/replay-host/src/replay.rs`: `SutState0` (lines 6–10), `step0()` (lines 12–27), `hash_state0()` (lines 28–39), `replay_hashes0()` (lines 41–52), `first_divergence0()` (lines 55–73), `diff_artifacts0()` (lines 78+)
- `crates/replay-host/src/artifact.rs`: `parse_header0()`, `parse_frames0()`, `parse_artifact()`, `parse_replay_frames_legacy0()`

### Layer 5: Divergence Analysis (`scripts/artifact_diff.py`)

A deterministic analysis pipeline classifying divergence between two artifacts
along four dimensions: localization, region, shape, and evolution.

Described in detail in Section 7.

---

## 4. Artifact Contract

The artifact contract defines the binary wire format that constitutes the
artifact boundary. Two layout versions exist: **RPL0 format version 0**
(fixed-layout, legacy) and **RPL0 format version 1** (variable-layout with
schema). They share the `RPL0` magic and the `EventFrame0` wire format but
differ in header structure and validation rules. The released operator-facing
parser, verification, and comparison workflows are the Python tooling routed by
[docs/replay/tooling.md](../replay/tooling.md). The Rust replay path remains experimental: it supports
RPL0 format version 0 replay, RPL0 format version 1 container parsing, and
current replay only under the legacy 16-byte `EventFrame0` interpretation.

### RPL0 Format Version 0 Layout

```
Offset    Size    Field            Encoding
──────    ────    ─────            ────────
0x00      4       magic            ASCII "RPL0"
0x04      4       version          u32 LE (= 0)
0x08      4       frame_count      u32 LE (= 10,000)
0x0C      4       reserved         u32 LE (= 0)
0x10      N×16    frame array      N × EventFrame0
```

Total artifact size (RPL0 format version 0): 160,016 bytes (16 + 10,000 × 16).

### RPL0 Format Version 1 Layout

```
Offset    Size    Field              Encoding
──────    ────    ─────              ────────
0x00      4       magic              ASCII "RPL0"
0x04      2       version            u16 LE (= 1)
0x06      2       header_len         u16 LE (≥ 0x98)
0x08      4       frame_count        u32 LE
0x0C      2       frame_size         u16 LE (= 16)
0x0E      2       flags              u16 LE (= 0)
0x10      4       schema_len         u32 LE
0x14      32      schema_hash        SHA-256
0x34      32      build_hash         SHA-256
0x54      32      config_hash        SHA-256
0x74      16      board_id           bytes
0x84      16      clock_profile      bytes
0x94      2       capture_boundary   u16 LE
0x96      2       reserved           u16 LE (= 0)
```

The RPL0 format version 1 header is followed by an optional schema block and
then the frame array.

### Version Dispatch

Version dispatch reads offset 0x04 as a u32. If the value is 0, the artifact
is RPL0 format version 0. If non-zero, offset 0x04 is reinterpreted as u16
(expecting value 1) and RPL0 format version 1 parsing proceeds. This dispatch
rule is frozen.

Evidence:
- `scripts/inspect_artifact.py`: version dispatch logic (lines 115–170)

### EventFrame0 Structure

```
Offset    Size    Field           Encoding
──────    ────    ─────           ────────
0x00      4       frame_idx       u32 LE
0x04      1       irq_id          u8
0x05      1       flags           u8
0x06      2       rsv             u16 LE
0x08      4       timer_delta     u32 LE
0x0C      4       input_sample    i32 LE (signed)
```

Each frame is 16 bytes. All multi-byte fields are little-endian.

Evidence:
- `crates/replay-core/src/artifact.rs`: `Header0` (lines 26–31), `EventFrame0` (lines 39–48), encoding functions (lines 53–74), compile-time size assertions (lines 16–18)
- `crates/replay-fw-f446/src/fw.rs`: `write_header0()` (line 251), `write_event_frame0()` (line 259)

### Validation Rules

**RPL0 format version 0 validation:**
- Magic must equal `b"RPL0"`.
- Version (u32 at 0x04) must equal 0.
- Reserved (u32 at 0x0C) must equal 0.
- File size must equal `HEADER_SIZE + frame_count × FRAME_SIZE`.

**RPL0 format version 1 validation:**
- Magic must equal `b"RPL0"`.
- Version (u16 at 0x04) must equal 1.
- `header_len` must be ≥ 0x98 and ≤ file length.
- `frame_size` must equal 16.
- `flags` must equal 0.
- `reserved` must equal 0.
- `SHA256(schema_block) == stored schema_hash`.
- File size must equal `header_len + schema_len + frame_count × frame_size`.

Evidence:
- `scripts/inspect_artifact.py`: RPL0 format version 0 validation (lines 130–155), RPL0 format version 1 validation (lines 170–210)
- `scripts/test_artifact_parser_adversarial.py`: adversarial validation tests (lines 85+)
- `scripts/test_artifact_parser_valid_v1.py`: valid RPL0 format version 1 corpus tests (lines 87–160)

### Identity Hashing

Artifact identity is computed as SHA-256 over the canonical region only.
Trailing bytes beyond `canonical_len` are excluded from the hash.

- RPL0 format version 0: `canonical_len = HEADER_SIZE + frame_count × FRAME_SIZE`
- RPL0 format version 1: `canonical_len = header_len + schema_len + frame_count × frame_size`

Evidence:
- `scripts/artifact_tool.py`: `cmd_hash()` (line 102)
- `artifacts/baseline.sha256`: stored baseline hash

---

## 5. Deterministic Replay

### State Machine

Replay is a pure function from frame stream to hash stream. The state machine
maintains three accumulators:

| Field | Type | Initial | Update Rule |
|-------|------|---------|-------------|
| `timer_sum` | u64 | 0 | `wrapping_add(frame.timer_delta)` |
| `sample_fold` | u32 | 0 | `rotate_left(irq_id & 31) ^ sample ^ frame_idx ^ (flags << 24) ^ (rsv << 8)` |
| `irq_count` | u32 | 0 | `wrapping_add(1)` |

The `sample_fold` field provides cross-field diffusion: a single-bit change in
any frame field propagates through rotation and XOR to affect the accumulated
state.

### Hash Function

After each state transition, a 64-bit hash is computed:

1. Merge fields: `x = timer_sum ^ (sample_fold << 1) ^ (irq_count << 33)`.
2. SplitMix64 finalizer:
   - `x ^= x >> 30; x *= 0xbf58_476d_1ce4_e5b9`
   - `x ^= x >> 27; x *= 0x94d0_49bb_1331_11eb`
   - `x ^= x >> 31`
3. Output: single u64 per frame.

The SplitMix64 finalizer provides strong avalanche-style diffusion: input
bit changes propagate broadly across output bits.

### Comparison

Two artifacts are compared by generating their hash streams independently and
scanning for the first mismatch. The comparison is:

- O(n) in frame count.
- Requires only the artifacts (not the original runtime).
- Reports `None` for identical artifacts or `Some(index)` for the first
  divergent frame.

### Complexity Bounds

All pipeline stages are linear in the frame count *n*:

| Operation | Function | Complexity |
|-----------|----------|------------|
| Replay | `replay_hashes0()` | O(n) |
| Diff | `first_divergence0()` | O(n) |
| Classification | `artifact_diff.py` pipeline | O(n) |

No stage allocates beyond the hash-stream vector (8 bytes × *n*). The
analysis pipeline performs a constant number of linear passes over the frame
array.

Evidence:
- `crates/replay-host/src/replay.rs`: complete replay implementation (lines 1–91)

---

## 6. Divergence Analysis

The divergence analysis pipeline operates on two artifacts and produces a
structured explanation of their differences. The pipeline is implemented in
`scripts/artifact_diff.py` and governed by the normative contract in
[DIVERGENCE_SEMANTICS.md](../replay/DIVERGENCE_SEMANTICS.md).

### Analysis Pipeline

```
┌────────────┐    ┌────────────┐    ┌────────────┐    ┌────────────┐
│ Localize   │───>│  Region    │───>│   Shape    │───>│ Evolution  │
│            │    │ Attribute  │    │ Classify   │    │ Classify   │
│ Find first │    │ Identify   │    │ Categorize │    │ Describe   │
│ divergence │    │ affected   │    │ sample     │    │ post-dive  │
│ frame      │    │ fields     │    │ trajectory │    │ trajectory │
└────────────┘    └────────────┘    └────────────┘    └────────────┘
```

### Stage 1: Localization

Find the first frame index where a supported field differs between the two
artifacts. If headers differ, `first_divergence_frame = 0`. Otherwise, scan
frames sequentially. If no difference exists, report `none`.

Evidence:
- `scripts/artifact_diff.py`: localization logic within main pipeline (lines 260–300)
- [DIVERGENCE_SEMANTICS.md](../replay/DIVERGENCE_SEMANTICS.md): localization rules

### Stage 2: Region Attribution

At the first divergence frame, identify which supported regions differ. The
supported regions, in fixed precedence order, are:

1. `header_schema` — header or pre-frame byte differences.
2. `timer_delta` — per-frame `timer_delta` field difference.
3. `irq_state` — per-frame `irq_id` field difference.
4. `sample_payload` — per-frame `input_sample` field difference.

`primary_region` is the first region in the precedence list that appears at the
first divergence frame.

Differences in unsupported fields (`frame_idx`, `flags`, `reserved`) cause the
tool to exit with `FAIL`. No silent fallback is permitted.

Evidence:
- `scripts/artifact_diff.py`: `frame_supported_diffs()` (line 123), `primary_region()` (line 104)
- `scripts/generate_demo_v4_fixtures.py`: region test fixtures

### Stage 3: Shape Classification

Shape classification applies only when `sample_payload` is present at the first
divergence frame. Let N = first divergence frame, `diff[i] = sample_A[i] - sample_B[i]`
for i ≥ N, and K = 8 (transient window).

Rules are evaluated in strict precedence:

| Class | Rule |
|-------|------|
| `transient` | ∃ r ∈ [N+1, N+K]: diff[r] = 0 ∧ ∀ j ∈ [r, min(N+K, end)]: diff[j] = 0 |
| `persistent_offset` | diff[i] is constant for all remaining frames |
| `rate_divergence` | \|diff[i+1]\| ≥ \|diff[i]\| for all remaining frames, with at least one strict increase |

If `sample_payload` participates but no rule matches, the tool exits with
`FAIL`.

Evidence:
- `scripts/artifact_diff.py`: `classify_sample_diffs()` (line 69)
- `scripts/generate_demo_v3_fixtures.py`: shape test fixtures (line 91+)
- `scripts/test_artifact_diff.py`: non-classifiable failure test (line 47+)

### Stage 4: Evolution Classification

Evolution classification describes the trajectory of divergence after the first
divergent frame. Rules are evaluated in strict precedence:

| Class | Rule |
|-------|------|
| `region_transition` | A later frame contains a supported region not present at the first divergence frame. |
| `self_healing` | No new region appears. After some later frame, all remaining frames are identical. |
| `monotonic_growth` | No new region. Not self-healing. `sample_payload` present. \|diff\| is nondecreasing with at least one strict increase. |
| `bounded_persistent` | Fallback. Divergence persists through the final frame without introducing new regions. |

Evidence:
- `scripts/artifact_diff.py`: `classify_evolution()` (line 150)
- `scripts/generate_demo_v5_fixtures.py`: evolution test fixtures (lines 81–108)

### Output Contract

The stable output fields are:

| Field | Type | Meaning |
|-------|------|---------|
| `first_divergence_frame` | int or `none` | First frame with supported divergence |
| `primary_region` | string or `none` | Highest-precedence region at first divergence |
| `all_regions_at_first_divergence` | list | All supported regions at first divergence frame |
| `region_summary` | `none`, region name, or `mixed` | Summary of regions at first divergence |
| `shape_class` | enum or `none` | Sample trajectory shape |
| `evolution_class` | enum or `none` | Post-divergence trajectory |
| `timeline_summary` | string or `none` | Narrative derived from evolution class |

Evidence:
- [DIVERGENCE_SEMANTICS.md](../replay/DIVERGENCE_SEMANTICS.md): complete normative semantics (lines 1–200)

---

## 7. Hardware / Software Integration

### Capture Node

The capture node is an STM32F446RE microcontroller on a Nucleo-64 board.

- **Processor**: ARM Cortex-M4F, 180 MHz maximum (operated at default HSI
  16 MHz internal RC oscillator — no PLL initialization).
- **Memory**: 512 KB Flash at 0x08000000, 128 KB RAM at 0x20000000.
- **Timer**: TIM2 configured for 1 kHz update interrupt (PSC=15, ARR=999).
- **Serial**: USART2 at 115200 baud, TX-only via PA2 (AF7).
- **Debug**: ST-LINK V2-1 via SWD.

### Capture Sequence

1. Firmware initializes peripherals (GPIO, USART2, TIM2).
2. TIM2 interrupt is unmasked at NVIC.
3. Each TIM2 update interrupt:
   a. Advances phase accumulator by `STEP = 0x0100_0000`.
   b. Extracts sample: `(phase >> 24) as i32`.
   c. Stores sample in static buffer via critical section.
   d. Increments write index (atomic, release ordering).
4. After 10,000 frames: sets `CAPTURE_DONE` flag.
5. Main loop detects flag, disables interrupts and TIM2.
6. `dump_artifact()` emits the RPL0 binary stream over USART2.

### Host Capture

The host reads the serial stream using `scripts/read_artifact.py`:

1. Scan for `b"RPL0"` magic bytes (up to 64 KB scan window).
2. Parse 16-byte header.
3. Read and validate 10,000 × 16-byte frames.
4. Write raw binary artifact to file.

Validation checks per frame (unless `--no-verify`):
- `frame_idx` matches expected sequence.
- `irq_id` equals 0x02 (TIM2).
- `timer_delta` equals 1000.
- Optional signal model verification.

Evidence:
- `crates/replay-fw-f446/src/fw.rs`: complete firmware implementation (290+ lines)
- `crates/replay-fw-f446/memory.x`: memory layout
- `boards/stm32f446-stlink.toml`: probe configuration
- `scripts/read_artifact.py`: host capture script (`main()` line 83+)

---

## 8. Platform Topology

```
┌─────────────────────────┐         ┌─────────────────────────────────────┐
│     CAPTURE NODE        │         │          ANALYSIS HOST              │
│                         │         │                                     │
│  STM32F446RE            │  USART2 │  Artifact storage                   │
│  ┌───────────────────┐  │ 115200  │  ┌─────────────────────────────┐   │
│  │ TIM2 ISR          │  │ ──────> │  │ read_artifact.py            │   │
│  │ Phase accumulator  │  │  serial │  │ Artifact A (.rpl)           │   │
│  │ Sample buffer      │  │         │  └─────────────────────────────┘   │
│  │ dump_artifact()    │  │         │                                     │
│  └───────────────────┘  │         │  Replay engine                      │
│                         │         │  ┌─────────────────────────────┐   │
│  PA2 (USART2 TX)       │         │  │ replay_hashes0()            │   │
│  ST-LINK SWD           │         │  │ Hash stream A               │   │
│                         │         │  │ Hash stream B               │   │
└─────────────────────────┘         │  └─────────────────────────────┘   │
                                    │                                     │
                                    │  Divergence analysis                │
                                    │  ┌─────────────────────────────┐   │
                                    │  │ artifact_diff.py            │   │
                                    │  │ Localization                 │   │
                                    │  │ Region attribution           │   │
                                    │  │ Shape classification         │   │
                                    │  │ Evolution classification     │   │
                                    │  └─────────────────────────────┘   │
                                    │                                     │
                                    │  Verification                       │
                                    │  ┌─────────────────────────────┐   │
                                    │  │ precision validate           │   │
                                    │  │ verify_kani.sh               │   │
                                    │  │ verify_release_repro.sh      │   │
                                    │  └─────────────────────────────┘   │
                                    └─────────────────────────────────────┘
```

The architecture separates capture (physical, ISR-driven, hardware-dependent)
from analysis (deterministic, software-only, host-based). The artifact is the
sole coupling between these domains.

---

## 9. Verification Architecture

The verification architecture is organized into four mechanisms.

### 9.1 Formal Verification (Kani)

The repository contains 34+ Kani harnesses in source verifying absence of
panics, arithmetic overflow safety, and specification conformance for
critical math and state operations. The normative default runner executes a
narrower Tier-1 subset defined by `verify_kani.sh`.

**Tier-1 harnesses** (fast, run by default):

| Crate | Harness | Property |
|-------|---------|----------|
| `geom-signal` | `proof_sqrt_no_panic` | Newton-Raphson safety (64 iterations) |
| `geom-signal` | `proof_sin_cos_no_panic` | CORDIC convergence + iteration bounds |
| `dpw4` | `proof_compute_x2_safe` | x² overflow safety |
| `dpw4` | `proof_saturate_safe` | i32 saturation correctness |
| `dpw4` | `proof_phase_u32_no_overflow` | Phase conversion safety |
| `dpw4` | `proof_phase_u32_fixed_to_u32_conversion` | Fixed-to-u32 phase conversion safety |
| `dpw4` | `proof_sine_scale_no_overflow` | Sine scaling overflow safety |
| `dpw4` | `proof_sine_to_i32_in_range` | Sine output stays within i32 range |
| `dpw4` | `proof_sine_egress_bounded` | Sine egress boundedness |
| `dpw4` | `proof_triangle_delta_clamp_identity_when_in_range` | Triangle clamp identity in range |
| `dpw4` | `proof_triangle_delta_clamp_saturates_when_out_of_range` | Triangle clamp saturation out of range |
| `dpw4` | `proof_triangle_z_update_is_saturating` | Triangle state update saturation |
| `dpw4` | `proof_spec_clamp_in_range_contract` | I256→I128 clamp (in-range) |
| `dpw4` | `proof_spec_clamp_out_of_range_contract` | I256→I128 clamp (out-of-range) |
| `dpw4` | `proof_spec_sar_sanity` | I256 arithmetic right-shift |
| `dpw4` | `proof_i256_sub_matches_spec` | I256 subtraction byte-level identity |
| `dpw4` | `proof_i256_sar_in_range_matches_spec` | I256 SAR (shift < 256) |
| `dpw4` | `proof_i256_sar_out_of_range_matches_spec` | I256 SAR (shift ≥ 256) sign extension |
| `dpw4` | `proof_i256_clamp_matches_spec` | Full clamping identity |
| `dpw4` | `proof_triangle_freeze_invariant` | Triangle freeze guard invariant |
| `replay-core` | `proof_v0_wire_size_constants` | v0 wire size constants |
| `replay-core` | `proof_encode_header0_wire_layout_and_le` | v0 header wire layout + LE encoding |
| `replay-core` | `proof_encode_event_frame0_wire_layout_and_le` | v0 frame wire layout + LE encoding |

**Tier-2 harnesses** (heavy, `RUN_HEAVY=1`):

| Crate | Harness | Property |
|-------|---------|----------|
| `geom-signal` | `proof_atan2_q1` through `proof_atan2_q4` | Per-quadrant atan2 range [-π, π] |
| `dpw4` | `proof_i256_mul_u32_matches_spec` | I256 multiply-by-u32 oracle identity |

Kani configuration: unwind depth 65, solver `cadical`, 360-second timeout per
harness.

Evidence:
- `crates/geom-signal/src/verification.rs`: signal harnesses (lines 13–107)
- `crates/dpw4/src/verification.rs`: DPW4 harnesses (lines 218–500)
- `crates/dpw4/src/i256.rs`: I256 harnesses (lines 263–468)
- `verify_kani.sh`: runner script with tier dispatch and parallelization
- `verify_kani_tier2.sh`: tier-2 wrapper

### 9.2 Hash-Locked Reference Outputs

Six normative scenarios produce deterministic `.det.csv` files whose SHA-256
hashes are frozen in source code. These hashes may only change during semantic
version bumps.

| Scenario | Description | Pinned SHA-256 Prefix |
|----------|-------------|----------------------|
| `saw_20_headroom` | 20 Hz sawtooth, 4800 samples @ 48 kHz | `ec99d4d0...` |
| `pulse_relational_8k` | 8 kHz pulse 10% duty, 300 samples @ 48 kHz | `a3b8e9f6...` |
| `triangle_linearity_1k` | 1 kHz triangle, 4800 samples @ 48 kHz | `9d2cb61f...` |
| `sine_linearity_1k` | 1 kHz sine (CORDIC), 4800 samples @ 48 kHz | `e30e4400...` |
| `master_sweep_20_20k` | 20 Hz–20 kHz chirp, 4 shapes, 48000 samples | `6ad85015...` |
| `long_run_0_1hz` | 0.1 Hz sawtooth, 1,000,000 samples @ 48 kHz | `3f2a364c...` |

Additionally, golden byte-array hashes are stored for sawtooth and pulse
waveforms in `crates/dpw4/src/goldens.rs` and enforced in forensic audit tests.

Evidence:
- `crates/dpw4/src/bin/precision.rs`: `NORMATIVE_DET_HASHES` (line 181+)
- `crates/dpw4/src/goldens.rs`: `SAW_GOLDEN_HASH` (line 13), `PULSE_GOLDEN_HASH` (line 19)
- `crates/dpw4/tests/forensic_audit.rs`: golden lock tests (`test_golden_lock` line 5)

### 9.3 Deterministic Build Enforcement

The `verify_release_repro.sh` script builds the `precision` binary twice in
isolated target directories (`target_a/`, `target_b/`) with `--locked` and
compares their SHA-256 hashes. Bit-identical binaries confirm deterministic
compilation.

Controls:
- `--locked` enforces exact Cargo.lock dependency versions.
- Separate `CARGO_TARGET_DIR` prevents cross-contamination.
- Clean build from scratch (removes prior target directories).

Evidence:
- `verify_release_repro.sh`: dual-build comparison script
- `rust-toolchain.toml`: pinned channel `1.91.1`

### 9.4 Multi-Tier Validation Gate (`precision validate`)

`make gate` is the canonical operator-facing release gate.
Its normative underlying command, `precision validate --mode quick`, executes
the following verification pipeline:

| Tier | Check | Description |
|------|-------|-------------|
| 0 | `version_consistency` | Workspace `Cargo.toml` version matches `Cargo.lock` version |
| 0 | `toolchain_pin` | `rust-toolchain.toml` channel is `1.91.1` |
| 1 | `header_stream_integrity` | Generate 100-frame DP32 header stream, verify Fletcher-32 checksums |
| 2 | `determinism_bit_exact` | Two independent artifact generations produce byte-identical outputs |
| 2 | `det_baseline` | Actual `.det.csv` hash matches frozen normative hash |
| 2 | `canon_protocol` | `.canon.sig` format and integrity validation |
| 2 | `pinned_regression` | Golden policy enforcement for pinned scenarios |

Exit codes: 0 (all pass), 1 (error), 2 (integrity failure).

Evidence:
- `crates/dpw4/src/bin/precision.rs`: validation pipeline (`run_validate()` line 303+)
- `crates/dpw4/src/checksum.rs`: Fletcher-32 implementation (`fletcher32_checked()` line 28)

---

## 10. Comparison With Existing Systems

The architecture occupies a specific position in the design space.

### Execution Tracing

Traditional execution tracers (e.g., ARM ETM, Intel PT) produce
hardware-generated instruction traces. These traces are tightly coupled to
processor architecture and require specialized decoders. The precision-signal
approach captures application-level execution samples at a defined rate,
producing portable artifacts independent of instruction-set architecture.

### Record/Replay Systems

Record/replay systems (e.g., rr, Mozilla RR) record nondeterministic inputs
and replay them to reproduce execution. They require the original binary and
often the original OS environment. The precision-signal approach does not
reproduce execution. It replays artifacts through an independent state machine,
enabling comparison without environment reproduction.

### Differential Testing

Differential testing runs multiple implementations on the same input and
compares outputs. The precision-signal approach is structurally similar but
operates on execution artifacts rather than function outputs, and provides
frame-level divergence localization rather than binary pass/fail.

### Signal Integrity Analysis

Signal analysis tools (e.g., oscilloscope FFT, spectrum analyzers) operate on
continuous waveforms. The precision-signal approach operates on discrete captured
frames with deterministic replay, providing structural divergence classification
rather than spectral analysis.

### Distinguishing Properties

The architecture differs from the above in a specific combination:

1. The artifact is the comparison unit, not the execution environment.
2. Replay is deterministic and independent of capture conditions.
3. Divergence is localized to frame, region, shape, and evolution.
4. Verification combines formal proofs, hash locks, and build determinism.

---

## 11. Current Implementation and Authority Routing

This section enumerates current implementation surfaces. Release-surface
classification is delegated to [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md); verification authority
is delegated to [VERIFICATION_GUIDE.md](../../VERIFICATION_GUIDE.md). The Rust replay engine remains
present in the workspace but is not classified as released unless
promoted by [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md).

### Current Surface Summary

| Component | Implementation Status | Implementation |
|-----------|--------|---------------|
| RPL0 artifact format definitions and Python reference inspection/parser tooling (legacy-header and extended-header paths) | Implemented; normative format defined in [docs/spec/rpl0_artifact_contract.md](../spec/rpl0_artifact_contract.md) | `crates/replay-core/src/artifact.rs`, `scripts/inspect_artifact.py` |
| Rust replay engine artifact consumption | Implemented in an experimental crate for RPL0 format version 0 replay, RPL0 format version 1 container parsing, and current replay under the legacy 16-byte `EventFrame0` interpretation | `crates/replay-host/src/replay.rs`, `crates/replay-host/src/artifact.rs` |
| First-divergence localization | Implemented; released workflows use Python comparison tooling, while Rust replay implementation remains experimental | `crates/replay-host/src/replay.rs`, `scripts/artifact_diff.py` |
| Region attribution | Stable | `scripts/artifact_diff.py` |
| Shape classification | Stable | `scripts/artifact_diff.py` |
| Evolution classification | Stable | `scripts/artifact_diff.py` |
| Deterministic reference oscillator (DPW4) | Stable | `crates/dpw4/src/lib.rs` |
| Fixed-point math core | Stable | `crates/geom-signal/src/` |
| Kani harness inventory (34+ in source) | Implemented; normative evidence is limited to the runner manifest and logs | `crates/*/src/verification.rs`, `crates/dpw4/src/i256.rs`, `verify_kani.sh` |
| Hash-locked reference outputs (6 scenarios) | Stable | `crates/dpw4/src/bin/precision.rs` |
| Deterministic build verification | Stable | `verify_release_repro.sh` |
| Multi-tier validation gate | Stable | `precision validate` CLI |
| Bare-metal capture firmware (STM32F446RE) | Implemented; current release-surface classification for `replay-fw-f446` is routed to [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md) and retained release `1.2.2` evidence lives under [docs/verification/releases/1.2.2/](../verification/releases/1.2.2/). The release `1.2.2` bundle records a direct hardware-backed rerun. | `crates/replay-fw-f446/` |
| Artifact parsing and comparison tools | Stable | `scripts/` Python tooling |
| Demo divergence ladder (Demo V1-Demo V5) | Stable | `scripts/generate_demo_v*_fixtures.py` |
| Normative divergence semantics | Stable | [docs/replay/DIVERGENCE_SEMANTICS.md](../replay/DIVERGENCE_SEMANTICS.md) |

### Workspace Routing

The workspace contains crates at different maturity levels, but this document
does not classify release maturity. For current release-surface questions, use
[docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md). For verification-admissibility questions, use
[VERIFICATION_GUIDE.md](../../VERIFICATION_GUIDE.md).

| Crate | Role |
|-------|------|
| `geom-signal` | Fixed-point math core |
| `geom-spatial` | 3D spatial math support crate |
| `dpw4` | Reference oscillator and validation CLI |
| `replay-core` | Replay core scaffolding and artifact definitions used by the experimental Rust replay path |
| `replay-host` | Experimental Rust replay host; current support is RPL0 format version 0 replay plus RPL0 format version 1 container parsing with legacy-frame replay semantics |
| `replay-fw-f446` | Replay capture firmware for the active RPL0 format version 1 operator path |
| `replay-embed` | Embedded replay scaffolding |
| `replay-cli` | Workspace placeholder library; not a currently exposed CLI surface |

---

## 12. Future Directions

The following are identified as potential extensions. None are implemented in
the current release surface. Each is labeled as a future direction and should not be interpreted as a
current capability.

**Multi-node capture.** The current system captures from a single
STM32F446RE node. Extending to multiple capture nodes would require artifact
synchronization and a merge protocol. Status: not implemented.

**Distributed replay infrastructure.** The current replay engine runs on a
single host. Distributing replay across multiple hosts would require
partitioning strategies for frame streams. Status: not implemented.

**Automated analysis appliances.** The current analysis pipeline is
invoked manually via CLI and Python scripts. Packaging the pipeline as an
automated appliance (continuous monitoring, artifact ingestion, report
generation) is not implemented.


**Extended artifact format.** The v1 header includes fields
(`build_hash`, `config_hash`, `board_id`, `clock_profile`) that support
future provenance tracking. These fields are defined in the format but not
yet used by the analysis pipeline.

---

## 13. What the Artifact Is Sufficient to Prove (and What It Is Not)

**The artifact is sufficient to prove:**

- **Deterministic replay**: Given the same artifact, any compliant replay
  engine on any platform with correct integer arithmetic will produce the
  same hash stream. This is structural, not empirical — the replay function
  is pure.
- **Divergence localization**: Two artifacts that differ can be compared to
  identify the first divergent frame, the affected regions, the shape of the
  divergence, and its evolution over time. The analysis is deterministic and
  reproducible.
- **Deterministic comparison signal**: Two artifacts with identical hash
  streams produce the same replay comparison signal. Artifact identity and
  field-level equivalence must be established by raw byte comparison and
  parsed-field comparison tooling, not by 64-bit replay hash equality alone.

**The artifact is NOT sufficient to prove:**

- **Root cause of divergence**: The artifact records *what* changed, not
  *why*. A divergence in `timer_delta` might indicate clock drift, ISR
  preemption, or a firmware bug — the artifact cannot distinguish these.
- **Temporal fidelity of capture**: The artifact does not prove that the
  capture node's timer was accurate in wall-clock terms. It records the
  timer values as reported by the hardware.
- **Completeness of execution state**: The artifact captures a fixed subset
  of execution state (phase, sample, timer delta, IRQ ID, flags). Any state
  not captured is invisible to replay and analysis.
- **Absence of hardware faults**: A bit-perfect artifact does not prove the
  hardware was operating correctly — only that the captured fields were
  self-consistent under replay.

---

## 14. Conclusion

The `precision-signal` repository implements a deterministic execution analysis
infrastructure organized around a central invariant: the artifact boundary.

Execution artifacts captured from physical hardware are encoded in the RPL0
binary format. A deterministic replay engine reconstructs state from
artifact frames and produces hash streams. Divergence analysis classifies
differences between hash streams along four dimensions: localization, region,
shape, and evolution.

The verification architecture combines formal proofs (Kani), hash-locked
reference outputs, deterministic build enforcement, and a multi-tier validation
gate to substantiate determinism claims.

The architectural contribution is that execution comparison does not require
execution reproduction. The artifact boundary decouples physical capture from
deterministic analysis, enabling structured comparison of any two executions
that produced valid artifacts.

---

## Appendix A: Architectural Invariants

1. **Artifact boundary invariant.** Everything to the left of the artifact
   (capture, hardware, ISR timing) is runtime-dependent. Everything to the
   right (replay, hashing, analysis) is deterministic and reproducible from
   the artifact alone.

2. **Replay determinism invariant.** Given identical frame streams, `replay_hashes0()`
   produces identical hash streams on any platform with correct integer arithmetic.

3. **Hash diffusion invariant.** The SplitMix64 finalizer provides strong
   avalanche-style diffusion: input bit changes propagate broadly across
   output bits.

4. **Classification completeness invariant.** Every supported divergent pair
   receives exactly one shape class and one evolution class, provided all
   frames use supported fields. Divergences involving unsupported fields, or
   pairs that fail gating conditions, cause explicit failure (`FAIL`) —
   never silent misclassification.

5. **Version immutability invariant.** Frozen normative hashes may only change
   during semantic version bumps. Any other change is a conformance violation.

6. **Wire format invariant.** Encoding uses explicit field-by-field
   little-endian serialization. `repr(C)` is layout discipline; the wire
   contract is the encoder output.

---

## Appendix B: Divergence Localization Diagram

```
   Artifact A                          Artifact B
   ──────────                          ──────────
   Frame 0  ──> H_A[0]                Frame 0  ──> H_B[0]
   Frame 1  ──> H_A[1]                Frame 1  ──> H_B[1]
   Frame 2  ──> H_A[2]                Frame 2  ──> H_B[2]
      ...          ...                    ...          ...
   Frame N  ──> H_A[N]  ═══╗          Frame N  ──> H_B[N]  ═══╗
                            ║                                   ║
                            ╠══ H_A[N] ≠ H_B[N] ══════════════╣
                            ║                                   ║
                            ║   first_divergence_frame = N      ║
                            ║                                   ║
   Frame N+1 ──> H_A[N+1]  ║          Frame N+1 ──> H_B[N+1]  ║
   Frame N+2 ──> H_A[N+2]  ║          Frame N+2 ──> H_B[N+2]  ║
      ...                   ║             ...                   ║
                            ╚═══════════════════════════════════╝

   Analysis:
   ┌─────────────────────────────────────────────────┐
   │ region:    sample_payload                        │
   │ shape:     persistent_offset                     │
   │ evolution: bounded_persistent                    │
   │ timeline:  divergence remains in sample_payload  │
   │            through final frame                   │
   └─────────────────────────────────────────────────┘
```

---

## Appendix C: Claim Evidence Table

| # | Whitepaper Claim | Evidence | Status |
|---|-----------------|----------|--------|
| 1 | Artifacts follow RPL0 format with 4-byte magic "RPL0" | `crates/replay-core/src/artifact.rs` lines 4, 26 | supported |
| 2 | EventFrame0 is 16 bytes with 6 fields (frame_idx, irq_id, flags, rsv, timer_delta, input_sample) | `crates/replay-core/src/artifact.rs` lines 39–48 | supported |
| 3 | Header0 is 16 bytes with magic, version, frame_count, reserved | `crates/replay-core/src/artifact.rs` lines 26–31 | supported |
| 4 | Compile-time size assertions enforce wire constants | `crates/replay-core/src/artifact.rs` lines 16–18 | supported |
| 5 | v0 artifact size is exactly 160,016 bytes | `crates/replay-core/src/artifact.rs` line 13 | supported |
| 6 | Encoding uses explicit field-by-field little-endian serialization | `crates/replay-core/src/artifact.rs` lines 53–74 | supported |
| 7 | TIM2 configured at 1 kHz (PSC=15, ARR=999) | `crates/replay-fw-f446/src/fw.rs` line 182+ | supported |
| 8 | ISR captures 10,000 frames via phase accumulator | `crates/replay-fw-f446/src/fw.rs` lines 95–149, constant `FRAME_COUNT=10_000` line 19 | supported |
| 9 | Phase advances by STEP=0x0100_0000 per interrupt | `crates/replay-fw-f446/src/fw.rs` line 22 | supported |
| 10 | Sample extracted as `(phase >> 24) as i32` | `crates/replay-fw-f446/src/fw.rs` line 129 | supported |
| 11 | Artifact emitted over USART2 at 115200 baud | `crates/replay-fw-f446/src/fw.rs` `init_usart2()` line 165, `dump_artifact()` line 221 | supported |
| 12 | Buffer is static `[i32; 10_000]` protected by critical sections | `crates/replay-fw-f446/src/fw.rs` lines 31–40 | supported |
| 13 | Memory layout: FLASH 512K, RAM 128K | `crates/replay-fw-f446/memory.x` | supported |
| 14 | SutState0 has three fields: timer_sum (u64), sample_fold (u32), irq_count (u32) | `crates/replay-host/src/replay.rs` lines 6–10 | supported |
| 15 | step0() uses wrapping_add for timer_sum and rotate-XOR for sample_fold | `crates/replay-host/src/replay.rs` lines 12–27 | supported |
| 16 | hash_state0() applies SplitMix64 finalizer | `crates/replay-host/src/replay.rs` lines 28–39 | supported |
| 17 | replay_hashes0() produces one cumulative hash per frame | `crates/replay-host/src/replay.rs` lines 41–52 | supported |
| 18 | first_divergence0() performs linear scan returning first mismatch index | `crates/replay-host/src/replay.rs` lines 55–73 | supported |
| 19 | diff_artifacts0() parses, replays, and compares two artifacts | `crates/replay-host/src/replay.rs` line 78+ | supported |
| 20 | Divergence analysis supports 4 regions: header_schema, timer_delta, irq_state, sample_payload | `scripts/artifact_diff.py` lines 14–22, `frame_supported_diffs()` line 123; [DIVERGENCE_SEMANTICS.md](../replay/DIVERGENCE_SEMANTICS.md) | supported |
| 21 | Shape classification has 3 classes: transient, persistent_offset, rate_divergence | `scripts/artifact_diff.py` `classify_sample_diffs()` line 69; [DIVERGENCE_SEMANTICS.md](../replay/DIVERGENCE_SEMANTICS.md) | supported |
| 22 | Evolution classification has 4 classes: region_transition, self_healing, monotonic_growth, bounded_persistent | `scripts/artifact_diff.py` `classify_evolution()` line 150; [DIVERGENCE_SEMANTICS.md](../replay/DIVERGENCE_SEMANTICS.md) | supported |
| 23 | Unsupported field differences cause FAIL exit (no silent fallback) | `scripts/artifact_diff.py` `fail()` line 36; `scripts/test_artifact_diff.py` line 47+ | supported |
| 24 | Repository contains 34+ Kani harnesses in source across `geom-signal`, `dpw4`, and `replay-core` | `crates/geom-signal/src/verification.rs`; `crates/dpw4/src/verification.rs`; `crates/dpw4/src/i256.rs`; `crates/replay-core/src/verification.rs` | supported |
| 25 | Normative Kani runner executes a narrower manifest-defined subset; Tier-1 excludes `proof_atan_shafer_safety` and Tier-2 adds atan2 shard proofs plus `proof_i256_mul_u32_matches_spec` when `RUN_HEAVY=1` | `verify_kani.sh` HARNESS_MANIFEST; `verify_kani_tier2.sh` | supported |
| 26 | Six normative scenarios have frozen SHA-256 hashes | `crates/dpw4/src/bin/precision.rs` line 181+ | supported |
| 27 | Hash regeneration requires semantic version bump | [VERIFICATION_GUIDE.md](../../VERIFICATION_GUIDE.md) governance policy | supported |
| 28 | Deterministic build verified by dual-build SHA-256 comparison | `verify_release_repro.sh` | supported |
| 29 | Toolchain pinned to rustc 1.91.1 | `rust-toolchain.toml` | supported |
| 30 | precision validate runs multi-tier verification pipeline | `crates/dpw4/src/bin/precision.rs` `run_validate()` line 303+ | supported |
| 31 | Fletcher-32 checksum used for DP32 header integrity | `crates/dpw4/src/checksum.rs` `fletcher32_checked()` line 28 | supported |
| 32 | v1 header includes schema_hash validated as SHA-256 of schema block | `scripts/inspect_artifact.py` v1 parsing at lines 170–210 | supported |
| 33 | Artifact identity hash covers canonical region only (excludes trailing bytes) | `scripts/artifact_tool.py` `cmd_hash()` line 102 | supported |
| 34 | Scalar type is I64F64 (128-bit fixed-point) | `crates/geom-signal/src/lib.rs` line 13 | supported |
| 35 | CORDIC uses 64 iterations with 32-entry atan lookup table | `crates/geom-signal/src/math.rs` `sin_cos_kernel()` line 71+ | supported |
| 36 | DPW4 uses 3rd-order differentiator (z1, z2, z3) on x² polynomial | `crates/dpw4/src/lib.rs` `Dpw4State` line 90, `tick_dpw4_raw()` line 172 | supported |
| 37 | Gain model uses two-path precision strategy based on |raw| threshold 2^64 | `crates/dpw4/src/lib.rs` `apply_gain()` line 287 | supported |
| 38 | Triangle integration uses I256 (256-bit signed integer) with modular arithmetic | `crates/dpw4/src/i256.rs` lines 2–78; `crates/dpw4/src/lib.rs` `tick_triangle_dpw4()` line 626 | supported |
| 39 | Freeze guard halts triangle integration when dphi > 0x4000_0000 | `crates/dpw4/src/lib.rs` within `tick_triangle_dpw4()` ~line 660+ | supported |
| 40 | Demo perturbation modes inject controlled divergence at frame 4096 | `crates/replay-fw-f446/src/fw.rs` feature flags `demo-divergence`, `demo-persistent-divergence` | supported |
| 41 | Host capture script scans for RPL0 magic over serial | `scripts/read_artifact.py` line 83+ (scan at lines 91–101) | supported |
| 42 | Adversarial parser tests cover bad magic, invalid version, hash mismatch, truncation | `scripts/test_artifact_parser_adversarial.py` `main()` line 85+ | supported |
| 43 | v1 corpus tests validate parse invariants across randomized artifacts | `scripts/test_artifact_parser_valid_v1.py` `assert_parse_invariants()` line 87, `main()` line 116 | supported |
| 44 | Version dispatch reads offset 0x04 as u32; value 0 selects v0, non-zero selects v1 | `scripts/inspect_artifact.py` `parse_artifact_bytes()` lines 115–170 | supported |
| 45 | Multi-node capture is not implemented | No implementation found | not implemented |
| 46 | Distributed replay infrastructure is not implemented | No implementation found | not implemented |
| 47 | Automated analysis appliances are not implemented | No implementation found | not implemented |
| 48 | No additional operator subsystems are shipped beyond the documented replay and validation surfaces | [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md) | supported |
| 49 | Transient window K = 8 frames | `scripts/artifact_diff.py` `TRANSIENT_WINDOW_FRAMES` line 13; [DIVERGENCE_SEMANTICS.md](../replay/DIVERGENCE_SEMANTICS.md) | supported |
| 50 | Region precedence order is fixed: header_schema, timer_delta, irq_state, sample_payload | `scripts/artifact_diff.py` `PRIMARY_REGION_PRECEDENCE` lines 18–22; [DIVERGENCE_SEMANTICS.md](../replay/DIVERGENCE_SEMANTICS.md) | supported |
| 51 | Fixture generators produce test pairs for all classification classes | `scripts/generate_demo_v3_fixtures.py`, `generate_demo_v4_fixtures.py`, `generate_demo_v5_fixtures.py` | supported |
| 52 | Output headroom is 1-bit right-shift before saturation | `crates/dpw4/src/constants.rs` `HEADROOM_BITS` line 71; `crates/dpw4/src/lib.rs` `apply_gain()` line 287 | supported |
| 53 | All egress samples route through saturate_i128_to_i32 | `crates/dpw4/src/lib.rs` `apply_gain()` line 287+; [MATH_CONTRACT.md](../MATH_CONTRACT.md) §3 | supported |
| 54 | Hashing uses raw little-endian sample bytes only (no struct padding) | `crates/dpw4/tests/forensic_audit.rs` `test_golden_lock` line 5; [MATH_CONTRACT.md](../MATH_CONTRACT.md) §3 | supported |
