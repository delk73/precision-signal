# Chaos Probes (Advisory, Non-Normative)

This document defines advisory chaos probes for the Signal Core. These probes are diagnostic only and do not participate in release gating.

Normative release gating remains:
- `cargo run --release -p dpw4 --features cli --bin sig-util -- validate --mode quick`

The chaos audit emits machine-readable metrics:
- `chaos_metrics.json`

## Execution

PR mode:

```bash
cargo run --release -p dpw4 --example chaos_audit --offline -- --mode pr
```

Nightly mode:

```bash
cargo run --release -p dpw4 --example chaos_audit --offline -- --mode nightly
```

Optional output path:

```bash
cargo run --release -p dpw4 --example chaos_audit --offline -- --mode pr --out chaos_metrics.json
```

## Probe Set

1. Sub-Hz Accumulator / Phase Quantization Probe (Saw)
- Parameters: `SR=48000`, `f=1e-7`, `N=1_000_000` (`pr`)
- Metrics:
  - `non_monotonic_phase_steps`
  - `phase_u32_stutter_steps`
  - `max_abs_residual`
  - `residual_unit`
- `max_abs_residual` is reported in the explicit unit:
  - `abs((raw_q124*2^-124*(1/24)) - f64_shadow_dpw4)`

2. Nyquist Corner Stress (Triangle)
- Parameters: `SR=48000`, `f=23999`, `N=48000` (`pr`)
- Metrics:
  - `min_sample`, `max_sample`
  - `dc_offset_samples`
  - `even_odd_abs_peak_delta`

3. CORDIC Gain Hunt (Sine)
- Parameters: `SR=48000`, `f=1000`, `N=48000` (`pr`)
- Metrics:
  - `hit_i32_max_count`, `hit_i32_min_count`
  - `max_abs_normalized`
  - `clipping_active_count` (`norm > 1.0`)

4. Pulse Symmetry Diagnostic
- Parameters: `SR=48000`, `f=1000`, `N=48000` (`pr`)
- Metrics:
  - `nonzero_diff_count`, `max_abs_diff_lsb`
  - `nonzero_sum_count`, `max_abs_sum_lsb`

## CI Policy

- Advisory job name: `chaos_probes_advisory`
- Always runs on push and pull requests.
- Uploads `chaos_metrics.json` as a workflow artifact.
- Job is non-gating for repository status.
- Job execution failures (build/run/metrics missing) are reported in that job, but do not block normative release gating.
