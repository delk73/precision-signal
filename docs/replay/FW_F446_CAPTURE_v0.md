# F446 Capture v0 — Legacy Capture Contract

> **Status: LEGACY / SUPERSEDED**
> This document retains the historical v0 capture contract for audit traceability only.
> Firmware capture now emits RPL0 version = 1 container artifacts with `[HEADER][SCHEMA BLOCK][FRAME DATA]`.
> Replay semantics remain the legacy 16-byte `EventFrame0` interpretation.
> Use [docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md](INTERVAL_CAPTURE_CONTRACT_v1.md)
> for the active STM32 capture contract and
> [docs/verification/releases/index.md](../verification/releases/index.md) for
> historical verification routing.

---

## Signal Contract

The phase8 signal model is the canonical replay signal:

```
sample = frame_idx & 0xFF
```

Mechanism: 32-bit phase accumulator with `STEP = 0x0100_0000`, read-then-advance ordering, output `(phase >> 24) as i32`.

The signal produces a deterministic 256-step periodic saw over `frame_idx = 0..9999`.

---

## Invariant Parameters

| Parameter | Value |
|---|---|
| `frame_count` | 10000 |
| `irq_id` | 2 (TIM2) |
| `timer_delta_nominal` | 1000 |
| `signal_model` | `phase8` |

---

## Artifact Format Freeze

| Parameter | Value |
|---|---|
| `artifact_header_version` | 0 |
| `artifact_magic` | `RPL0` |
| `frame_size_bytes` | 16 |
| `artifact_size_bytes` | 160016 |

Header and frame wire format is defined in [docs/replay/WIRE_FORMAT_v0.md](WIRE_FORMAT_v0.md).
All integer fields are little-endian.  Emission order: Header0 fields, then EventFrame0 entries in increasing `frame_idx`.

---

## Canonical Reference Artifact

```
path:         artifacts/baseline.bin
sha256:       b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3
signal_model: phase8
```

Metadata: `artifacts/baseline.json` (must contain `"signal_model": "phase8"` and matching `sha256`).

Hash file: `artifacts/baseline.sha256` (must be consistent with `baseline.json`).

---

## Deterministic Invariant

Identical firmware binary → identical artifact bytes.

Within the Phase C scope and validated capture workflow, identical `replay-fw-f446` firmware produces byte-identical artifacts.

### Allowed Variability

None.  Phase C scope permits no non-deterministic fields.

---

## Canonical Validation Runbook

```bash
# Flash
make flash-ur
make flash-compare-ur

# Capture
SERIAL=/dev/ttyACM0 python3 scripts/artifact_tool.py capture \
  --quick --signal-model phase8 --out artifacts/run.bin

# Inspect, verify, compare
python3 scripts/artifact_tool.py inspect artifacts/run.bin --frames 0:8
python3 scripts/artifact_tool.py verify artifacts/run.bin --signal-model phase8
python3 scripts/artifact_tool.py compare artifacts/baseline.bin artifacts/run.bin
python3 scripts/artifact_tool.py inspect artifacts/run.bin --frames 252:260
```

All capture tooling requires explicit `--signal-model`.  There is no implicit default.

---

## Wrap-Boundary Correctness

The phase8 signal wraps at `frame_idx = 256` (`sample = 0x00`).

Inspection of frames 252–260 must show:

| `frame_idx` | `input_sample` |
|---|---|
| 252 | `0xFC` |
| 253 | `0xFD` |
| 254 | `0xFE` |
| 255 | `0xFF` |
| 256 | `0x00` |
| 257 | `0x01` |
| 258 | `0x02` |
| 259 | `0x03` |
| 260 | `0x04` |

---

## Repeatability Evidence

Phase B repeatability proof: `artifacts/phase8_manifest.txt`.

All captures in the manifest must produce identical SHA-256 hashes matching `baseline.bin`.

---

## Signal Model History

| Model | Status | Contract |
|---|---|---|
| `phase8` | **Normative** | `sample = frame_idx & 0xFF` |
| `ramp` | Historical / non-normative | `sample = (frame_idx + 1) & 0xFFFFFFFF` |

The ramp model was used during Sprint 1–2 development.  It is retained in tooling for forensic use but is not part of the normative capture contract.

---

## Target and Scope

- Target MCU/board: STM32F446RE / NUCLEO-F446RE
- Crate: `crates/replay-fw-f446`
- Runtime model: PAC-only (`stm32f4`), `cortex-m-rt`, no HAL/RTOS/async
- Output: RPL0 artifact bytes over USART2 TX (ST-LINK VCP path)

## Capture and Dump Phases

- **Capture phase:**
  - TIM2 update interrupt runs at nominal 1 kHz.
  - ISR records exactly 10,000 deterministic samples.
  - During capture, only TIM2 IRQ is enabled by firmware.
- **Dump phase:**
  - TIM2 IRQ is disabled and timer stopped.
  - Global interrupts are disabled.
  - Firmware emits header then frames as raw bytes over USART2 using polling TX.

## Frame Population

- `frame_idx`: monotonically increasing index (`0..9999`)
- `irq_id`: constant `0x02` (TIM2)
- `flags`: `0`
- `rsv`: `0`
- `timer_delta`: constant `1000`
- `input_sample`: `(phase >> 24) as i32` per phase8 signal contract
- `clear_tim2_update_flag()`: TIM2 configuration assumes only UIF is active; if additional SR flags are enabled later, SR clear semantics must be re-reviewed.

## Optional Debug Instrumentation

- Feature: `debug-irq-count` (disabled by default).
- Purpose: provide deterministic evidence that TIM2 ISR execution continues during capture.
- Mechanism:
  - Exports `IRQ_COUNT` as `#[no_mangle] pub static AtomicU32`.
  - Resets `IRQ_COUNT` to `0` during startup before TIM2 IRQ enable.
  - Increments `IRQ_COUNT` at TIM2 ISR entry (`tim2_isr()`).
- Scope: observability only; no timer/capture/interrupt-routing behavior changes when the feature is disabled.

```bash
cargo build -p replay-fw-f446 --features debug-irq-count
```
