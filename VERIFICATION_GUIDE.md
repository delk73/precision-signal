# precision-signal: Canonical Verification Protocol
**Version: 1.7.0 (Active Release Baseline)**
**Status: Frozen Definition**

## Purpose

This guide defines how to verify that an implementation of the deterministic execution analysis infrastructure matches the reference. Verification uses cryptographic hashes, formal proofs, and physical measurements to evaluate bit-exact determinism under the pinned environment and defined verification gates.

## Release Contract

This document is the canonical release contract for `precision-signal`.

Release readiness for a retained repository release record requires:

- retained Kani evidence from the manual preflight `bash verify_kani.sh` to exist under `docs/verification/releases/1.7.0/`
- the canonical operator-facing release gate `make gate` to pass
- the canonical `1.7.0` release-record orchestration `make release-1.7.0` to pass
- the retained release evidence bundle to live under `docs/verification/releases/<version>/`
- `make release-bundle-check VERSION=<version>` to pass against that retained bundle

For release `1.7.0`, run the pre-tag path in this order:

1. `bash verify_kani.sh`
2. `make gate`
3. `make release-1.7.0`
4. `make release-bundle-check VERSION=1.7.0`

Nested-target note for `1.7.0`: `make release-1.7.0` already reruns `make gate`,
`make doc-link-check`, and `make release-bundle-check VERSION=1.7.0` while
recording retained outputs under `docs/verification/releases/1.7.0/`. The
ordered path above remains the operator-facing pre-tag sequence for this
release; the duplicated runs come from the existing orchestration and do not
change the requirement set.

Not part of the active `1.7.0` pre-tag contract:

- `make gate-full` is supplementary validation only
- manual STM32 validation and hardware support procedures are supporting/reference material, not release authority
- changelog timing is not a release-admissibility requirement in this guide

For release-surface questions, use this guide as the source of truth for:

- what must be true for release readiness
- what command is canonical
- where retained release evidence lives

For release `1.7.0`, reviewers should traverse this path: `bash verify_kani.sh`
for the manual once-per-release preflight, `make release-1.7.0` for the
canonical retained-record orchestration, [docs/replay/tooling.md](docs/replay/tooling.md)
for released replay-tooling boundaries, and `docs/verification/releases/1.7.0/`
for the retained release evidence bundle. Historical retained evidence remains explicit under
`docs/verification/releases/`.

Pre-split historical evidence may still refer to `precision` as the validation
binary; the current validation entrypoint is `sig-util -- validate`, which implements `make gate`.

The rest of the release-adjacent documentation is supporting only:

- `README.md`: entry routing
- `docs/RELEASE_SURFACE.md`: release-surface classification and routing
- `docs/verification/build_reproducibility.md`: supporting explanation for pinned builds and dual-build identity checks
- `docs/verification/CI_EVIDENCE.md`: historical CI evidence, not the release contract
- `docs/verification/hardware_procedures.md`: manual hardware support procedures, not release authority
- retained files under `docs/verification/releases/<version>/`: evidence for a specific release once generated

---

## 1. Governance & Authority

This document defines the **only** valid interpretation of "precision-signal conformance" for the `precision-signal` repository. To prevent interpretation drift, all signals produced or audited by this system are classified as either **Normative** or **Advisory**.

### 1.1 Normative vs. Advisory Signals

| Signal | Authority | Role | Failure Consequence |
| --- | --- | --- | --- |
| **SHA-256 Hashes** | **Normative** | The absolute definition of correctness. | **Immediate Rejection** |
| **Formal Proofs** | **Normative** | Kani symbolic execution of core kernels within stated harness assumptions. | **Non-Conformance** |
| **Phase Invariants** | **Normative** | Phase Engine locked to Scalar addition / O(1) Modulo. | **Non-Conformance** |
| **Pinned Toolchain** | **Normative** | The only valid execution environment (`1.91.1`). | **CI Failure** |
| **`libm` Shadow Model** | *Advisory* | Sanity check for drift and residual observation. | **Audit Warning** |
| **DC Offset Metrics** | *Advisory* | Empirical performance characteristic. | **Audit Warning** |

### 1.2 Reference Governance
* **Hash Regeneration**: Hashes may **only** be regenerated during a formal Semantic Versioning bump (minor or major). Regeneration without a version bump is strictly forbidden.
* **128-bit Bedrock**: All implementations must use the **I64F64** fixed-point representation for the Phase Engine and Core Math.
* **Phase-Domain Precision**: The math core supports long-duration phase evolution for the repository's pinned fixed-point model, with no drift indicated by the current reference arithmetic and validation surfaces, implementing `sin`, `cos`, and `sqrt` for `I64F64`. Spatial magnitude operations saturate at ~2.8×10¹⁴ meters.

---

## 2. Pinned Environment

To ensure bit-exact forensic compatibility, the Reference Baseline is locked to the following environment:
- **Compiler**: `rustc 1.91.1` (Enforced via `rust-toolchain.toml`)
- **Arch Class**: 64-bit Word Size (ARM64, x86_64)
- **Core Precision**: 128-bit `Scalar` (`I64F64`) bedrock.
- **Egress Width**: 32-bit (S32LE).

---

## Core Float Quarantine

Quarantine is satisfied if and only if:

```bash
cargo check --workspace --no-default-features
cargo check -p dpw4 --no-default-features --target thumbv7em-none-eabihf
```

succeed without enabling `float-ingest`.

In this repo's governance, enforcement is build-surface based.
Enforcement is not token-based.
Enforcement is not grep-based.
`#[cfg(test)]` is outside normative surface.
CLI is outside core surface (release validation uses it, but it is not part of the core crates).

---

## 3. Protocol Level -1: Mathematical Foundation (Normative)

Objective: Use formal methods (Kani) to prove that core mathematical and DSP kernels are panic-free and robust.

### 3.1 Canonical Runner (Tiered)
Use the repository runner for deterministic CI behavior and normative token validation.

```bash
# Tier-1 (default): fast harnesses only
bash verify_kani.sh

# Tier-2 (heavy): includes atan2 shard proofs (q1-q4)
RUN_HEAVY=1 bash verify_kani.sh
```

Normative evidence boundary: the harness manifest embedded in
`verify_kani.sh` is the authoritative runner surface for formal-verification
claims in this repository. Harnesses present in source but omitted from that
manifest are implementation inventory, not normative runner evidence.

### 3.2 Environment Controls
```bash
# Conservative shard cap: min(online cores, DEFAULT_MAX_JOBS)
# Recommended Env defaults:
#   DEFAULT_MAX_JOBS=4 on >=16 GB RAM runners
#   DEFAULT_MAX_JOBS=2 on memory-constrained runners

# Keep logs on success (on failure they are always retained)
KEEP_LOGS=1 bash verify_kani.sh
```

### 3.3 Success Criteria
- **Implementation**: `dpw4` boundary kernels and `geom-signal` primitives.
- **Harnesses**: the manifest-defined subset executed by `verify_kani.sh`
  for Tier-1, with additional manifest-defined Tier-2 harnesses when
  `RUN_HEAVY=1`.
- **Status**: Each per-harness log must contain `VERIFICATION:- SUCCESSFUL` and must not contain `** N of M failed` where `N > 0`.
- **Implication**: Provides panic-safety and invariant evidence for the kernels covered by these harnesses and their assumptions. The active release-scoped proof boundary and exclusions must be read from `docs/verification/releases/1.7.0/`.

### 3.4 Harness-to-Crate Mapping
| Harness | Crate | Tier |
| --- | --- | --- |
| `proof_compute_x2_safe` | `dpw4` | Tier-1 |
| `proof_saturate_safe` | `dpw4` | Tier-1 |
| `proof_phase_u32_no_overflow` | `dpw4` | Tier-1 |
| `proof_phase_u32_fixed_to_u32_conversion` | `dpw4` | Tier-1 |
| `proof_sine_scale_no_overflow` | `dpw4` | Tier-1 |
| `proof_sine_to_i32_in_range` | `dpw4` | Tier-1 |
| `proof_sine_egress_bounded` | `dpw4` | Tier-1 |
| `proof_triangle_delta_clamp_identity_when_in_range` | `dpw4` | Tier-1 |
| `proof_triangle_delta_clamp_saturates_when_out_of_range` | `dpw4` | Tier-1 |
| `proof_triangle_z_update_is_saturating` | `dpw4` | Tier-1 |
| `proof_i256_sub_matches_spec` | `dpw4` | Tier-1 |
| `proof_i256_sar_in_range_matches_spec` | `dpw4` | Tier-1 |
| `proof_i256_sar_out_of_range_matches_spec` | `dpw4` | Tier-1 |
| `proof_i256_clamp_matches_spec` | `dpw4` | Tier-1 |
| `proof_spec_clamp_in_range_contract` | `dpw4` | Tier-1 |
| `proof_spec_clamp_out_of_range_contract` | `dpw4` | Tier-1 |
| `proof_spec_sar_sanity` | `dpw4` | Tier-1 |
| `proof_triangle_freeze_invariant` | `dpw4` | Tier-1 |
| `proof_triangle_freeze_egress_invariant` | `dpw4` | Tier-1 |
| `proof_sqrt_no_panic` | `geom-signal` | Tier-1 |
| `proof_sin_cos_no_panic` | `geom-signal` | Tier-1 |
| `proof_v0_wire_size_constants` | `replay-core` | Tier-1 |
| `proof_encode_header0_wire_layout_and_le` | `replay-core` | Tier-1 |
| `proof_encode_event_frame0_wire_layout_and_le` | `replay-core` | Tier-1 |
| `proof_atan2_q1` | `geom-signal` | Tier-2 |
| `proof_atan2_q2` | `geom-signal` | Tier-2 |
| `proof_atan2_q3` | `geom-signal` | Tier-2 |
| `proof_atan2_q4` | `geom-signal` | Tier-2 |

### 3.5 Diagnostics and Interpretation
- **Per-harness logs**: Stored under `kani_logs/<package>__<harness>.log`.
- **Retention policy**: Logs are always kept on failure. On success they are deleted unless `KEEP_LOGS=1`. For `RUN_HEAVY=1`, success logs default to kept unless explicitly overridden.
- **Output format**: Runner uses `--output-format terse` for all harnesses to improve CI readability.
- **Discovery-only (manual, non-normative)**: For Kani 0.67.0, run discovery from the crate directory with target options before `list` (no `-p`):
  - `cd crates/geom-signal && cargo kani --lib list`
  - `cd crates/dpw4 && cargo kani --lib list`
  (`cargo kani list --lib` is rejected by this Kani version.)
  This is optional local harness discovery only. It is not required by the canonical runner (`bash verify_kani.sh` / `RUN_HEAVY=1 bash verify_kani.sh`), may fail in some workspace/feature-gated-bin (`required-features`) layouts, and is not normative evidence. Only runner logs and success token checks are normative evidence.
- **"dereference failure ... Status: SUCCESS" lines**: These indicate Kani proved the failing path unreachable under harness constraints; they are successful checks, not proof failures.

### 3.6 Release-Scoped Correctness and Limits
- The active release (`1.7.0`) retains its release-scoped correctness claims and limits under `docs/verification/releases/1.7.0/`.
- That retained `1.7.0` bundle is a narrowed, non-firmware release record for the primary precision CLI surface only, scoped to `crates/dpw4/src/bin/common/mod.rs`, `crates/dpw4/src/bin/precision/mod.rs`, and `crates/dpw4/tests/precision_authoritative_surface.rs`, within the explicit limits documented in the `1.7.0` release directory.
- That claim is exercised-path and release-scoped, not global.
- Heavy Tier-2 proofs remain optional unless the active release bundle explicitly retains a heavy proof run. If omitted, the retained release bundle must state the exclusion and the remaining release-claim boundary explicitly.

---

## 4. Observation Level A: The Physical Layer (Normative)

Observation levels describe measurement layers, not protocol layers. PWM is an adapter-level choice on Raspberry Pi hardware and is not part of the precision-signal reference protocol.

Objective: Validate that the **128-bit Scalar Phase Engine** maps correctly to physical time.

### 4.1 Execution
```bash
sudo taskset -c 3 cargo run -p dpw4 --release --example rpi_verify_logic
```

### 4.2 Success Criteria
- **Scope Reading**: $440.0\,\text{Hz} \pm 0.5\%$.
- **Waveform**: 50% Duty Cycle Square (derived from Phase MSB).
- **Implication**: Validates the frequency-to-increment mapping of the Scalar engine.

---

## 5. Protocol Level 1: The Golden Lock (Normative)

Verification of the core kernel is performed using SHA-256 fingerprinting of raw 32-bit signal traces. While the underlying math is now 128-bit, it is mathematically mapped to maintain bit-parity with the legacy reference output.

### 5.1 Forensic Hashing Policy
To eliminate ABI padding noise and platform-specific structure alignment, hashes are computed exclusively on **serialized Little-Endian bytes** (`.to_le_bytes()`) of the signal output. Hashing raw struct memory or `&self` pointers is strictly prohibited.

### 5.2 Reference Canonical Hash (440Hz @ 44.1kHz, 10k samples)

Transport Golden Lock is intentionally **Saw-only** in v1.0 hardened baseline. The core forensic audit gate is **Saw + Pulse** and remains core-only (`cargo test -p dpw4 --test forensic_audit`).

| Waveform | SHA-256 Fingerprint |
| --- | --- |
| **Sawtooth (Transport Golden Lock)** | `3ec8a3eb464342e551b99902490121110de5c069fb7c95d49796a85b299eb44e` |

### 5.3 Execution
```bash
cargo test -p dpw4 --test forensic_audit
```
This gate is core-only and requires no optional feature flags.
**Expected Result**: `test_golden_lock ... ok`  
**Validation**: Provides hash-locked evidence that the differentiator kernel output matches the Reference Baseline for this audited path.

Any modification to `HEADROOM_BITS` requires:
- Regeneration of forensic golden hashes
- Re-evaluation of absolute magnitude assertions
- Passing the `forensic_audit` CI gate

### 5.4 CORDIC Sine Integrity (Normative)
The CORDIC implementation must be bit-aligned with the Phase Engine.
- **Success Criteria**: `test_sine_saw_phase_sync` passes with residual `<= 1000` units at `pi`.
- **Implication**: Provides evidence that the Geometric path and the legacy DPW path remain phase-aligned under this test.
- **Egress Policy**: Sine output must honor `HEADROOM_BITS` at egress (no bypass of container headroom policy).

---

## 6. Protocol Level 2: Determinism Validation (Normative)

Bit-identical execution across runs and platforms is a core requirement of the Baseline.

### 6.1 Canonical Gate Command
```bash
make gate
```
This is the canonical operator-facing release gate.
It runs `cargo run --release -p dpw4 --features cli --bin sig-util -- validate --mode quick`.
Exit code is authoritative (`0` pass, `1` fail).
`--mode full` is currently identical to `--mode quick` and reserved for future extension.
If explicitly exercised, use:

```bash
make gate-full
```

`make gate-full` is supplementary validation only. It does not replace `make gate`
as the canonical release gate and should be retained separately when used.
For retained release evidence, archive the release-ready gate record under
`docs/verification/releases/<version>/`.

### 6.2 Normative Determinism Hash Source of Truth

The normative `.det.csv` baseline hashes are maintained in code at
`crates/dpw4/src/bin/precision.rs` via `NORMATIVE_DET_HASHES`.

This guide does not duplicate the active hash table. The authoritative review
path is:

- inspect the current `NORMATIVE_DET_HASHES` definition for the active release
  source of truth
- retain the command transcript from `make gate` in
  `docs/verification/releases/<version>/`

Historical retained transcripts may quote the hashes observed for a specific
release, but those retained copies are historical evidence, not the active
normative table.

### 6.3 Supported Entry Surface
Use `make gate` for routine operator execution of the quick validation gate.
The underlying normative command is `sig-util validate --mode quick`.
No other command is an equally authoritative release-admissibility gate.

### 6.4 Release Evidence Location

The canonical retained release-evidence location is:

```text
docs/verification/releases/<version>/
```

For a release reviewer, this is the directory to inspect for the retained
release record. Supporting checks may exist elsewhere while they run, but
retained release evidence must be anchored here if it is part of the release
decision.

### 6.5 Retained Release Record Requirements

For a retained repository release record under
`docs/verification/releases/<version>/`, the required evidence set is:

- Non-firmware retained release bundle:
  - `README.md`
  - `index.md`
  - `cargo_check_dpw4_thumb_locked.txt`
  - `kani_evidence.txt`
  - `make_demo_evidence_package.txt`
  - `make_doc_link_check.txt`
  - `make_gate.txt`
  - `make_replay_tests.txt`
  - `release_reproducibility.txt`
- Firmware-including retained release bundle:
  - `firmware_release_evidence.md`
  - `sha256_summary.txt`
  - `hash_check.txt`
  - `replay_manifest_v1.txt` for the active RPL0 `version = 1` capture path
  - `replay_manifest_v0.txt` only for historical retained bundles captured
    before the `v1` manifest transition

For the active RPL0 `version = 1` firmware capture path, the retained
`replay_manifest_v1.txt` record must preserve the current release-defining
metadata actually archived by the firmware-release workflow:

- `contract_version`
- `artifact_version`
- `schema_hash`
- `signal_model`
- current baseline metadata, including `baseline_path` and `baseline_sha256`
- release-run summary fields needed to interpret the retained record
  (`requested_runs`, `completed_runs`, `final_status`, `failure_class`,
  `baseline_hash_match`, `timestamp_utc`, `run_dir`)

Additional release-specific outputs may be retained alongside either bundle
class, but they are additive. They do not replace the class-specific required
files above.

Reviewer sequence for a retained release bundle: inspect the retained evidence
summary in `docs/verification/releases/<version>/`, then run
`make release-bundle-check VERSION=<version>` against that same directory.

### 6.6 Non-Normative Canary Scenario

`phase_wrap_440` is a non-normative determinism canary.

- It is generated during validation artifact production.
- It must remain internally deterministic across repeated runs.
- It is not part of the normative hash-locked release baseline.
- Its WARN status is informational and does not contribute to release
  acceptance.

---

## 7. Protocol Level 3: Transport & Header (Normative)

Verification of the **DP32 Protocol** for infrastructure-grade transport.

### 7.1 SignalFrameHeader Schema (64 Bytes)

| Offset | Size | Field | Value / Description |
| --- | --- | --- | --- |
| 0 | 4 | `magic` | `b"DP32"` |
| 4 | 4 | `version` | Protocol version (Always 1) |
| 8 | 8 | `sequence` | Monotonic u64 count |
| 16 | 4 | `sample_rate` | u32 Sample Rate (Hz) |
| 20 | 4 | `bit_depth` | u32 Bit Depth (Always 32) |
| 24 | 32 | `pad` | Zero-filled padding (`HEADER_PAD_SIZE = 32`) |
| 56 | 4 | `reserved` | Must be zero on wire (`HEADER_RESERVED_OFFSET = 56`) |
| 60 | 4 | `checksum` | Fletcher-32 of bytes 0–59 (`HEADER_CHECKSUM_OFFSET = 60`) |
| **Total** | **64** | | `HEADER_SIZE = 64`; asserted by `test_header_offsets_are_canonical` |

---

## 8. Protocol Level 4: Forensic Ledger Audit (Advisory)

Providing bit-level transparency into the internal state of the oscillator.

### 8.1 Artifact Generation
```bash
cargo run -p dpw4 --features cli --bin precision -- artifacts --out docs/verification
```
**Traces Produced** (in `docs/verification/`):
- **Normative for `sig-util validate` determinism gate**:
  - `saw_20_headroom.det.csv`
  - `pulse_relational_8k.det.csv`
  - `triangle_linearity_1k.det.csv`
  - `sine_linearity_1k.det.csv`
  - `master_sweep_20_20k.det.csv`
  - `long_run_0_1hz.det.csv`
- **Normative for transport/header integrity**:
  - `long_run_0_1hz_headers.bin` (header-only stream for `header_audit --frame-size 64`)
- **Non-normative canary traces**:
  - `phase_wrap_440.det.csv`: consistency-only canary trace; not release acceptance.
  - `phase_wrap_440.canon.sig`: consistency-only canary sidecar; not release acceptance.
- **Advisory forensic traces**:
  - `phase_wrap_440.csv`: Phase-wrap trace for manual audit.
  - `saw_20_headroom.csv`: Validates `i128` stability at extreme gain.
  - `pulse_relational_8k.csv`: Validates Relational Phase Alignment of the Two-Saw method.
  - `triangle_linearity_1k.csv`: Triangle linearity and symmetry audit.
  - `sine_linearity_1k.csv`: CORDIC curvature and residual audit.
  - `master_sweep_20_20k.csv`: Synchronous 4-channel chirp metrology ledger.
  - `long_run_0_1hz.csv`: Long-run drift visibility trace.

### 8.2 Precision Hierarchy Audit
Audit the `internal_residual` column (defined as `residual = Scalar_core - f64_ref`).
- **Primary Truth**: The **I64F64** core now acts as the Primary Source of Truth.
- **Advisory Role**: The `f64` model is strictly advisory for detecting gross implementation drift.
- **Evidence**: Proves the `i128` core tracks the `f64` reference at machine precision (`< 1e-16`).

---

## 9. Protocol Level 5: Billion-Sample Stability (Advisory)

Long-duration drift and DC stability are validated via the **Pro-Audit** stress test, simulating extreme sub-audio operation over a billion samples.

### 9.1 Audit Execution
```bash
cargo run -p dpw4 --release --example stress_test_long_run -- --pro-audit
```

### 9.2 Success Criteria
- **Samples**: `1_000_000_000` (approx. 5.7 hours at 48kHz)
- **Max Residual**: No observed drift in this advisory audit run.
- **DC Offset**: `< 1e-14` (empirical signal centroid).

---

## 10. Separate Experimental Surface

No additional experimental audit surface is included in this repository.

---

## 11. Compliance Checklist (Manual Audit)

| Item | Requirement | Code Reference |
| --- | --- | --- |
| **ABI Alignment** | `#[repr(C)]` on `OscState`, `DpwGain`, `Dpw4State`. | `src/lib.rs` |
| **Phase Logic** | Locked to Scalar addition with $O(1)$ Modulo Normalization. | `src/lib.rs` |
| **Endianness** | L.E. byte serialization for all hashing. | `tests/forensic_audit.rs` |
| **Phase Continuity**| No resets permitted except via explicit `reset()`. | `src/lib.rs` |

---

## 12. Red Flags & Non-Conformance

Any of the following conditions constitutes an **Immediate Failure** of the Reference Baseline:
1. **Hash Mismatch**: Any deviation in `forensic_audit` SHA-256 output.
2. **Phase Jitter**: Any deviation from the Scalar phase engine at sample boundaries.
3. **FP Core**: Inclusion of floating-point math within the core DSP path.
4. **Floating Point Intrusion**: Use of `f64` or `libm` within the `geom-signal` or `dpw4` tick loops.
5. **Padding Noise**: Hashes that vary based on compiler optimization levels.
6. **Proof Failure**: Any failure in the Kani formal verification harnesses.

---

## 13. Tooling Trust Boundary (Normative)

The `precision` CLI tool is the **normative authority** for verification of generated signals.

*   **Primary Authority**: Results produced by `precision` (verify) and the `forensic_audit` test suite are normative.
*   **Normative Exception**: `verify_kani.sh` is normative for Protocol Level -1 orchestration and evidence checks.
*   **Advisory Scripts**: Any other scripts provided in the repository (e.g., Python validation, shell pipes) are advisory and used for developer convenience.
*   **Authoritative Hashes**: SHA-256 results produced over the raw S32LE payload by the `precision` tool's reference generation or matched via `forensic_audit` are the absolute definition of correctness.

## 14. Release-Supporting Build Reproducibility

Dual-build release identity is a release-supporting check, not a second release
gate.

Execution:

```bash
bash verify_release_repro.sh
```

Role:

- use `make gate` as the canonical release gate
- use `bash verify_release_repro.sh` when the release record must retain a
  same-machine dual-build identity check for the `sig-util` release binary
- for `1.2.0`, treat the retained result as supporting-only evidence, not a
  required release-admissibility artifact
- if retained, archive its result under `docs/verification/releases/<version>/`

The script supports direct release-bundle routing by setting:

```bash
RELEASE_EVIDENCE_DIR=docs/verification/releases/<version>/ bash verify_release_repro.sh
```

That archived result is supporting release evidence in the canonical retained
evidence directory and is retained as `release_reproducibility.txt`; it does
not replace `make gate` as the canonical release execution path.

---
**Conclusion**: This protocol provides independent, reproducible verification evidence for assessing a `precision-signal` implementation under the pinned environment and repository-defined gates. For technical support or certification audits, consult the Normative Governance section of this guide.
