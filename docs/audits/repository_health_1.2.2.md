# Repository Health Audit — 1.2.2 (post-remediation)

**Audit date:** 2026-03-25
**Auditor:** automated agent (Claude Opus 4.6)
**Workspace version:** 1.2.2
**Toolchain:** rustc 1.91.1 (pinned via rust-toolchain.toml)
**Branch:** feat/release-1.2.2
**HEAD:** d03be96

---

## 1. Audit Scope Summary

This audit covers the `precision-signal` repository at workspace version 1.2.2
after remediation of all four findings from the prior audit baseline (HEAD `6111fde`).

Scope:
- Cargo workspace (10 crates, 4 default members)
- Two Rust CLI binaries: `precision`, `header_audit`
- Two released Python operator tools: `artifact_tool.py`, `artifact_diff.py`
- One experimental Rust binary: `replay-host`
- Firmware crate: `replay-fw-f446`
- Verification workflows: `make gate`, `make replay-tests`, `make release-bundle-check`
- Documentation set under `docs/`
- Retained release evidence under `docs/verification/releases/1.2.2/`

Prior audit: `docs/audits/repository_health_1.2.2.md` at HEAD `6111fde`.
Four findings (W-1 through W-4) were identified. All four have been remediated
in commits `b9321ea` through `d03be96`.

---

## 2. Metric Scores

| Metric | Score | Justification | Evidence |
|---|---:|---|---|
| Engineering integrity | 88 | Pinned toolchain, `#![forbid(unsafe_code)]`, determinism hashes locked to semver bumps | file: `rust-toolchain.toml`; file: `VERIFICATION_GUIDE.md` §1.2 |
| Determinism / reproducibility | 90 | 7 golden hashes pass, 5/5 hardware runs identical, fixture-drift gate | command-output: `make gate => VERIFICATION PASSED`; file: `docs/verification/releases/1.2.2/sha256_summary.txt` |
| Specification quality | 85 | All 6 spec docs now have versioning terminology blocks; RPL0 contract byte-level; MATH_CONTRACT dense and locked with carry-forward note | file: `docs/spec/rpl0_artifact_contract.md`; file: `docs/MATH_CONTRACT.md` |
| Verification discipline | 87 | Multi-tier gate chain; `make release-bundle-check` now passes cleanly (no WARN) | command-output: `make gate => PASS`; command-output: `make release-bundle-check VERSION=1.2.2 => PASS` |
| Codebase maintainability | 78 | Workspace well-factored (core crates zero-dep); some crates are stubs; 10 members with 4 default | file: `Cargo.toml` workspace members |
| Architecture clarity | 80 | Clear release/experimental boundary; workspace.md routes well | file: `docs/RELEASE_SURFACE.md`; file: `docs/architecture/workspace.md` |
| Documentation depth | 80 | Normative specs thorough; replay docs cover operator workflows | file: `docs/README.md` (index routing) |
| Documentation organization | 80 | Top-level routing clear; performance docs now correctly classified as historical reference; versioning terminology blocks present throughout | file: `docs/README.md`; file: `docs/architecture/performance/CONTROL_SCHEDULER_BENCHMARKING.md` line 5 |
| Repository presentation | 82 | README terse and correct; no inflated claims; release surface distinct from experimental | file: `README.md` |
| Developer onboarding | 85 | "First 5 Minutes" block; carry-forward notes eliminate version-label confusion; all versioning terminology blocks present | file: `README.md` §First 5 Minutes; file: `docs/MATH_CONTRACT.md` carry-forward note |
| Conceptual coherence | 85 | Consistent "deterministic execution analysis infrastructure"; normative/advisory distinction maintained | file: `VERIFICATION_GUIDE.md` §1.1; file: `docs/README.md` |
| Research / innovation value | 75 | Phase-domain fixed-point oscillator with formal proofs; hardware-backed determinism; novel but narrow domain | file: `docs/MATH_CONTRACT.md`; file: `verify_kani.sh` |
| OSS trustworthiness signals | 81 | MIT license, pinned toolchain, hash-locked releases, retained evidence bundles, doc-link CI, all manifests repo-relative | file: `LICENSE`; file: `docs/verification/releases/1.2.2/replay_manifest_v1.txt` |

---

## 3. Implementation Reality Map

| Component | Implemented | Classification | User Path | Verification Path | Evidence |
|---|---|---|---|---|---|
| `precision validate` | yes | Release | `make gate` | `make gate` → 7 golden hashes | file: `crates/dpw4/src/bin/precision.rs` enum Commands::Validate |
| `precision generate` | yes | Release | `precision generate --shape saw ...` | CLI_SURFACE_EVIDENCE.md | file: `crates/dpw4/src/bin/precision.rs` enum Commands::Generate |
| `precision artifacts` | yes | Release | `precision artifacts --out <dir>` | CLI_SURFACE_EVIDENCE.md | file: `crates/dpw4/src/bin/precision.rs` enum Commands::Artifacts |
| `precision inspect` | yes | Release | `precision inspect --file <path>` | CLI_SURFACE_EVIDENCE.md | file: `crates/dpw4/src/bin/precision.rs` enum Commands::Inspect |
| `precision verify` | yes | Release | `precision verify --file <path>` | CLI_SURFACE_EVIDENCE.md | file: `crates/dpw4/src/bin/precision.rs` enum Commands::Verify |
| `header_audit` | yes | Release | `header_audit <file>` | CLI_SURFACE_EVIDENCE.md | file: `crates/dpw4/src/bin/header_audit.rs` |
| `artifact_tool.py` | yes | Release | `python3 scripts/artifact_tool.py verify ...` | `make replay-tool-tests` | file: `scripts/artifact_tool.py` |
| `artifact_diff.py` | yes | Release | `python3 scripts/artifact_diff.py ...` | `make replay-tool-tests` | file: `scripts/artifact_diff.py` |
| `replay-host diff` | yes | Experimental | `cargo run -p replay-host -- diff ...` | not in release gate | file: `crates/replay-host/src/main.rs` |
| `replay-fw-f446` | yes | Release | `make flash-ur` | retained evidence: `docs/verification/releases/1.2.2/firmware_release_evidence.md` | file: `crates/replay-fw-f446/` |

---

## 4. CLI Surface Inventory

| Command | Entrypoint | Type | Runnable | Verification Path | Classification |
|---|---|---|---|---|---|
| `precision validate --mode quick` | `crates/dpw4/src/bin/precision.rs` | rust-bin | observed | `make gate` | Release |
| `precision validate --mode full` | `crates/dpw4/src/bin/precision.rs` | rust-bin | observed | `make gate-full` (supplementary) | Release |
| `precision generate` | `crates/dpw4/src/bin/precision.rs` | rust-bin | observed | `docs/verification/CLI_SURFACE_EVIDENCE.md` | Release |
| `precision artifacts` | `crates/dpw4/src/bin/precision.rs` | rust-bin | observed | `docs/verification/CLI_SURFACE_EVIDENCE.md` | Release |
| `precision inspect` | `crates/dpw4/src/bin/precision.rs` | rust-bin | observed | `docs/verification/CLI_SURFACE_EVIDENCE.md` | Release |
| `precision verify` | `crates/dpw4/src/bin/precision.rs` | rust-bin | observed | `docs/verification/CLI_SURFACE_EVIDENCE.md` | Release |
| `header_audit` | `crates/dpw4/src/bin/header_audit.rs` | rust-bin | observed | `docs/verification/CLI_SURFACE_EVIDENCE.md` | Release |
| `artifact_tool.py verify` | `scripts/artifact_tool.py` | python-tool | observed | `make replay-tool-tests` | Release |
| `artifact_tool.py hash` | `scripts/artifact_tool.py` | python-tool | observed | `make replay-tool-tests` | Release |
| `artifact_diff.py` | `scripts/artifact_diff.py` | python-tool | observed | `make replay-tool-tests` | Release |
| `replay-host diff` | `crates/replay-host/src/main.rs` | rust-bin | observed | none in release gate | Experimental |

---

## 5. Derived Release Surface

| Capability | Implemented | Classification | User Path | Verification Path | Evidence | Confidence |
|---|---|---|---|---|---|---|
| Deterministic validation gate | yes | Release | `make gate` | `make gate` => VERIFICATION PASSED | command-output: observed-this-session | direct |
| Forensic artifact generation | yes | Release | `precision generate`, `precision artifacts` | CLI_SURFACE_EVIDENCE.md | file: `docs/verification/CLI_SURFACE_EVIDENCE.md` | direct |
| Artifact inspection/verification | yes | Release | `precision inspect`, `precision verify`, `header_audit` | CLI_SURFACE_EVIDENCE.md | file: `docs/verification/CLI_SURFACE_EVIDENCE.md` | direct |
| Python replay operator tooling | yes | Release | `artifact_tool.py`, `artifact_diff.py` | `make replay-tool-tests` => PASS | command-output: observed-this-session | direct |
| Hardware capture (F446) | yes | Release | `make flash-ur` | retained evidence: `docs/verification/releases/1.2.2/firmware_release_evidence.md` | file: retained-evidence | direct |
| Rust replay (v0 frames) | yes | Experimental | `cargo run -p replay-host -- diff` | none in release gate | command-output: observed-this-session | direct |
| Formal verification (Kani) | yes | Release | `bash verify_kani.sh` | retained evidence: `docs/verification/releases/1.2.2/kani_evidence.md` | file: `verify_kani.sh`; file: `docs/verification/releases/1.2.2/kani_evidence.md` | direct |

---

## 6. Claim-Reality Gap Register

| # | Claim | Source | Status | Impact | Confidence | Notes |
|---|---|---|---|---|---|---|
| 1 | `make gate` => VERIFICATION PASSED | README.md §First 5 Minutes | exact | — | direct | command-output: `make gate => VERIFICATION PASSED` (observed-this-session) |
| 2 | Released replay-facing operator tooling is the Python toolchain | docs/replay/tooling.md | exact | — | direct | `artifact_tool.py` and `artifact_diff.py` exercised via `make replay-tool-tests` |
| 3 | Rust replay is experimental and not part of release surface | docs/RELEASE_SURFACE.md, docs/replay/tooling.md | exact | — | direct | `replay-host` description: "outside the v1 release surface"; not in `make gate` |
| 4 | Hardware capture re-executed for 1.2.2 | docs/verification/releases/1.2.2/firmware_release_evidence.md | exact | — | retained-evidence | 5/5 runs identical, SHA256 `f79e71d...` consistent across all retained files |
| 5 | Deterministic repeatability: PASS (5/5 runs identical) | docs/verification/releases/1.2.2/firmware_release_evidence.md | exact | — | retained-evidence | sha256_summary.txt shows 6 identical hashes including baseline |
| 6 | MATH_CONTRACT locked at v1.2.1, carried forward to 1.2.2 | docs/MATH_CONTRACT.md | exact | — | direct | Carry-forward note and versioning terminology block now present; axis ambiguity resolved |
| 7 | Spec docs applicable to release 1.2.2 | docs/spec/*.md | exact | — | direct | All spec docs now carry "Applies to: release 1.2.2 (content unchanged)" and versioning terminology blocks |

---

## 7. Documentation Topology Map

| Document | Purpose | Category | Public | Overlap | Authority |
|---|---|---|---|---|---|
| README.md | Entry routing | entry | public | none | canonical |
| docs/README.md | Documentation index | index | public | none | canonical |
| docs/RELEASE_SURFACE.md | Release classification and routing | release-classification | public | none | canonical |
| VERIFICATION_GUIDE.md | Release contract and conformance governance | normative | public | none | canonical |
| docs/MATH_CONTRACT.md | Arithmetic and signal-path contract | normative | public | none | canonical |
| docs/spec/rpl0_artifact_contract.md | Binary artifact format spec | normative | public | none | canonical |
| docs/replay/tooling.md | Replay-tooling boundary | release-classification | public | minor with RELEASE_SURFACE.md | supporting |
| docs/replay/README.md | Replay subsystem index | index | public | none | supporting |
| docs/replay/FW_F446_CAPTURE_v1.md | Capture contract | normative | public | none | canonical |
| docs/replay/DIVERGENCE_SEMANTICS.md | Divergence explanation contract | normative | public | none | canonical |
| docs/cli/precision.md | CLI reference | normative | public | none | supporting |
| docs/architecture/workspace.md | Workspace framing | deep-architecture | public | none | supporting |
| docs/system_architecture_disclosure.md | Architecture overview | descriptive | public | minor (architecture overview) | supporting |
| docs/verification/build_reproducibility.md | Build identity checks | workflow | public | none | supporting |
| docs/verification/CI_EVIDENCE.md | Historical CI evidence | historical-audit | public | none | historical |
| docs/verification/CLI_SURFACE_EVIDENCE.md | CLI promotion evidence | workflow | public | none | supporting |
| docs/verification/releases/1.2.2/ | Retained release evidence bundle | normative | public | none | canonical |
| docs/governance/DESIGN_AXIOMS.md | Governance axioms | normative | public | none | supporting |
| docs/spec/dpw_gain.md | Gain spec | normative | public | none | supporting |
| docs/spec/oscillator_api.md | Oscillator dispatch contract | normative | public | none | supporting |
| docs/spec/reference_invariants.md | Mathematical invariants | normative | public | none | supporting |
| docs/spec/pulse_implementation_spec.md | Pulse/square spec | normative | public | none | supporting |
| docs/spec/header_layout_addendum.md | Header layout | normative | public | none | supporting |
| docs/architecture/performance/ | Performance benchmarking docs | historical-reference | internal | none | historical |
| docs/internal/ | Internal normalization notes | reference | internal | none | supporting |

---

## 8. Terminology Consistency Register

| Rule | File | Line | Classification | Severity | Evidence | Notes |
|---|---|---:|---|---|---|---|
| NAM-004 | docs/spec/rpl0_artifact_contract.md | 55 | ambiguous-version | info | "v1 parsing path" | Adequately disambiguated by Versioning Terminology block at top of file |
| NAM-004 | docs/spec/oscillator_api.md | 14 | ambiguous-version | info | "v1.x line" | Refers to release version range; context adequate with terminology block |

No NAM-001 (artifact vX), NAM-002 (replay vX), NAM-003 (_vX.md artifact spec),
NAM-005 (missing terminology block), or NAM-006 (capability statement) violations
found in public-facing documents.

Prior findings resolved:
- NAM-004 warn on stale `Version: v1.0.0-rc5` headers → now `Document revision: v1.0.0-rc5` with "Applies to" and terminology blocks (all 5 files)
- NAM-005 on 3 spec docs → all 6 spec docs now have Versioning Terminology blocks

---

## 9. Prior Finding Disposition

| Finding | Prior Status | Current Status | Evidence |
|---|---|---|---|
| W-1: Retained manifest absolute run_dir | medium | **improved** | `replay_manifest_v1.txt` now `run_dir=artifacts/replay_runs/...`; `replay_repeat_check.py` uses `repo_relative_path()`; `make release-bundle-check VERSION=1.2.2 => PASS` (no WARN); all three release bundles (1.2.0, 1.2.1, 1.2.2) corrected |
| W-2: Stale v1.0.0-rc5 version headers + NAM-005 | low | **improved** | All spec docs now have `Document revision:` + `Applies to:` + Versioning Terminology blocks |
| W-3: MATH_CONTRACT v1.2.1 label gap | low | **improved** | Carry-forward note, versioning terminology block, and "Applies to: release 1.2.2 (content unchanged)" added |
| W-4: Performance docs classification | low | **improved** | Reclassified from "Normative" to "Historical reference" with terminology blocks |

---

## 10. Strengths

1. **Determinism discipline is exemplary.** 7 golden hashes locked to semver bumps,
   hardware-backed 5/5 repeatability, fixture-drift gate prevents silent regression.
   Evidence: `make gate => VERIFICATION PASSED` (observed-this-session);
   file: `docs/verification/releases/1.2.2/sha256_summary.txt`.

2. **Release/experimental boundary is well-drawn.** `RELEASE_SURFACE.md` explicitly
   classifies every surface; `replay-host` description says "outside the v1 release
   surface"; `docs/replay/tooling.md` reinforces this. No inflation observed.

3. **Verification gate chain is layered and runnable.** `make ci-local` chains
   `doc-link-check fw test replay-tests gate fixture-drift-check`.
   Evidence: `make gate => PASS`; `make replay-tests => PASS`; `make release-bundle-check => PASS`.

4. **Documentation routing is clear.** README → docs/README.md → RELEASE_SURFACE.md
   and VERIFICATION_GUIDE.md. Normative vs descriptive distinction maintained.

5. **Normative specs are binding and precise.** `rpl0_artifact_contract.md` has
   byte-level layout, versioning terminology block, and hard invariants.
   `MATH_CONTRACT.md` has code-line references and Kani proof citations.

6. **Retained release evidence is structured and portable.** All manifests now use
   repo-relative paths. `make release-bundle-check VERSION=1.2.2` passes cleanly.

7. **Version-axis hygiene is comprehensive.** All 6 spec docs, MATH_CONTRACT,
   performance docs, CLI reference, governance axioms, capture contract, and
   divergence semantics now have versioning terminology blocks distinguishing
   document revision from release version.

---

## 11. Weaknesses

No findings at medium or higher severity.

### Finding W-5: improved — supplementary `precision validate --mode full` evidence retained

The `--mode full` validation path was exercised and a supplementary Makefile
target now exists to run it without changing the canonical release gate.

Evidence:
- file: `crates/dpw4/src/bin/precision.rs` enum ValidateMode::Full
- command: `make gate-full` => `VERIFICATION PASSED`
- file: `docs/verification/releases/1.2.2/gate_full_evidence.md`

Disposition: `improved`
Impact: `bounded`
Severity: `low`
Classification: `implementation`
Confidence: `direct`
Resolution note: `make gate` remains canonical; `make gate-full` is documented
as supplementary validation only.

### Finding W-6: improved — Kani verification refreshed for 1.2.2

Formal verification via `verify_kani.sh` was re-run successfully and a fresh
retained evidence file now exists under the `1.2.2` release bundle.

Evidence:
- command: `bash verify_kani.sh` => `PASS`
- file: `verify_kani.sh`
- file: `docs/verification/releases/1.2.2/kani_evidence.md`

Disposition: `improved`
Impact: `bounded`
Severity: `low`
Classification: `implementation`
Confidence: `direct`
Resolution note: canonical Tier-1 Kani evidence is now fresh for release `1.2.2`.

---

## 12. Remediation Priorities

| Priority | Finding | Effort | Impact |
|---:|---|---|---|
| 1 | W-6: Re-execute Kani verification for 1.2.2 | completed this session | improved |
| 2 | W-5: Document or gate `--mode full` | completed this session | improved |

---

## 13. Evidence and Unknowns

### Direct evidence (observed-this-session)

- command: `make gate` => `VERIFICATION PASSED` (7 determinism hashes)
- command: `make gate-full` => `VERIFICATION PASSED` (supplementary full-mode execution)
- command: `make replay-tests` => all parser + tool suites PASS
- command: `make release-bundle-check VERSION=1.2.2` => `PASS` (no warnings)
- command: `make test` => all workspace tests pass (64 tests, 4 ignored)
- command: `python3 scripts/check_doc_links.py` => `PASS`
- command: `bash verify_kani.sh` => `PASS` (Tier-1 runner; 23 harnesses executed, 5 heavy harnesses skipped by default)
- command: `bash verify_release_repro.sh` => `PASS` (dual-build identity retained as supporting evidence)

### Retained evidence

- file: `docs/verification/releases/1.2.2/firmware_release_evidence.md` — hardware capture 5/5 PASS
- file: `docs/verification/releases/1.2.2/kani_evidence.md` — fresh Tier-1 Kani evidence for `1.2.2`
- file: `docs/verification/releases/1.2.2/gate_full_evidence.md` — supplementary `--mode full` retained record
- file: `docs/verification/releases/1.2.2/sha256_summary.txt` — 6 identical hashes
- file: `docs/verification/releases/1.2.2/replay_manifest_v1.txt` — manifest with `final_status=PASS`, `run_dir=artifacts/replay_runs/run_20260325T184038Z`
- file: `docs/verification/CI_EVIDENCE.md` — CI run 23230032284 at v1.2.0-rc1

### Unknowns

- `make ci-local` was not run as a single command (individual components verified).
- No GitHub Actions workflow file was inspected (branch-local only).

---

## Final Question

### 1. What executable capabilities are actually implemented today?

- `precision validate` (canonical determinism gate, 7 golden hashes)
- `precision generate` / `artifacts` / `inspect` / `verify` (operator CLI surface)
- `header_audit` (Fletcher-32 header audit tool)
- `artifact_tool.py` (released Python operator CLI: verify, hash, capture, inspect, compare)
- `artifact_diff.py` (released Python divergence-localization tool)
- `replay-host diff` (experimental Rust v0-frame replay)
- `replay-fw-f446` (firmware capture, hardware-backed)
- Parser and tool regression suites (13+ test scripts)
- Formal verification harness runner (`verify_kani.sh`)

### 2. How accurately does the repository communicate those capabilities?

Accurately. All four findings from the prior audit have been remediated. The
release/experimental boundary is explicitly drawn and consistently maintained.
Version-axis terminology is now systematically disambiguated across specs,
normative contracts, and historical reference material. No claim-reality
contradictions were identified. No inflated claims were found.

### 3. What single sprint would most improve repository credibility and clarity?

**Evidence freshness closure for release 1.2.2 is now complete.**
Fresh Kani evidence has been retained under the `1.2.2` release bundle, and
`precision validate --mode full` is now exercised via a documented
supplementary `make gate-full` target while `make gate` remains canonical.
