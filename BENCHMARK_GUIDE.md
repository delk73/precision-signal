# Precision-DPW: Performance Benchmark Guide
**Version: 1.0.0-rc5 (Reference Baseline)**
**Status: Advisory Reference**

---

This guide is retained historical performance background only.
It is not part of the active authority spine, does not define release claims,
and should be read behind [docs/README.md](docs/README.md) and
[docs/architecture/performance/README.md](docs/architecture/performance/README.md).

## Overview

This guide documents the **performance characteristics** of the Reference Baseline engine on reference hardware. These metrics are **advisory**—they characterize computational cost but are not normative requirements for conformance (see `VERIFICATION_GUIDE.md` for normative tests).

---

## Benchmark Hygiene

> [!NOTE]
> All benchmark results on Linux are **advisory**. Results vary with system state, scheduling, and thermal conditions.

### Recommended Run Conditions

For reproducible measurements:

1. **CPU Governor**: Set to `performance` mode
   ```bash
   sudo cpupower frequency-set -g performance
   ```
2. **Disable Turbo Boost** (if possible): Reduces thermal variance (x86 laptops/desktops often; Pi typically N/A)
3. **Minimize System Activity**: Close browsers, IDE indexers, background services
4. **Core Isolation** (optional): Pin to a specific CPU core
   ```bash
   taskset -c 3 cargo bench ...
   ```

### Interpreting Throughput Units

With Throughput::Elements(N), Criterion still measures time/iter, and additionally reports derived throughput metrics (ns/elem, Melem/s) normalized by N.

- **Sample-step benchmarks**: Set to `Elements(BATCH_SIZE)`. The reported time is for the full batch (256 samples). Rely on the detailed `thrpt` (Melem/s) or divide time by 256 for ns/sample.
- **Block benchmarks**: Set to `Elements(BLOCK_SIZE)`. The reported `ns/elem` is the **per-sample** cost averaged over the block, not the total time for the block.

**Column Interpretation**: With `Throughput::Elements(N)`, Criterion's **Time** column is time/iter (one batch), while **Throughput** (ns/elem or Melem/s) is normalized per-element.

To find the raw nanoseconds per iteration (ns/iter), observe the time without the `thrpt` column or remove the throughput configuration.

## Benchmark Categories

### Kernel Microbenchmarks

**Location:** `crates/geom-signal/benches/trig_throughput.rs`

**Purpose:**
- Measure individual primitive cost (sin_cos, sqrt, atan2_shafer, atan_shafer)
- Compare algorithm variants
- Detect catastrophic regressions

**Characteristics:**
- Isolated function calls, no chaining
- Advisory only—not acceptance-gated
- Useful for order-of-magnitude comparison and regression detection >10%

### Inner-Loop Throughput Benchmarks

**Location:** `crates/geom-signal/benches/signal_inner_loop.rs`

**Purpose:**
- Measure ns/sample (sample_step) and block-averaged ns/sample (block bench with Throughput::Elements(BLOCK_SIZE))
- Characterize engine hot-path execution patterns on Linux-baseline hardware
- Bridge between kernel cost and real-time feasibility

These benchmarks inform feasibility analysis; they do not define acceptance thresholds.

**Benchmarked workloads:**
- **DPW4 Sawtooth sample step**: `tick_dpw4` with pre-latched gain, incrementing phase
- **DPW4 Pulse sample step**: `tick_shape(shape = PULSE)` with 50% duty
- **Sine sample step**: `tick_shape(shape = SINE)` (CORDIC + scaling)

**Characteristics:**
- Criterion-controlled iteration count; inner loop body is fixed and allocation-free
- No control-plane logic inside loop
- Produces data for advisory ns/sample estimation (budgets not set by benchmarks)

**Critical requirement:** All benchmarks MUST be run under `--release` (which is default for `cargo bench`).

### Historical Performance Baselines (Advisory)

The following measurements characterize historical performance on reference platforms.

---

## Reference Hardware

### Raspberry Pi 3B
- **CPU:** ARMv8 Cortex-A53 (Quad-Core, 1.2 GHz)
- **Architecture:** 64-bit ARM
- **Use Case:** Embedded / Real-time audio synthesis baseline
- **Test Environment:** Isolated core via `taskset -c 3`

### Intel x86_64 (Development Baseline)
- **CPU:** Modern Intel processor (exact model varies by developer)
- **Architecture:** x86_64
- **Use Case:** Development and reference comparison
- **Test Environment:** Unlocked (default OS scheduling) or isolated core

---

## Benchmark Harnesses

| Harness | Package | Signal Type | Purpose |
|---------|---------|-------------|---------|
| `rpi_verify_logic` | `dpw4` | Sawtooth (Polynomial) | Baseline polynomial DPW performance |
| `rpi_verify_geometric` | `dpw4` | Sine (CORDIC) | 128-bit CORDIC engine cost |
| `spatial_bench` | `geom-spatial` | Vector3 Distance | Spatial coordinate solving cost |

---

## Measured Baselines (Reference v1.0.0-rc5)

### Raspberry Pi 3B (ARMv8)

| Engine | MHz | Real-time Factor @ 48kHz | Mono Voices |
|--------|-----|--------------------------|-------------|
| **Polynomial (Sawtooth)** | 2.34 | ~48x | ~48 |
| **CORDIC (Sine)** | 0.66 | ~13.8x | ~13 |

### Intel x86_64 (Development)

| Engine | MHz | Real-time Factor @ 48kHz | Mono Voices |
|--------|-----|--------------------------|-------------| 
| **Polynomial (Sawtooth)** | 19.1 | ~398x | ~398 |
| **CORDIC (Sine)** | 2.45 | ~51x | ~51 |
| **Spatial (Vector3)** | 6.8 | ~141x | ~141 |

**Note:** Spatial engine shows ~9% performance decrease from earlier measurements (~7.5 MHz → ~6.8 MHz) due to shift-scaling magnitude algorithm overhead. This trade-off extends the safe coordinate range from ~10⁹ m to 10¹² m while preventing overflow panics.

---

## Interpretation Guidelines

### Signal Budget Calculation (Audio Example)
```
Mono Voices = (Engine MHz) / (Sample Rate MHz)
            = (Engine MHz) / 0.048
```

Example: A 2.34 MHz Sawtooth engine at 48kHz supports ~48 mono voices.

### Cost of Geometry
The CORDIC engine (64-iteration 128-bit square root) is approximately **8x more expensive** than the polynomial DPW path on the Pi 3B. This ratio is consistent across architectures.

### Hardware Scaling
- **Pi 3B → x86_64:** ~8.3x speedup for polynomial, ~3.7x for CORDIC.
- **Explanation:** 128-bit software math benefits less from higher clock speeds than branch-heavy polynomial logic.

### Optimization Notes
- **Fast Path Available:** `sin_cos_fast` (skips 128-bit modulo) yields ~1.8% speedup on Pi 3B, confirming the 64-iteration loop is the dominant cost.
- **SIMD Opportunity:** AVX2/AVX-512 could theoretically yield ~5-8x speedup for spatial calculations, but would require dual-precision (f64) fallback as SIMD is not optimized for 128-bit fixed-point.

---

## Running Benchmarks

> [!IMPORTANT]
> All benchmarks automatically run in release mode. The `cargo bench` command
> does not accept the `--release` flag as it is implied.

### Kernel Microbenchmarks

```bash
# Run all kernel microbenchmarks
cargo bench -p geom-signal --bench trig_throughput
```

### Inner-Loop Throughput Benchmarks

```bash
# Run all inner-loop throughput benchmarks
cargo bench -p geom-signal --bench signal_inner_loop

# Run specific workload (sample step)
cargo bench -p geom-signal --bench signal_inner_loop -- dpw4_sawtooth_inner_loop/sample_step

# Run specific workload (block)
cargo bench -p geom-signal --bench signal_inner_loop -- dpw4_sawtooth_block/128
```

### Historical Performance Harnesses (Examples)

These are standalone performance measurement examples, not integrated benchmarks.

### Polynomial Engine (Sawtooth)
```bash
# Pi 3B (Isolated Core 3)
sudo taskset -c 3 cargo run -p dpw4 --release --example rpi_verify_logic

# x86_64 (Unlocked)
cargo run -p dpw4 --release --example rpi_verify_logic
```

### CORDIC Engine (Sine)
```bash
# Pi 3B (Isolated Core 3)
sudo taskset -c 3 cargo run -p dpw4 --release --example rpi_verify_geometric

# x86_64 (Unlocked)
cargo run -p dpw4 --release --example rpi_verify_geometric
```

### Spatial Engine (Vector3 Distance)
```bash
# x86_64 (Isolated Core 10, Ultra-Lean)
taskset -c 10 cargo run -p geom-spatial --release --example spatial_bench

# x86_64 (Unlocked)
cargo run -p geom-spatial --release --example spatial_bench
```

---

## Benchmark Output Format

All harnesses report in the following format:
```
Engine: X.XXX MHz | [Additional Metrics]
```

- **MHz:** Millions of operations per second (sample ticks or distance calculations).
- **Additional Metrics:** May include predicted scope frequency or voice count.

---

## Hardware Conformance

These benchmarks are **not** part of the normative Reference Baseline requirements. They serve as:
1. **Reference Expectations:** Guide deployment sizing (e.g., "Can I run 20 voices on a Pi?").
2. **Regression Detection:** Ensure optimizations don't degrade performance.
3. **Architectural Insight:** Quantify the "Cost of Geometry" for spatial synthesis.

For **correctness verification**, see `VERIFICATION_GUIDE.md`.

---

**Last Updated:** 2026-02-03 (Benchmark hygiene + documentation correction for Criterion throughput semantics)
