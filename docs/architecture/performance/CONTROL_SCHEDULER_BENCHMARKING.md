# Control Scheduler Benchmarking Plan

**Status:** Normative  
**Scope:** Control-plane (outside-control) scheduler only  
**Out of Scope:** Hot-path signal/DSP kernel benchmarking

> **Implementation Status (v1.0.0-rc5)**
>
> This document defines the **planned** control-scheduler benchmarking framework.
> As of v1.0.0-rc5, no control scheduler, ControlPlane, telemetry traces, or
> control-policy benchmarks are implemented. All references to scheduler code,
> traces, and tests are forward-looking requirements.

---

## 1. Purpose

This document defines the **canonical benchmarking plan** for the condition-aware
control scheduler. The scheduler operates **outside** all hot signal/render loops
and governs operating modes, parameters, and maintenance work admission based on
system telemetry.

The goals of this plan are to:

- Detect **catastrophic regressions** in control-plane cost.
- Provide **repeatable, deterministic** benchmarks via telemetry trace replay.
- Validate **policy behavior** (mode transitions, thrash bounds, starvation bounds).
- Preserve portability to **bare-metal** targets.

Micro-optimizations and sub-percent variance are explicitly non-goals.

---

## 2. Architectural Context

### 2.1 Control-plane role

The control scheduler:

- Samples or receives system telemetry.
- Computes an **OperatingMargin** scalar.
- Applies policy (hysteresis, cooldown).
- Selects operating **Mode**.
- Publishes a latched **ControlPlane** for consumption by hot loops.
- Admits or defers bounded maintenance work (logging, scrubbing).

The scheduler **never executes** inside per-sample or per-render inner loops.

> **Note:** This role description is planned; v1.0.0-rc5 does not implement this scheduler.

---

## 3. Terminology (Normative)

| Term | Definition |
|---|---|
| **SystemTelemetry** | Snapshot of environmental/system state (temperature, voltage, vibration proxy, cycle count, etc.) |
| **OperatingMargin** | Bounded scalar representing suitability for precision operation |
| **Mode** | Discrete operating mode selected by policy |
| **ControlPlane** | Atomically published, latched parameters consumed by hot loops |
| **Tick** | One invocation of the control scheduler |
| **Trace** | Deterministic sequence of `SystemTelemetry` samples |

---

## 4. Core Functions (Contractual)

### 4.1 Margin evaluation

```rust
compute_operating_margin(SystemTelemetry) -> OperatingMargin
```

**Requirements:**

* Pure function (no side effects).
* Deterministic.
* Bounded output range (e.g. `0..=10000`).
* Monotonic penalties with respect to defined telemetry dimensions.
* No allocation; bounded execution time.

---

### 4.2 Policy decision

```rust
decide_mode(OperatingMargin, PolicyState) -> Mode
```

**Requirements:**

* Deterministic given inputs and prior state.
* Enforces hysteresis and cooldown.
* Prevents unbounded mode thrashing.
* No allocation; bounded execution time.

---

### 4.3 Scheduler tick

```rust
tick(SystemTelemetry) -> ControlPlane
```

**Responsibilities:**

* Evaluate margin.
* Update policy state.
* Select mode.
* Publish a new `ControlPlane`.
* Perform bounded queue management.

---

## 5. Benchmark Layers

Benchmarks are explicitly divided into **three layers** and MUST NOT be conflated.

---

### 5.1 Bench A — Margin evaluation

**Name pattern:**
`control/compute_operating_margin/<trace>`

**Measures:**

* Cost of `compute_operating_margin()` over a telemetry trace.

**Purpose:**

* Detect accidental complexity growth in scalar evaluation logic.
* Provide a stable baseline independent of policy behavior.

---

### 5.2 Bench B — Policy decision

**Name pattern:**
`control/decide_mode/<trace>`

**Measures:**

* Policy evaluation cost given a precomputed margin stream.

**Purpose:**

* Measure branch/state-machine complexity.
* Validate hysteresis and cooldown overhead in isolation.

---

### 5.3 Bench C — Full scheduler tick

**Name pattern:**
`control/tick/<trace>`

**Measures:**

* End-to-end control tick cost:
  telemetry → margin → policy → publish → bounded queues.

**Purpose:**

* Represent true supervisor overhead in firmware.

---

## 6. Telemetry Traces (Deterministic Inputs)

All benchmarks operate on **deterministic trace replay**.

### 6.1 Required trace set

| Trace             | Description                                    |
| ----------------- | ---------------------------------------------- |
| **TRACE_NOMINAL** | Stable operation with slow thermal drift       |
| **TRACE_STRESS**  | Periodic voltage sag and threshold crossings   |
| **TRACE_THRASH**  | Oscillation near thresholds to test hysteresis |
| **TRACE_FAULT**   | Step changes to extreme conditions             |

### 6.2 Trace format

* Stored as `&'static [SystemTelemetry]`.
* No dynamic allocation.
* Length: 1k–16k samples.
* Same traces used for benchmarks and behavior tests.

---

## 7. Behavioral Metrics (Acceptance Gates)

These metrics are **normative** and must be enforced via tests.

### 7.1 Mode behavior

* Mode residency distribution.
* Total transition count.
* Maximum transition rate.
* Minimum dwell time per mode.

### 7.2 Stability constraints

* No unbounded thrashing under `TRACE_THRASH`.
* Cooldown invariants respected.
* Deterministic outcomes for identical traces.

### 7.3 Work admission

* Bounded queue sizes.
* Maximum wait (starvation) bounded by policy contract.

---

## 8. Performance Acceptance Criteria

### 8.1 Coarse thresholds only

Performance is evaluated in **bins**, not percent deltas.

Examples:

* Median `tick()` cost ≤ configured budget.
* Worst-case `tick()` cost ≤ hard ceiling.
* No regression exceeding predefined bin boundaries.

Exact numeric budgets are platform- and configuration-specific and are
defined separately.

---

## 9. Measurement Environments

### 9.1 Linux (development)

* Criterion-based benchmarks.
* Report ns/tick.
* Deterministic traces reduce variance.
* Small deltas (<~5%) are informational only.

### 9.2 Bare metal (target)

* Same benchmark loops.
* Hardware cycle counters.
* Report cycles/tick.
* Identical acceptance gates.

---

## 10. Repository Layout

### 10.1 Current Layout (v1.0.0-rc5)

```
docs/
  CONTROL_SCHEDULER_BENCHMARKING.md
  HOT_PATH_EXECUTION_AND_BENCHMARKING.md
```

No `src/control/`, `benches/control_scheduler.rs`, or `tests/control_policy.rs` currently exist.

### 10.2 Target Layout (Planned)

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

## 11. Explicit Non-Goals

* Benchmarking DSP, trig, or geometry kernels.
* Micro-optimizing scheduler cost below practical significance.
* Using live telemetry in benchmarks.
* Introducing dynamic allocation or unbounded loops.

---

## 12. Change Control

Once implemented, any change that:

* modifies scheduler cost characteristics, or
* alters policy behavior under defined traces

**must** be evaluated against this benchmarking plan.

This document defines the **only valid interpretation** of
control-scheduler benchmarking for this repository.
