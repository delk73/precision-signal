# Changelog

> Changelog policy: this file is release-facing and summarizes what ships.
> Audit policy: proofs, hashes, command transcripts, and close-out narratives live in retained evidence and `docs/audits/`.
> Source of truth: git history is authoritative; this changelog is an index.
> Release policy: `[Unreleased]` contains next-cut scope only.

## [Unreleased]

### Added

* **[Docs/WIP] Initialized experimental/non-normative WIP documentation surface.**
  Added `docs/wip/` for exploratory work, bring-up evidence, and experiment
  logging. This surface is strictly non-normative and does not affect the
  release or verification authority.
* **[Hardware] Added BeagleBone Black prudent bring-up procedure (T0-T5).**
  Added `docs/wip/bbb_prudent_bringup.md` defining an offline-first,
  surface-reduced bring-up for hostile or unknown-provenance hardware.
* **[WIP] Logged BBB-001 intake, isolation, and toolchain verification.**
  Captured evidence for board `BBB-001` in `docs/wip/bbb_bringup_evidence_BBB-001.md`,
  including successful on-target bit-exact `make gate` and `make replay-tests`.

## [1.2.2] - 2026-03-26

### Added

* **[Docs] Added shared documentation link-integrity tooling and enforcement.**
  Added the shared `doc-link-integrity` skill, the `docs-consistency` agent,
  `make doc-link-check`, and CI/regression enforcement for public
  documentation link integrity.

### Changed

* **[Audit] Canonicalized repository-auditor evidence modeling and expectations.**
  The evidence model now distinguishes `observed-this-session`,
  `retained-evidence`, and `unknown`, with added canonicalization rules for
  evidence formatting and interpretation.

* **[Docs] Converted public documentation navigation to source-resolving links.**
  Public documentation references were mechanically converted to clickable,
  source-resolving Markdown links, and CI/local workflows now include
  documentation link checking.

### Fixed
* **[Docs] Fixed GitHub Pages routing for retained release evidence link.**
  
* **[Docs] Fixed non-clickable public documentation references.**

* **[Audit] Fixed ambiguous audit evidence formatting expectations.**

### Notes

* No change to DSP core.
* No change to artifact format.
* No change to validation semantics.
* No expansion of release surface.


## [1.2.1] - 2026-03-20

### Added

* **[Firmware] Promoted the F446 board capture path to normative RPL0 v1 container output.**
  `replay-fw-f446` now emits `[HEADER][SCHEMA BLOCK][FRAME DATA]` artifacts
  with a fixed v1 header encoder, embedded schema bytes, deterministic schema /
  build / config identity hashes, and host-side regression coverage that locks
  those digest constants against metadata drift.

* **[Docs] Added the active F446 capture contract for the v1 operator path.**
  Added `docs/replay/FW_F446_CAPTURE_v1.md` as the normative board-capture
  contract for the current release surface, including the emitted header
  fields, schema-block expectations, determinism guarantees, and operator
  verification workflow.

### Changed

* **[Replay] Promoted the firmware baseline, repeat-capture flow, and release manifests to the v1 artifact contract.**
  The canonical `artifacts/baseline.bin` and its retained metadata now record
  `artifact_version = 1`, schema hash, header length, schema length, and frame
  size. The capture/promotion scripts were updated to parse v1 containers,
  verify schema integrity, and write `replay_manifest_v1.txt` for repeat-capture
  and release-evidence runs.

* **[Docs] Re-routed replay operator documentation to the active v1 contract.**
  Updated README, docs index, CI-gate guidance, and replay subsystem documents
  to point operators at `FW_F446_CAPTURE_v1.md` as the current contract while
  reclassifying the v0 capture documents as legacy or historical references.

### Release

* **[Release] Introduced new retained release evidence for the v1 capture cutover (`1.2.1`).**

* **[Release] Aligned workspace/package release identity and normative release governance to `1.2.1`.**
  Updated workspace version metadata, the canonical retained-record contract,
  and public release-surface routing so the accepted `1.2.1` evidence, active
  firmware capture path, and released operator-facing tooling describe one
  consistent release boundary.

* **[Release] Preserved `1.2.0` retained evidence without mutation to maintain audit integrity.**

## [v1.2.0-rc1] - 2026-03-17

### Added

* **[License] Added MIT License.**
  Added `LICENSE` file (MIT, copyright 2026 Dan Elkins) and a corresponding
  License section in `README.md`.

* **[Docs] System architecture disclosure document.**
  Added `docs/system_architecture_disclosure.md` recording the implemented
  architecture: artifact capture, deterministic replay, divergence
  localization, identity hashing, and verification infrastructure.

* **[Docs] Architecture whitepaper for the v1 replay release surface.**
  Added `docs/architecture_whitepaper.md` covering the artifact boundary,
  replay state machine, divergence analysis pipeline, verification tiers,
  and system architecture of the deterministic execution analysis
  infrastructure.

* **[Docs] Release surface and claim taxonomy document.**
  Added `docs/RELEASE_SURFACE.md` defining descriptive claim classes
  (Implemented, Normative, Experimental) and the canonical release-boundary
  language used by README and architecture docs.

### Changed

* **[Docs] Clarified v1 release boundaries and workspace maturity in README.**
  Reframed the project description as a multi-crate platform workspace for
  deterministic execution analysis. Added a workspace maturity map, system
  category section, current v1 release surface summary, and explicit
  experimental-boundary language. Shortened demo ladder labels to canonical
  names.

* **[Docs] Updated documentation to reflect inherent execution analysis
  infrastructure.**
  Aligned `docs/README.md`, `docs/architecture/workspace.md`,
  `docs/architecture/system_surfaces.md`, `docs/replay/README.md`,
  `docs/replay_explained.md`, `VERIFICATION_GUIDE.md`, and
  `docs/verification/build_reproducibility.md` with the execution-analysis
  framing and v1 release-boundary language.

* **[Crates] Updated crate descriptions for release-boundary clarity.**
  Revised Cargo.toml descriptions for `dpw4`, `replay-cli`, `replay-core`,
  `replay-embed`, and `replay-host` to reflect v1 release surface membership
  and experimental status where applicable.

* **[Docs] Refreshed demo labels and descriptions for consistency.**
  Tightened demo ladder labels in `docs/demo_visual.html`, `docs/index.html`,
  `docs/demo_captured_divergence.md`, `docs/demo_v4_region_attribution.md`,
  `docs/demo_v5_evolution.md`, and `docs/demo_claim_matrix.md` to use
  canonical short names and consistent field ordering.

## [v1.1.0-replay-evolution] - 2026-03-13

### Added

* **[Replay Demo] Demo V5 divergence evolution semantics surface and workflows.**
  Added deterministic post-divergence classification to the replay demo ladder
  with the labels:

  ```
  self_healing
  bounded_persistent
  monotonic_growth
  region_transition
  ```

  Lifecycle targets:

  ```
  make demo-v5-verify
  make demo-v5-audit-pack
  make demo-v5-record
  make demo-v5-release
  ```

* **[Replay Demo] Deterministic Demo V5 fixture generator and regression suite.**

  ```
  scripts/generate_demo_v5_fixtures.py
  tests/test_demo_v5_evolution.py
  artifacts/demo_v5/
  ```

  These produce and verify reproducible evolution fixtures covering
  self-healing, bounded persistence, monotonic growth, and region transition.

### Changed

* **[Replay Demo] Extended `artifact_diff.py` with evolution semantics.**
  Comparator output now includes:

  ```
  evolution_class
  timeline_summary
  ```

  alongside the existing first-divergence frame, shape, and region fields.
  The no-divergence surface is also covered by regression for the `none` output
  values.

* **[Docs] Extended the public demo ladder through Demo V5.**
  Updated the README, GitHub Pages index, and interactive visual demo to carry
  the explanatory ladder from perturbation through evolution semantics while
  preserving captured-artifact provenance in:

  ```
  README.md
  docs/index.html
  docs/demo_visual.html
  docs/demo_v5_evolution.md
  ```

* **[Polish] Completed Demo V5 cleanup pass before release consideration.**
  Renamed the divergence summary printer for semantic clarity, removed dead
  fixture-generator state, tightened README wording around captured divergence
  provenance, and added explicit no-divergence regression coverage for the V5
  output surface.

## [v1.0.0-captured-divergence] - 2026-03-13

### Added

* **[Replay Demo] Captured divergence demonstration workflow and release surface.**
  Added a release-facing captured-artifact demo proving that replay divergence
  analysis works on artifacts captured from separate hardware executions rather
  than synthetic post-processing.

  Release artifacts:

  ```
  artifacts/demo_captured/run_A.rpl
  artifacts/demo_captured/run_B.rpl
  ```

  Workflow/documentation surface:

  ```
  make capture-demo-A
  make capture-demo-B
  make demo-captured-verify
  make demo-captured-release
  docs/demo_captured_divergence.md
  ```

* **[Replay Demo] Hardware-captured divergence evidence pack.**
  Added committed captured artifacts for the release evidence surface under
  `artifacts/demo_captured/`, with the documented expected behavior:

  ```
  first_divergence_frame: 4096
  primary_region: sample_payload
  region_summary: sample_payload
  shape_class: transient
  ```

### Changed

* **[Release] Refreshed captured divergence capture manifests.**
  Added promoted manifest records for the two hardware capture directories:

  ```
  artifacts/demo_captured/a_capture/manifest.txt
  artifacts/demo_captured/b_capture/manifest.txt
  ```

  This aligns the release evidence pack with the captured artifact refresh used
  for the tag cut.

## [v1.0.0-demo-v4] - 2026-03-13

### Added

* **[Replay Demo] Demo V4 divergence region attribution surface and workflows.**
  Added Demo V4 documentation and deterministic fixtures enabling structural
  attribution of the first divergence region:

  ```
  header_schema
  timer_delta
  irq_state
  sample_payload
  mixed
  ```

  Lifecycle targets:

  ```
  make demo-v4-verify
  make demo-v4-audit-pack
  make demo-v4-record
  make demo-v4-release
  ```

* **[Replay Demo] Deterministic fixture generator and regression suite.**

  ```
  scripts/generate_demo_v4_fixtures.py
  tests/test_demo_v4_region_attribution.py
  ```

  These produce and verify reproducible divergence-region fixtures under
  `artifacts/demo_v4/`.

* **[Replay Demo] Region attribution in `artifact_diff.py`.**

  Comparator now reports:

  ```
  first_divergence_frame
  shape_class
  primary_region
  all_regions_at_first_divergence
  region_summary
  reconvergence_summary
  ```

### Changed

* **[Replay Demo] Extended fixture drift guard.**

  `fixture-drift-check` now regenerates and validates both:

  ```
  artifacts/demo_v3/
  artifacts/demo_v4/
  ```

* **[Docs] Added Demo V4 explanatory documentation.**

  ```
  docs/demo_v4_region_attribution.md
  ```

  This document describes region attribution rules, precedence ordering,
  and interaction with Demo V3 shape classification.

## [v1.0.0-demo-v3] - 2026-03-13

### Added

* **[Makefile] Added `demo-v2-record` target.**
  Introduces a record-keeping target that captures the output of `demo-v2-verify` and `demo-v2-audit-pack` into a dated audit log in `docs/audits/runlogs/`.
  The log includes the git commit SHA and UTC timestamp for traceability.

* **[Replay Demo] Demo V3 divergence classification surface and workflows.**
  Added Demo V3 documentation and deterministic fixtures for divergence-shape classification with the labels:

  ```
  transient
  persistent_offset
  rate_divergence
  ```

  Lifecycle targets:

  ```
  make demo-v3-verify
  make demo-v3-audit-pack
  make demo-v3-record
  make demo-v3-release
  ```

* **[Replay Demo] Deterministic fixture generator and regression coverage.**

  ```
  scripts/generate_demo_v3_fixtures.py
  scripts/test_artifact_diff.py
  scripts/test_demo_v3_fixtures.py
  ```

  These generate and validate reproducible Demo V3 fixtures under `artifacts/demo_v3/`.

### Changed

* **[Makefile] Added Demo V3 fixture drift guard and release convenience target.**

  ```
  make fixture-drift-check
  make demo-v3-release
  ```

  `fixture-drift-check` regenerates `artifacts/demo_v3/` and fails on drift, and `ci-local` now includes this guard.
  `demo-v3-release` runs the full Demo V3 lifecycle (`verify`, `audit-pack`, `record`) in one command.

* **[Replay Demo] Extended `artifact_diff.py` with divergence classification.**
  Comparator output now includes the classification label alongside the first divergence frame for Demo V3-compatible sample divergences.

* **[Docs] Clarified Demo fixture determinism and audit lifecycle policy.**
  Updated Demo V3 documentation to state that fixtures are deterministically derived from `artifacts/demo_persistent/run_A.rpl`, expected to remain bit-identical across runs, and protected by CI drift checks.
  Added runlog retention and release closeout guidance under `docs/audits/`.

## [1.0.0-demo-v2] - 2026-03-12

### Added

* **[Replay Demo] Persistent divergence localization demonstration (Demo V2).**
  Introduces a second replay demo proving **persistent execution-trajectory divergence localization** in addition to the transient mismatch shown in Demo V1.

  * Fixture artifacts:
    `artifacts/demo_persistent/run_A.rpl`
    `artifacts/demo_persistent/run_B.rpl`
  * First divergence frame: **4096**
  * Divergence remains persistent for all subsequent frames.
  * Demonstrates deterministic identification of the **exact first divergence frame** using:

    ```
    python3 scripts/artifact_diff.py
    python3 scripts/artifact_tool.py inspect
    ```

* **[Replay Demo] Demo V2 capture workflow.**
  Added firmware feature `demo-persistent-divergence` to inject a one-time phase perturbation inside the TIM2 ISR, producing a persistent shift in the sample trajectory.

* **[Makefile] Added deterministic Demo V2 workflow targets.**

  ```
  make demo-v2-capture
  make demo-v2-verify
  make demo-v2-audit-pack
  ```

  These targets provide:

  * reproducible hardware capture of canonical and perturbed artifacts
  * mechanical verification of divergence behavior
  * generation of a plain-text audit bundle suitable for independent verification.

### Changed

* **[Docs] Added persistent divergence demo documentation.**
  New walkthrough:

  ```
  docs/demo_persistent_divergence.md
  ```

  clarifying fixture provenance and optional hardware recapture workflow.

* **[Docs] Extended demo visualizer to support Demo V2.**

  ```
  docs/demo_visual.html
  ```

  The visual demo now allows switching between:

  * Demo V1 — transient divergence
  * Demo V2 — persistent divergence localization

* **[Docs] Updated claim/evidence matrix for replay demonstrations.**

  ```
  docs/demo_claim_matrix.md
  ```

  Distinguishes between:

  * transient divergence localization (Demo V1)
  * persistent divergence localization (Demo V2 fixture)
  * captured execution-state divergence (optional hardware evidence).

### Validation & Operational Notes

* Demo V2 fixture artifacts deterministically exhibit:

  ```
  First divergence frame: 4096
  ```
* Expected verification pattern:

  ```
  run_A --signal-model phase8  → PASS
  run_B --signal-model phase8  → FAIL at frame 4096
  run_B --signal-model none    → PASS
  ```
* Hardware capture path (optional) reproduces the same divergence pattern using the `demo-persistent-divergence` firmware feature.

* [Replay] Replay artifact subsystem is now operationally closed.
  Blessed artifact: `artifacts/phase8_run1.bin`
  (SHA-256: `b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3`).
* [Replay] Prepared `FW_F446_CAPTURE_v0.md` as the normative capture contract for phase8.
  Signal model: phase8 (`sample = frame_idx & 0xFF`).
  Artifact format frozen: version=0, magic=RPL0, size=160016 bytes.
* [Scripts] `lock_baseline.py` now requires explicit `--signal-model`,
  records it in `baseline.json`, and verifies source artifact before promotion.
  Supports `--manifest-name` for named manifest lookup.
* [Scripts] `repeat_capture.py` requires explicit `--signal-model`
  and supports `--manifest-name` for named manifest output.
* [CI] Added `Replay Baseline Verify (phase8)` step: validates structure,
  signal model, and SHA-256 hash of committed `artifacts/baseline.bin`.
* [CI] Added `Replay Baseline Metadata Consistency` gate to ensure `baseline.bin`,
  `baseline.json`, and `baseline.sha256` remain mutually consistent.
* [Docs] README capture workflow updated to canonical phase8 runbook
  with explicit `--signal-model phase8` in all commands.
* [Docs] Prepared `CI_GATES.md` for normative status and new gate.

### Validation & Operational Notes
* [Replay] Phase B operator validation (baseline promotion and repeatability
  proof) pending hardware execution.

### Added
* [Scripts] Added modular `--signal-model` support (`none`, `ramp`, `phase8`) to `scripts/artifact_tool.py` and `scripts/read_artifact.py`, enabling artifact validation against multiple firmware signal models.
* [Replay] RPL0 Artifact Contract v1 specification defining a stable replay artifact format with fixed 16-byte frames and schema extension block.
* [Replay] Hardened artifact parsing/identity tooling: canonical `artifact_tool.py hash` command, bounded header/schema/frame region checks, and adversarial malformed-input parser tests.

### Changed
* [Replay] Hardened `replay-fw-f446` TIM2 ISR: `sr().modify()` → `sr().write()` for proper rc_w0 UIF clear (preemptive status-register hardening). See `docs/replay/ISR_ADVISORY.md` Finding A.
* [Replay] Normalized `replay-fw-f446` phase accumulator to read-then-advance ordering, producing `sample = frame_idx & 0xFF` per frozen phase8 convention. See `docs/replay/ISR_ADVISORY.md` Finding B.
* [Scripts] Updated `read_artifact.py` phase8 expected-sample formula to `frame_idx & 0xFF` to match frozen convention.

### Validation & Operational Notes
* [Replay] Phase A hardware validation passed on STM32F446: phase8 first-check, full verify, wrap-boundary inspect, and repeat-capture compare all passed; canonical observed contract is `sample = frame_idx & 0xFF`.
* [Ops] Observed transient ST-Link enumeration failure and transient `/dev/ttyACM0` readiness/read errors prior to successful capture; subsequent flash/capture/compare succeeded without code changes.

### Added
* [Scripts] Added `scripts/test_phase8_model.py` offline contract test for phase8 signal model with migration gate, negative check, and boundary assertions.
* [Replay] Migrated `replay-fw-f446` signal generation from a monotonic 32-bit ramp to a 32-bit phase accumulator (`SIGNAL_PHASE`) with step `0x0100_0000`, emitting the high byte (`phase >> 24`) as the sample to produce a 256-step periodic saw.
* [Scripts] Updated artifact validation logic in `scripts/artifact_tool.py` to support selectable signal-model verification rather than assuming the baseline ramp.

### Added
* [Replay] Added diagnostic-only `debug-repeat-dump` feature to `replay-fw-f446` for continuous artifact emission in loop, bypassing the standard capture-and-halt behavior.

### Changed
* [Replay] Simplify artifact script validation logic by removing XOR constant and using sequential sample index tracking.

### Added
* [Replay] Added `scripts/artifact_tool.py`, a unified CLI wrapper for `capture`, `verify`, `compare`, `inspect`, and `lock-baseline` workflows.
* [Replay] Added `scripts/inspect_artifact.py` for targeted per-frame hex/field inspection and summary of `RPL0` artifacts.
* [Replay] Implement repeat UART artifact capture script with manifest generation.
* [Replay] Added `scripts/compare_artifact.py` to identify semantic divergence between two `RPL0` artifacts.
* [Replay] Added `scripts/lock_baseline.py` to promote verified artifacts to canonical baselines with full metadata (SHA256, git commit, capture environment).
* [Replay] Initial `artifacts/baseline.json` and `baseline.sha256` for Sprint 1 verification tracking.
* [Replay] Sprint 1 host artifact contract/wire types in `replay-core` (`RPL0`/v0 `Header0` + `EventFrame0`) with explicit little-endian field encoding and frozen size constants.
* [Replay] Host-side artifact parser in `replay-host` (`parse_header0`, `parse_frames0`, `debug_dump_first_frames`) with strict magic/version/length validation and safe `from_le_bytes` decoding.
* [Replay] Host parser tests for valid synthetic artifact parsing plus bad-magic and bad-length rejection cases.
* [Replay] Added `replay-fw-f446` firmware MVP crate for NUCLEO-F446RE capture->halt->dump artifact egress over USART2 (ST-LINK VCP), using the existing replay wire contract.
* [Replay] Added optional `debug-irq-count` feature in `replay-fw-f446` exposing `IRQ_COUNT` (`#[no_mangle]` `AtomicU32`) for debugger-visible TIM2 ISR progress instrumentation without changing default firmware behavior.
* [Replay] Added `scripts/read_artifact.py` host-side UART ingestion utility for `RPL0` stream sync, Header0/EventFrame0 little-endian decode, and boundary frame telemetry printout.

### Changed
* [Replay] Sprint 2: Evolved `replay-fw-f446` signal generation from static XOR-based patterns to a dynamic ramp (`SIGNAL_STATE` + `SIGNAL_STEP`) to verify sub-frame counter integrity in captured artifacts.
* [Repo] Updated `.gitignore` to exclude `scripts/__pycache__`.
* [Docs] Release hygiene reset: changelog is release-facing; proofs and ledgers moved under `docs/audits/`.
* [Replay] Hardened debug/release boundary for replay capture: `IRQ_COUNT_PROBE` is now emitted only with `debug-irq-count`, while canonical firmware/capture docs now use default no-feature commands (`make flash-ur`, `make flash-compare-ur`, `artifact_tool.py capture/verify/compare/inspect`). `debug-irq-count` is explicitly documented as diagnostic-only.
* [Replay] Updated `artifacts/baseline.json` with current Sprint 1 verified commit and capture timestamp.
* [Replay] UART ingestion utility `scripts/read_artifact.py` updated with robust partial read handling, explicit `EXPECTED_FRAME_COUNT` (10,000) validation, and failure-mode data persistence via `write_capture`.
* [Replay] Corrected `replay-fw-f446` `TIMER_DELTA_NOMINAL` back to 1000 and `INPUT_XOR` back to `0x5A5A_1F1F` after verifying baseline sensitivity and repeatability via `scripts/compare_artifact.py`.
* [Replay] `replay-embed` placeholder now references `replay_core::artifact::Header0` to stay aligned with the new core contract surface.
* [Replay] Hardened `replay-fw-f446` compile surface: embedded implementation isolated under target cfg module, host stub main enabled for workspace host tests, and embedded linker directives gated to arm/none targets.
* [Replay] Fixed `replay-fw-f446` embedded link path by invoking `-Tlink.x` (with crate-local `memory.x` discoverable via link-search), replacing the prior `-Tmemory.x`-only flow that allowed empty/debug-only ELF output.
* [Replay] Rooted embedded runtime symbols at crate root for `replay-fw-f446`: `#[entry] fn main() -> !` and `#[interrupt] fn TIM2()` now forward to `fw::fw_main()` / `fw::tim2_isr()`, keeping firmware behavior unchanged while ensuring retained startup/ISR roots.
* [Replay] Enabled `cortex-m` feature `critical-section-single-core` for `replay-fw-f446` (and corresponding `Cargo.lock` update adding `critical-section`) to satisfy embedded critical-section linker symbols under the corrected runtime link flow.
* [Replay] TIM2 IRQ debug instrumentation in `replay-fw-f446` uses the unmangled probe symbol `IRQ_COUNT_PROBE` in `.bss.irq_probe`, updated by volatile read/write in `tim2_isr()` and available only when `debug-irq-count` is enabled.
* [Replay] Corrected `replay-fw-f446` USART2 BRR programming for APB1=16 MHz / 115200 baud (`div_mantissa=8`, `div_fraction=11`) and restored normal `fw_main()` capture path after temporary UART diagnostic spew.
* [CI] Hardened the `replay-fw-f446` build oracle in `.github/workflows/ci.yml` to assert ELF existence, require a code section (`.text`), and verify a non-zero entry point before artifact conversion.
* [Build] Added repo-root `Makefile` targets (`fw`, `fw-bin`, `test`, `gate`, `ci-local`, `clean`) to mirror recurring local/CI validation commands under `--locked`; `fw-bin` now emits a deterministic repo-local artifact at `target/thumbv7em-none-eabihf/debug/replay-fw-f446.bin` with stale-file removal and non-zero-size assertion.

## [Post-rc5 / Uncut]

Temporary holding area: items move into the next release cut once versioned.
`[Unreleased]` contains next-tag scope only.

### Changed
* [Audit] Δ-02 Gain mantissa invariance. See `docs/audits/delta-02_gain-mantissa-invariance.md`.
* [Audit] Δ-03 Triangle DPW4 integrity. See `docs/audits/delta-03_triangle-dpw4-integrity.md`.
* [Audit] Δ-04 Sine quantizer semantics. See `docs/audits/delta-04_sine-quantizer-semantics.md`.
* [Audit] Δ-BD Build determinism hardening. See `docs/audits/delta-bd_build-determinism.md`.
* [Audit] P5 artifact boundary policy/enforcement. See `docs/audits/P5_artifact_boundary.md`.
* [Audit] Post-rc5 narrative ledger archive. See `docs/audits/POST_RC5_UNSORTED_LEDGER.md`.

## [v1.0.0-rc5] - "Reference Baseline" - 2026-01-29

### Release Gate & Signal Core
* **Native Validation Gate**: Promoted `precision validate` to the canonical release gate (`--mode quick` and `--mode full`), with `verify_determinism.sh` retained only as a compatibility wrapper.
* **Toolchain Pin**: Locked normative execution to Rust toolchain `1.91.1` via `rust-toolchain.toml` and validation checks.
* **Geometric Basis Set**: Expanded normative deterministic artifacts to a 5-scenario basis set:
  * `saw_20_headroom.det.csv`
  * `pulse_relational_8k.det.csv`
  * `triangle_linearity_1k.det.csv`
  * `sine_linearity_1k.det.csv` (CORDIC Sine kernel identity gate)
  * `master_sweep_20_20k.det.csv` (synchronous multi-channel sweep gate)
* **Synchronous Master Sweep**: Added a phase-continuous 20Hz->20kHz sweep with phase-locked Saw, Pulse, Triangle, and CORDIC Sine channels.
* **Test Coverage**: Expanded baseline coverage to 18 `dpw4` unit tests and 46 total workspace tests.

### The Geometric Shift
* **Paradigm:** Refactored entire codebase from Time-Based DSP to **128-bit Geometric Signal Simulation** (`I64F64`).
* **Architecture:** Split monorepo into workspace:
    * `crates/geom-signal`: Pure mathematical bedrock (CORDIC, Sqrt, Scalar).
    * `crates/dpw4`: DSP reference implementation (Oscillators, Transport).

### Hardening & Verification
* **Math Bedrock:** Expanded CORDIC `ATAN_TABLE` from 12 to 32 entries.
    * *Result:* Reduced small-angle precision floor in core trig stress tests.
* **Stress Testing:** Added `stress_math.rs` property tests.
    * *Astronomical-Scale Stress Test:* Verified signal stability at $\theta = 10^{15}$ (approx. 660 years of phase evolution).
    * *Pythagorean:* Verified $sin^2 + cos^2 \approx 1$ with tolerance $\le 5.4 \times 10^{-17}$.

### Performance Benchmarks (Advisory; non-normative) - Raspberry Pi 3B / ARMv8
Hardware verification established the following "Polygon Budgets" for real-time synthesis at 48kHz:
* **Polynomial Engine (Sawtooth):** 2.34 MHz (~48x real-time).
* **Geometric Engine (Sine):** 0.66 MHz (~13.8x real-time).
* *Optimization Note:* The `sin_cos_fast` path (skipping 128-bit modulo) yielded only ~1.8% speedup, confirming that the 64-iteration CORDIC loop is the primary cost center.

### Fixed
* **Documentation:** Overhauled README to reflect the "Hourglass Architecture" (128-bit Phase -> 32-bit Poly -> 128-bit State).
* **Verification:** Updated `VERIFICATION_GUIDE.md` to mandate `I64F64` Scalar math as the reference standard.

### Changed
* **API Break:** Triangle is now a deprecated type alias of TriangleDPW4. Code relying on value construction Triangle (unit struct) breaks and must migrate to TriangleDPW4 (or another explicit constructor pattern). Generic type usage ::<Triangle> still compiles but is deprecated.

## [v1.0.0-rc4] - 2026-01-25

### Added
* **[Docs]** Created `docs/REFERENCE_HARDWARE.md` defining the "Scope Rounding" physics and recommended 1st-Order RC reconstruction filter.
* **[Versioning]** Bumped version to `1.0.0-rc4` and updated Reference Lock invariants.

## [v1.0.0-rc3] - 2026-01-24

### Added
* **[Verify]** Implemented **Protocol Level 0 (Unlocked Physics)** in `examples/rpi_scope_validation.rs`.
* **[Verify]** Added **Hardware-Resilient Mock Mode** to the RPi validation harness, allowing logic verification on non-Pi workstations.
* **[Dev]** Locked `rppal` to `0.22.1` for Raspberry Pi 5 compatibility and safety.

## [v1.0.0-rc2] - 2026-01-23

### Scope Refinement

* **Precision Focus**: Removed instances of "Nano" from the main verification guide to eliminate confusion. Level 4 (Billion-Sample Stability) is now the final verification tier for the Reference Baseline.
* **Guide Hardening**: Formally designated `VERIFICATION_GUIDE.md` as the **Canonical Verification Protocol**.
* **Cross-Platform Hashing**: Updated `verify_determinism.sh` with OS-detection for `md5sum` (Linux) vs `md5` (macOS).

## [v1.0.0-rc1] - 2026-01-23

### Canonical Reference Baseline

* **Frozen Definition**: Established the core as a **Canonical Reference Baseline**. This is the only valid interpretation of "Precision-DPW conformance".
* **Normative Governance**: Introduced a strict hierarchy of truth. SHA-256 hashes and behavioral invariants are now *Normative* (rejection on failure), while metrics like DC offset are *Advisory*.
* **Integer Model**: Fixed the reference as **Two's Complement** bit-exact with Rust/LLVM semantics.
* **Core Hardening**: 
    * Phase truncation logic locked to `(s_q31 >> 1)`.
    * `OscState` and `DpwGain` marked `#[repr(C)]` for ABI stability.
* **Forensic Hashing**: Updated audit procedures to hash only **serialized Little-Endian bytes** of the signal, eliminating ABI padding noise.
* **Nano Acceptance Criteria**: Defined hard-coded numeric bounds (LSB peak error over 1s) for hardware conformance.
* **Toolchain Lock**: Enforced `rust-toolchain.toml` at version `1.91.0`.

## [v0.5.0] - 2026-01-22

### Finalized Reference System

* **Gold Standard Established**: Formally designated this repository as the **Precision-DPW Reference** implementation, serving as the immutable ground truth for the  /  core.
* **Reference Contract**: Implemented strict `libm` routing for all floating-point math to ensure cross-platform bit-determinism.
* **Relational Pulse Logic**: Implemented bit-aligned  subtraction for Pulse and Square waveforms, maintaining  precision before gain/clipping application.
* **Forensic Ledger**: Overhauled the CSV schema to provide deep state transparency, including dynamic exponents, clipping flags, and internal residuals.
* **Nano-DPW Transition**: Validated differentiator stability across 440Hz phase-wrap boundaries, confirming the core is ready for performance-optimized "Nano" ejections.

### Spec Delta: Phase 13 — Pulse & 64-bit Elevation

**Affected Spec Sections:**

* `docs/spec/reference_invariants.md` (§1, §3, §4)
* `docs/spec/pulse_implementation_spec.md` [NEW]

**Changes:**

1. **Accumulator Elevation**: Upgraded the `Emitter` phase accumulator from `u32` to `u64`, providing sub-nanosecond pulse-width resolution and eliminating sub-bass pitch jitter.
2. **Differential Synthesis**: Implemented `Pulse(t, w) = (Saw(t) - Saw(t-w)) * 0.5`. This  scaling satisfies Phase 8 (Headroom) requirements for differential signals.
3. **Square Wave Definition**: Formalized `Square` as an alias for `Pulse(t, 0.5)`.
4. **Hardware Rationalization**: Baselined the core for 64-bit word-size architectures (ARM64, x86_64) and high-performance embedded targets (Cortex-M4/M7).
5. **Conditioner Expansion**: Expanded DC-tracking state to 4 slots to accommodate high-order shape tracking for Square and Pulse waveforms.

### Forensic Verification (v0.5.0)

**Added:**

* **Triple-Trace Generator**: `examples/generate_forensic_artifacts.rs` now produces exhaustive traces (saw, pulse, headroom) with 8-column state logging.
* **Precision Hierarchy Proof**: Empirically demonstrated that the  core tracks the `f64` reference at 20Hz with residuals , while  implementations exhibit visible quantization laddering.
* **Headroom Validation**: Verified stable differentiator states at magnitudes exceeding  (Gain x34) while maintaining safe production output via convergent rounding and hard clipping.

**Removed:**

* Legacy `plotters` and `csv` crate dependencies to minimize the dependency graph.
* All non-deterministic `std::f64` method calls in the forensic loop.
