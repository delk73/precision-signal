# Hot Path Execution and Benchmarking Plan

**Document revision:** v1.0.0-rc5  
**Applies to:** historical pre-release planning material (not current release 1.2.2)  
**Status:** Historical reference  
**Scope:** Signal / geometry hot paths (inner loops)  
**Related:** CONTROL_SCHEDULER_BENCHMARKING.md

## Versioning Terminology

- Document revision labels the historical planning snapshot captured in this file.
- Release versions identify shipped software releases.
- This file is historical roadmap/reference material and does not define the current release `1.2.2` surface.

> **Implementation Status (v1.0.0-rc5)**
>
> This document describes both:
>
> - the **current hot-path execution and benchmarking state**, and
> - a **planned architectural pattern** for future control-plane integration.
>
> As of v1.0.0-rc5:
>
> - Hot-path kernels (DPW, trig, geometry) are fully implemented.
> - Kernel-level microbenchmarks exist.
> - No control scheduler, ControlPlane, or telemetry-driven policy layer
>   is currently implemented.
>
> All control-plane references in this document are **forward-looking**
> and do not describe current runtime behavior.

---

## 1. Purpose

This document defines:

1. What constitutes the **hot path** in this repository.
2. How hot-path execution is architecturally isolated from the control scheduler.
3. The **current benchmarking state** for hot-path code.
4. Identified gaps in existing benchmarks.
5. A grounded plan to align hot-path and control-plane benchmarking into a
   coherent, non-overlapping process.

The intent is to prevent benchmark drift, misinterpretation, and accidental
coupling between control logic and deterministic inner loops.

---

## 2. Definition: Hot Path (Normative)

A **hot path** is any code that:

- Executes at **fixed, high frequency** (audio-rate, render-rate, or per-sample).
- Runs inside a **tight loop** with no dynamic allocation.
- Must meet **hard real-time or quasi–real-time budgets**.
- Is expected to be **bit-exact deterministic** across architectures.
- Is insensitive to control-plane jitter.

Examples:
- DPW kernels (DPW2, DPW4, blends).
- Trigonometric primitives (`sin_cos`, `atan2`, `sqrt`) used in signal evolution.
- Geometry update steps executed per sample or per render step.

Non-examples:
- Mode selection.
- Telemetry sampling.
- Logging, scrubbing, calibration.
- Any code that can be skipped, deferred, or rate-limited.

---

## 3. Architectural Separation (Planned Invariant)

> **Planned Architecture**
>
> The separation described in this section defines the **intended target
> architecture**. It is not yet implemented in the v1.0.0-rc5 baseline.
>
> The current system invokes hot-path kernels directly (e.g. via
> `Oscillator::tick()` and DPW step functions) without a mediating
> control-plane scheduler.

### 3.1 Control-plane vs hot-path contract

- The **control scheduler** publishes a latched `ControlPlane`.
- Hot paths **only read** latched values.
- No hot-path code:
  - reads live telemetry,
  - evaluates policy,
  - performs environment-dependent branching beyond a mode enum or scalar.

This separation is mandatory.

---

## 4. Current Hot-Path Benchmarking State

### 4.1 Existing benchmarks

Currently present in `crates/geom-signal`:

- `benches/trig_throughput.rs`
  - `sin_cos`
  - `sqrt`
  - `atan_shafer`
  - `atan2_shafer`

Characteristics:
- Benchmarks primitives **in isolation**.
- Measures per-call latency (ns/op).
- Uses Criterion on Linux hosts (Pi, x86).
- Suitable for:
  - order-of-magnitude comparison,
  - regression detection >~10%,
  - relative ranking between primitives.

---

### 4.2 What current benchmarks do well

- Validate that kernels are:
  - fast enough to be viable,
  - stable across architectures,
  - not catastrophically regressing.
- Provide early warning when algorithmic complexity changes.
- Give realistic upper bounds for worst-case kernel cost.

---

### 4.3 Limitations of current benchmarks (Gaps)

The current benchmarks **do not**:

1. Represent **actual hot-path workload mixes**
   - Real firmware does not call `sin_cos` or `atan2` in isolation.
   - Dependency chains, register pressure, and cache behavior are not captured.

2. Capture **per-sample or per-block execution**
   - No notion of per-sample or per-block cycle counts (linux jitter).
   - Metrics are limited to advisory nanoseconds on desktop/linux environments.
   - Hard to compare directly against real-time chip-cycle budgets.

3. Reflect **latched mode behavior**
   - Benchmarks do not account for:
     - Normal vs degraded paths,
     - alternate kernels selected by mode.

4. Provide **deterministic cycle counts**
   - Linux jitter, turbo, and scheduler noise dominate small deltas.
   - Absolute numbers (nanoseconds) are advisory, not contractual.
   - Cycle-accurate measurement is deferred to bare-metal characterization.

5. Distinguish **kernel benchmarking** from **system throughput**
   - Leads to misinterpretation when compared with control-plane cost.

---

## 5. Hot-Path Benchmarking Categories (Normative)

Hot-path benchmarking is divided into **two non-overlapping categories**.

---

### 5.1 Kernel microbenchmarks (existing, retained)

**Purpose:**
- Measure individual primitive cost.
- Compare algorithm variants.
- Detect catastrophic regressions.

**Examples:**
- `sin_cos` throughput
- `atan2_shafer` vs alternatives

**Rules:**
- Isolated calls only.
- No scheduler involvement.
- Results are advisory, not acceptance-gated.

---

### 5.2 Inner-loop throughput benchmarks (present)

**Purpose:**
- Measure **nanoseconds per sample** or **nanoseconds per block** for real workloads.
- Characterize hot-path execution patterns on Linux (x86, ARM) platforms.

**Characteristics:**
- Criterion-controlled iteration count; inner loop body is fixed and allocation-free.
- Fixed call sequence.
- No control-plane logic inside loop.
- Mode and parameters latched before entering loop.

**Implementation:** `crates/geom-signal/benches/signal_inner_loop.rs`

**Benchmarked workloads:**
- **DPW4 Sawtooth sample step**: `tick_dpw4` with pre-latched gain, incrementing phase
- **DPW4 Pulse sample step**: `tick_shape(shape = PULSE)` with 50% duty
- **Sine sample step**: `tick_shape(shape = SINE)` (CORDIC + i32 scaling)
- **DPW4 Sawtooth block**: 128-sample block rendering

These benchmarks are the **bridge** between kernel cost and real-time feasibility.

> [!NOTE]
> **Numeric budgets not set**: This benchmarking infrastructure produces the data
> needed to define budgets, but does not establish acceptance criteria.
> Budget definition is deferred to future work.
>
> **Units Note**: Benchmarks currently report **nanoseconds per sample** (ns/sample).
> Cycle-accurate measurement (cycles/sample) is a non-goal for this Linux-based pass.

---

## 6. Alignment With Planned Control Scheduler Benchmarks

> **Status Note**
>
> Control-scheduler benchmarks do not yet exist. This section defines
> how hot-path benchmarks are expected to align with control-plane
> benchmarks once the control scheduler described in
> [docs/architecture/performance/CONTROL_SCHEDULER_BENCHMARKING.md](CONTROL_SCHEDULER_BENCHMARKING.md) is implemented.

### 6.1 Separation of concerns (hard rule)

| Plane | Bench Focus | Frequency |
|---|---|---|
| Control | `tick()` cost, policy behavior | 100–1000 Hz |
| Hot path | ns/sample or ns/block (advisory) | 10 kHz–50 kHz |

No benchmark should attempt to measure both simultaneously.

---

### 6.2 Shared structure (alignment)

Both benchmarking plans share:
- Deterministic inputs.
- Coarse acceptance bins.
- Clear scope boundaries.
- No reliance on live system state.

This allows:
- Independent evolution.
- Comparable reasoning ("does this fit budget?").
- Clean audits.

---

## 7. Acceptance Philosophy for Hot Paths

### 7.1 What matters

- Does the hot loop fit inside the **real-time budget** with margin?
- Is cost stable across architectures?
- Are alternate modes bounded?

### 7.2 What does not matter

- ±1–3% desktop noise.
- Criterion p-values for microbenchmarks.
- Cross-machine absolute ns comparisons.

---

## 8. Current State and Remaining Gaps

### 8.1 Implemented (v1.0.0-rc5)

1. **Inner-loop throughput benchmarks** ✓
   - `crates/geom-signal/benches/signal_inner_loop.rs`
   - DPW4 sawtooth, pulse, sine sample steps
   - DPW4 sawtooth block rendering (128 samples)

2. **Explicit workload definitions** ✓
   - Sample-step benchmarks for canonical hot paths
   - Block-level benchmark for buffer rendering

### 8.2 Remaining Gaps (Deferred)

1. **Mode-latched benchmarking** (awaits control scheduler)
   - Normal / Degraded / Alternate kernel selection
   - Cannot implement until control-plane exists

2. **Documented budgets** (awaits platform characterization)
   - ns/sample or ns/block (platform- and workload-specific, advisory)
   - Requires extended characterization on target hardware
   - This pass produces the data; budget definition is future work

Kernel microbenchmarks remain unchanged and complement inner-loop benchmarks.

---

## 9. Repository Layout

### 9.1 Current Layout (v1.0.0-rc5)

```
docs/
  CONTROL_SCHEDULER_BENCHMARKING.md
  HOT_PATH_EXECUTION_AND_BENCHMARKING.md

crates/geom-signal/
  benches/
    trig_throughput.rs
  src/
    lib.rs
    math.rs
    algebraic.rs
    verification.rs

crates/dpw4/
  src/
    lib.rs
    verification.rs
```

### 9.2 Current Layout (v1.0.0-rc5, after this pass)

```
docs/
  CONTROL_SCHEDULER_BENCHMARKING.md
  HOT_PATH_EXECUTION_AND_BENCHMARKING.md
  DESIGN_AXIOMS.md

crates/geom-signal/
  benches/
    trig_throughput.rs          # kernel microbenchmarks (existing)
    signal_inner_loop.rs        # inner-loop throughput benchmarks (present)

crates/dpw4/
  src/
    lib.rs
```

### 9.3 Target Layout (Planned)

```
crates/control-scheduler/
  src/
    telemetry.rs
    margin.rs
    policy.rs
    scheduler.rs
    control_plane.rs
  benches/
    control_scheduler.rs
  tests/
    control_policy.rs
```

---

## 10. Change Control

Any change that:
- increases hot-path inner-loop cost, or
- alters which kernels run in a given mode

must be evaluated against:
- inner-loop benchmarks (this document), and
- control-plane benchmarks (paired document).

---

## 11. Summary

- Hot paths and control schedulers are **intentionally decoupled**.
- Current benchmarks cover kernels but **not real workloads**.
- This document defines the missing layer needed to reason about
  real-time feasibility.
- Together with the control scheduler plan, this forms a complete,
  grounded benchmarking framework.
