# Repository Health Audit — precision-signal 1.6.0 (Baseline)

**Audit date:** 2026-04-14
**Audit type:** Fresh baseline — full repository health audit
**Release candidate:** 1.6.0
**Auditor:** automated repository auditor
**Prior audit reference:** none (fresh baseline)

---

## 1. Audit Scope Summary

This audit evaluates the `precision-signal` repository at version `1.6.0` as a fresh baseline. It covers:

- executable surfaces, CLI entrypoints, and build configuration
- release contract fulfillment per `VERIFICATION_GUIDE.md`
- retained release bundle completeness and coherence
- documentation topology, terminology consistency, and claim-reality alignment
- all scored metrics with evidence

Evidence was gathered from static file inspection and live command execution.

---

## 2. Metric Scores

| Metric | Score | Justification | Evidence |
|---|---|---|---|
| Engineering integrity | 88 | hash-locked determinism, formal verification harnesses, pinned toolchain, float quarantine, narrowed release claims | file: `VERIFICATION_GUIDE.md`; command-output: `make gate => VERIFICATION PASSED`; command-output: `cargo check --workspace --no-default-features => ok` |
| Determinism / reproducibility discipline | 90 | dual-build identity reproducibility check, SHA-256 hash-locked baselines, pinned toolchain `1.91.1`, `SOURCE_DATE_EPOCH` in repro script | file: `docs/verification/releases/1.6.0/release_reproducibility.txt`; file: `rust-toolchain.toml` |
| Specification quality | 85 | normative math contract locked, RPL0 artifact contract with terminology block, CLI contract with schema authority, gain model specified | file: `docs/MATH_CONTRACT.md`; file: `docs/spec/rpl0_artifact_contract.md`; file: `docs/authority/cli_contract.md` |
| Verification discipline | 90 | 22 Tier-1 + 4 Tier-2 Kani harnesses, `precision_authoritative_surface` (20 tests), retained Kani evidence, make gate canonical entrypoint, parser adversarial suite | file: `docs/verification/releases/1.6.0/kani_evidence.txt`; command-output: `make gate => VERIFICATION PASSED`; file: `docs/verification/releases/1.6.0/make_replay_tests.txt` |
| Codebase maintainability | 80 | modular bin layout with common/precision/sig_util split, 10 workspace crates, clear feature gating | file: `crates/dpw4/src/bin/precision.rs`; file: `Cargo.toml` |
| Architecture clarity | 82 | clear separation between normative core and CLI layer, float quarantine enforced at build surface, replay tooling boundary documented, authority hierarchy explicit | file: `docs/system_architecture_disclosure.md`; file: `docs/architecture/workspace.md`; file: `VERIFICATION_GUIDE.md` |
| Documentation depth | 83 | extensive normative contracts, verification guide, replay subsystem docs, demo lifecycle, evidence packaging, retained release bundles across 7 releases | file: `docs/README.md`; file: `docs/replay/tooling.md`; file: `docs/verification/releases/index.md` |
| Documentation organization | 75 | clear authority hierarchy (VERIFICATION_GUIDE > README > RELEASE_SURFACE), but large doc tree with some overlap between architecture docs and replay docs; `docs/README.md` index is comprehensive | file: `docs/README.md`; file: `docs/RELEASE_SURFACE.md` |
| Repository presentation | 82 | root README is concise with correct authority routing table; release surface classification is accurate and narrowed; clean root directory | file: `README.md` |
| Developer onboarding | 76 | Quick Start section works (`make gate` confirmed), authority routing table present, but no CONTRIBUTING.md, no architecture decision records, large document tree requires orientation | file: `README.md`; command-output: `make gate => VERIFICATION PASSED` |
| Conceptual coherence | 85 | consistent model: deterministic execution analysis infrastructure, clear separation of release/experimental/reference, normative vs advisory signals well-defined | file: `VERIFICATION_GUIDE.md §1.1`; file: `docs/RELEASE_SURFACE.md` |
| Research / innovation value | 82 | formal verification of DSP kernels, deterministic build reproducibility, cryptographic hash-locked validation, hardware-coupled firmware capture path, divergence classification taxonomy | file: `verify_kani.sh`; file: `docs/demos/demo_evidence_packaging.md` |
| OSS trustworthiness signals | 78 | MIT license, CODEOWNERS, pinned toolchain, locked dependencies, reproducibility checks, but no CI badge, no published releases/tags visible, no CONTRIBUTING.md | file: `LICENSE`; file: `CODEOWNERS`; file: `Cargo.lock` |

---

## 3. Implementation Reality Map

| Component | Crate/Location | Implemented | Classification | Notes |
|---|---|---|---|---|
| DPW4 oscillator core | `crates/dpw4/src/lib.rs` | yes | Release | I64F64 fixed-point phase engine, 5 waveform shapes |
| `precision` CLI | `crates/dpw4/src/bin/precision.rs` + `precision/mod.rs` | yes | Release | Commands: record, replay, diff, envelope |
| `sig-util` CLI | `crates/dpw4/src/bin/sig_util.rs` + `sig_util/mod.rs` | yes | Release | Commands: validate, generate, artifacts, inspect, header-audit, verify |
| `substrate_probe` | `crates/dpw4/src/bin/substrate_probe.rs` | yes | Reference | Audit/probe workflow support tool |
| `geom-signal` | `crates/geom-signal/` | yes | Release | sin, cos, sqrt, atan2 for I64F64 |
| `geom-spatial` | `crates/geom-spatial/` | yes | Release | Vector3 spatial math |
| `replay-core` | `crates/replay-core/` | yes | Release | RPL0 artifact format, parser, encoder |
| `replay-host` | `crates/replay-host/` | yes | Experimental | diff (bounded 1.5.0 slice released), import-interval-csv, validate-interval-csv |
| `replay-cli` | `crates/replay-cli/` | yes | Experimental | Scaffolding only |
| `replay-embed` | `crates/replay-embed/` | yes | Experimental | Embedded replay support |
| `replay-fw-f446` | `crates/replay-fw-f446/` | yes | Experimental | STM32 F446 firmware capture |
| `audit-float-boundary` | `crates/audit-float-boundary/` | yes | Reference | Float quarantine audit utility |
| `xtask` | `crates/xtask/` | yes | Reference | Workflow orchestration |
| Python `artifact_tool.py` | `scripts/artifact_tool.py` | yes | Reference | Capture, inspection, verification, hashing |
| Python `artifact_diff.py` | `scripts/artifact_diff.py` | yes | Reference | Divergence localization |

---

## 4. CLI Surface Inventory

### Rust Binaries

| Command | Entrypoint | Type | Runnable | Verification Path | Classification |
|---|---|---|---|---|---|
| `precision record` | `crates/dpw4/src/bin/precision/mod.rs` | rust-bin | observed | `precision_authoritative_surface.rs` | Release |
| `precision replay` | `crates/dpw4/src/bin/precision/mod.rs` | rust-bin | observed | `precision_authoritative_surface.rs` | Release |
| `precision diff` | `crates/dpw4/src/bin/precision/mod.rs` | rust-bin | observed | `precision_authoritative_surface.rs` | Release |
| `precision envelope` | `crates/dpw4/src/bin/precision/mod.rs` | rust-bin | observed | `precision_authoritative_surface.rs` | Release |
| `sig-util validate` | `crates/dpw4/src/bin/sig_util/validate.rs` | rust-bin | observed | `make gate` | Release |
| `sig-util generate` | `crates/dpw4/src/bin/sig_util/generate.rs` | rust-bin | observed | n/a | Reference |
| `sig-util artifacts` | `crates/dpw4/src/bin/sig_util/artifacts.rs` | rust-bin | observed | `make gate` (internally) | Reference |
| `sig-util inspect` | `crates/dpw4/src/bin/sig_util/inspect.rs` | rust-bin | observed | n/a | Reference |
| `sig-util header-audit` | `crates/dpw4/src/bin/sig_util/verify.rs` | rust-bin | observed | n/a | Reference |
| `sig-util verify` | `crates/dpw4/src/bin/sig_util/verify.rs` | rust-bin | observed | n/a | Reference |
| `substrate_probe` | `crates/dpw4/src/bin/substrate_probe.rs` | rust-bin | observed | `make conformance-audit` | Reference |
| `replay-host diff` | `crates/replay-host/src/main.rs` | rust-bin | observed | `make replay-tests` | Reference (1.5.0 bounded release) |
| `replay-host import-interval-csv` | `crates/replay-host/src/main.rs` | rust-bin | observed | n/a | Experimental |
| `replay-host validate-interval-csv` | `crates/replay-host/src/main.rs` | rust-bin | observed | n/a | Experimental |

### Python Tools

| Command | Entrypoint | Type | Runnable | Classification |
|---|---|---|---|---|
| `artifact_tool.py verify` | `scripts/artifact_tool.py` | python-tool | observed | Reference |
| `artifact_diff.py` | `scripts/artifact_diff.py` | python-tool | observed | Reference |
| `check_release_bundle.py` | `scripts/check_release_bundle.py` | python-tool | observed | Reference |
| `check_doc_links.py` | `scripts/check_doc_links.py` | python-tool | observed | Reference |

### Makefile Entrypoints (key targets)

| Target | Role | Classification |
|---|---|---|
| `make gate` | Canonical operator-facing release gate | Release |
| `make gate-full` | Supplementary validation | Reference |
| `make release-1.6.0` | Retained-record orchestration for 1.6.0 | Release |
| `make release-bundle-check` | Bundle coherence checker | Release |
| `make demo-evidence-package` | Packaged proof route | Reference |
| `make doc-link-check` | Documentation link integrity | Reference |
| `make replay-tests` | Replay test suite | Reference |
| `make ci-local` | Local CI composite | Reference |
| `make conformance-audit` | Substrate audit workflow | Reference |

---

## 5. Derived Release Surface

| Capability | Implemented | Classification | User-Facing Path | Verification Path | Evidence | Confidence |
|---|---|---|---|---|---|---|
| Deterministic validation gate | yes | Release | `make gate` | `make gate` => VERIFICATION PASSED | command-output: observed-this-session | direct |
| `precision` CLI (record/replay/diff/envelope) | yes | Release | `precision <command>` | `precision_authoritative_surface.rs` (20 tests) | file: retained evidence; command-output: observed-this-session | direct |
| Hash-locked determinism (6 normative traces) | yes | Release | `sig-util validate --mode quick` | `make gate` | command-output: 6/6 PASS, observed-this-session | direct |
| Float quarantine | yes | Release | build-surface enforcement | `cargo check --workspace --no-default-features` + thumbv7em | command-output: observed-this-session | direct |
| Kani formal verification (Tier-1) | yes | Release | `bash verify_kani.sh` | 22 harness logs | retained-evidence: `kani_evidence.txt` | retained-evidence |
| RPL0 artifact format (version 0+1) | yes | Release | parser/encoder in `replay-core` | `make replay-tests` (adversarial + v1 corpus) | command-output: retained evidence | direct |
| `replay-host diff` (bounded 1.5.0 slice) | yes | Reference | `replay-host diff <a> <b>` | `make replay-tests` | retained-evidence: `docs/verification/releases/1.5.0/` | retained-evidence |
| Demo evidence packaging | yes | Reference | `make demo-evidence-package` | self-verifying | command-output: observed-this-session | direct |
| Build reproducibility | yes | Reference | `verify_release_repro.sh` | dual-build SHA-256 comparison | retained-evidence: `release_reproducibility.txt` | retained-evidence |
| Firmware capture/import (STM32 F446) | yes | Experimental | hardware-dependent | `supplemental/firmware_evidence.md` | retained-evidence | retained-evidence |
| Schema-aware Rust replay | no | Experimental | n/a | n/a | n/a | direct |
| Broader replay-host capabilities | partial | Experimental | n/a | n/a | n/a | direct |

---

## 6. Claim-Reality Gap Register

### Claim 1: "make gate passes"

```
Claim: "make gate passes" as canonical release gate
Supporting Evidence:
- command-output: make gate => VERIFICATION PASSED (observed-this-session)
- file: docs/verification/releases/1.6.0/make_gate.txt (retained)
Status: exact
Impact: release-risk
Confidence: direct
Notes: live execution confirmed. Retained evidence matches.
```

### Claim 2: "Narrowed Tier-1 claim for primary precision CLI surface"

```
Claim: narrowed retained release record for the primary precision CLI surface
Supporting Evidence:
- file: docs/verification/releases/1.6.0/README.md (defines scope)
- file: docs/verification/releases/1.6.0/precision_authoritative_surface_test_evidence.txt (20 tests pass)
- file: crates/dpw4/tests/precision_authoritative_surface.rs
Status: exact
Impact: credibility
Confidence: direct
Notes: scope is explicitly bounded. Claim matches evidence.
```

### Claim 3: "No replay-host scope expansion"

```
Claim: no replay-host scope expansion in 1.6.0
Supporting Evidence:
- file: docs/RELEASE_SURFACE.md (replay-host diff classified as historical 1.5.0 slice)
- file: CHANGELOG.md ("No replay-host or firmware scope expansion")
Status: exact
Impact: credibility
Confidence: direct
Notes: consistent across all authority documents.
```

### Claim 4: "Bounded supporting firmware capture/import evidence"

```
Claim: firmware evidence is bounded supplemental only, not a firmware release
Supporting Evidence:
- file: docs/verification/releases/1.6.0/supplemental/firmware_evidence.md (explicit disclaimer)
- file: docs/verification/releases/1.6.0/README.md ("firmware release claim" in NOT-claimed list)
- file: docs/RELEASE_SURFACE.md ("does not promote a firmware release for 1.6.0")
Status: exact
Impact: release-risk
Confidence: direct
Notes: triple-consistent across all documents.
```

### Claim 5: Python tooling is "released operator tooling"

```
Claim: released replay-facing operator tooling is the Python toolchain
Supporting Evidence:
- file: docs/replay/tooling.md (explicit statement)
- file: docs/system_architecture_disclosure.md (explicit statement)
- file: docs/RELEASE_SURFACE.md (classifies as "Support / Reference / Historical Only")
Status: partial
Impact: credibility
Confidence: inference
Notes: docs/replay/tooling.md and system_architecture_disclosure.md call Python tooling "released operator tooling", but docs/RELEASE_SURFACE.md classifies artifact_tool.py and artifact_diff.py as "retained support/reference tooling, not canonical 1.6.0 operator surface". These are not contradictory but could confuse a cold reviewer about whether the Python tools are "released" or "reference". The Python tools are historically released but explicitly not the canonical 1.6.0 operator surface.
```

### Claim 6: "make demo-evidence-package passes"

```
Claim: demo evidence packaging passes
Supporting Evidence:
- command-output: make demo-evidence-package => PASS (observed-this-session)
- file: docs/verification/releases/1.6.0/make_demo_evidence_package.txt (retained)
Status: exact
Impact: release-risk
Confidence: direct
```

### Claim 7: "make release-bundle-check VERSION=1.6.0 passes"

```
Claim: release bundle coherence check passes
Supporting Evidence:
- command-output: make release-bundle-check VERSION=1.6.0 => PASS (observed-this-session)
Status: exact
Impact: release-risk
Confidence: direct
```

### Claim 8: "make doc-link-check passes"

```
Claim: documentation link integrity check passes
Supporting Evidence:
- command-output: make doc-link-check => PASS (observed-this-session)
- file: docs/verification/releases/1.6.0/make_doc_link_check.txt (retained)
Status: exact
Impact: release-risk
Confidence: direct
```

---

## 7. Documentation Topology Map

| Document | Purpose | Category | Public | Overlap | Overlap Severity | Authority Level |
|---|---|---|---|---|---|---|
| `VERIFICATION_GUIDE.md` | Release contract and verification protocol | normative | public | none | — | canonical |
| `README.md` | Entry routing | entry | public | none | — | canonical |
| `docs/RELEASE_SURFACE.md` | Release-surface classification | release-classification | public | none | — | canonical |
| `CHANGELOG.md` | Release-facing shipped summary | release-classification | public | minor with RELEASE_SURFACE | minor | supporting |
| `docs/README.md` | Documentation index | index | public | none | — | canonical |
| `docs/MATH_CONTRACT.md` | Arithmetic and signal-path contract | normative | public | none | — | canonical |
| `docs/spec/rpl0_artifact_contract.md` | RPL0 binary artifact format | normative | public | none | — | canonical |
| `docs/authority/cli_contract.md` | CLI schema and operational contract | normative | public | none | — | canonical |
| `docs/replay/tooling.md` | Replay-tooling boundary | release-classification | public | minor with RELEASE_SURFACE on Python tooling language | minor | supporting |
| `docs/system_architecture_disclosure.md` | System architecture overview | deep-architecture | public | minor with replay/tooling.md | minor | supporting |
| `docs/verification/releases/index.md` | Retained release evidence index | index | public | none | — | canonical |
| `docs/verification/releases/1.6.0/README.md` | 1.6.0 release bundle summary | release-classification | public | none | — | canonical |
| `docs/verification/releases/1.6.0/index.md` | 1.6.0 bundle routing | index | public | none | — | supporting |
| `docs/verification/CLI_SURFACE_EVIDENCE.md` | Historical CLI promotion evidence | historical-audit | public | none | — | historical |
| `docs/demos/demo_evidence_packaging.md` | Packaged proof route | workflow | public | none | — | supporting |
| `docs/replay/README.md` | Replay subsystem scope index | index | public | none | — | supporting |
| `docs/governance/DESIGN_AXIOMS.md` | Design principles | normative | public | none | — | supporting |
| `docs/wip/README.md` | WIP routing | index | internal | none | — | supporting |
| `docs/audits/README.md` | Audit ledger index | index | public | none | — | supporting |

---

## 8. Terminology Consistency Register

| File | Line/Context | Term | Classification | Severity | Notes |
|---|---|---|---|---|---|
| `docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md` | filename | `_v1.md` | format-version filename | low | This is a capture contract filename, not an artifact-spec filename. The `v1` denotes the contract version. NAM-003 does not apply. Compliant. |
| `docs/replay/FW_F446_CAPTURE_v1.md` | filename | `_v1.md` | format-version filename | low | Historical board-capture contract. `v1` denotes contract version. Compliant. |
| `docs/replay/FW_F446_CAPTURE_v0.md` | filename | `_v0.md` | format-version filename | low | Legacy capture contract. Compliant. |
| `docs/replay/HOST_REPLAY_v0.md` | filename | `_v0.md` | format-version filename | low | Host replay semantics for format version 0 boundary. Compliant. |
| `docs/replay/WIRE_FORMAT_v0.md` | filename | `_v0.md` | format-version filename | low | Wire format doc for v0. Compliant. |
| `docs/MATH_CONTRACT.md` | title and intro | "v1.2.1" | lock-release identifier | low | Terminology block present; explicitly disambiguates document revision vs release version. Compliant (NAM-005 satisfied). |
| `docs/spec/rpl0_artifact_contract.md` | intro | versioning terminology | format-version | low | Explicit "Versioning Terminology" section present. Compliant (NAM-005 satisfied). |
| `VERIFICATION_GUIDE.md` | §6.5 | `replay_manifest_v1.txt` / `replay_manifest_v0.txt` | format-version filename | low | Used for RPL0 format version context. Compliant. |
| `crates/dpw4/src/bin/sig_util/mod.rs` | line 15 | "1.6.0 high-integrity contract" | release-version | low | Clear release version reference. Compliant. |

No NAM-001, NAM-002, NAM-003, or NAM-006 violations detected.
No axis-mixing or ambiguous bare `vX` usage found in normative or release-facing documents.

---

## 9. Strengths

1. **Exemplary verification discipline**: 22 Tier-1 + 4 Tier-2 Kani formal verification harnesses, hash-locked determinism gate, adversarial parser test suite, 20-test authoritative surface integration suite. Evidence: `VERIFICATION_GUIDE.md §3`; `docs/verification/releases/1.6.0/kani_evidence.txt`; `docs/verification/releases/1.6.0/precision_authoritative_surface_test_evidence.txt`.

2. **Honest, narrowed release claims**: the 1.6.0 release explicitly documents what is NOT claimed, including full contract closure, exhaustive coverage, and firmware release. This is rare and credibility-enhancing. Evidence: `docs/verification/releases/1.6.0/README.md` ("What is NOT claimed" section).

3. **Strong build reproducibility**: dual-build identity check with `SOURCE_DATE_EPOCH`, pinned toolchain `1.91.1`, locked dependencies, and bit-exact SHA-256 comparison. Evidence: `docs/verification/releases/1.6.0/release_reproducibility.txt`.

4. **Clear authority hierarchy**: `VERIFICATION_GUIDE.md` > `README.md` > `docs/RELEASE_SURFACE.md` is explicit, consistent, and maintained. Evidence: `VERIFICATION_GUIDE.md` ("Release Contract" section); `docs/RELEASE_SURFACE.md` ("This document is a routing and classification aid... It is not the release contract").

5. **Retained release evidence chain**: 7 releases (`1.2.0` through `1.6.0`) with retained bundles, including a documented evidence gap for `1.3.0`. Evidence: `docs/verification/releases/index.md`.

6. **Float quarantine enforcement**: build-surface float quarantine verified on both host and embedded targets. Evidence: command-output `cargo check --workspace --no-default-features => ok`; `cargo check -p dpw4 --no-default-features --target thumbv7em-none-eabihf => ok`.

7. **Clean terminology**: normative specs include versioning terminology blocks. No axis-mixing violations found. Evidence: `docs/MATH_CONTRACT.md` ("Versioning Terminology" section); `docs/spec/rpl0_artifact_contract.md` ("Versioning Terminology" section).

---

## 10. Weaknesses

### W-1: Python tooling classification language inconsistency

**Finding:** `docs/replay/tooling.md` and `docs/system_architecture_disclosure.md` describe Python tooling as "released operator tooling" or "released replay-facing operator tooling," while `docs/RELEASE_SURFACE.md` classifies `artifact_tool.py` and `artifact_diff.py` as "retained support/reference tooling; not canonical 1.6.0 operator surface."

**Evidence:**
- file: `docs/replay/tooling.md` line 2–4
- file: `docs/RELEASE_SURFACE.md` lines 44–46

**Impact:** credibility
**Severity:** medium
**Classification:** documentation
**Confidence:** direct

**Recommended Direction:** Harmonize the language. Either consistently call Python tools "historically released, currently reference" or adjust `docs/replay/tooling.md` to use the same "support/reference" language as `docs/RELEASE_SURFACE.md`.

### W-2: No CONTRIBUTING.md or architecture decision records

**Finding:** The repository lacks a `CONTRIBUTING.md` file and formal architecture decision records (ADRs). For a project with this level of governance sophistication, the absence is noticeable.

**Evidence:**
- file: `README.md` (no contributing section)
- absence of `CONTRIBUTING.md`

**Impact:** onboarding
**Severity:** low
**Classification:** documentation
**Confidence:** direct

**Recommended Direction:** Add a minimal `CONTRIBUTING.md` covering build prerequisites, test expectations, and the authority hierarchy.

### W-3: `docs/demos/demo_evidence_packaging.md` described as "canonical packaged proof route" in `docs/README.md`

**Finding:** `docs/README.md` index describes `demo_evidence_packaging.md` as "canonical packaged proof route and retained bundle for the completed replay pipeline." This phrasing could imply it is part of the release contract, but `docs/RELEASE_SURFACE.md` explicitly classifies it as "not the canonical 1.6.0 operator release surface."

**Evidence:**
- file: `docs/README.md` line 19
- file: `docs/RELEASE_SURFACE.md` lines 56–59

**Impact:** credibility
**Severity:** medium
**Classification:** documentation
**Confidence:** direct

**Recommended Direction:** Downgrade the `docs/README.md` index description to "reference packaged proof route" or add a qualifier such as "for the replay demo pipeline (not the 1.6.0 release contract)."

### W-4: Large accumulated artifacts directory

**Finding:** The `artifacts/` directory contains numerous timestamped run directories and demo output directories. While these serve development and evidence purposes, they add significant repository size for cloners.

**Evidence:**
- directory listing: `artifacts/` (20+ timestamped directories, multiple demo directories)

**Impact:** maintainability
**Severity:** low
**Classification:** implementation
**Confidence:** direct

**Recommended Direction:** Consider a `.gitattributes` LFS policy or periodic archival of older timestamped artifacts outside the retained release evidence chain.

---

## 11. Remediation Priorities

| Priority | Finding | Effort | Impact |
|---|---|---|---|
| 1 | W-1: Harmonize Python tooling classification language | low | credibility |
| 2 | W-3: Qualify `demo_evidence_packaging.md` description in docs index | low | credibility |
| 3 | W-2: Add CONTRIBUTING.md | low | onboarding |
| 4 | W-4: Artifact directory management | medium | maintainability |

---

## 12. Evidence and Unknowns

### Direct Evidence (observed-this-session)

| Item | Command | Result |
|---|---|---|
| `make gate` | `cargo run --locked --release -p dpw4 --features cli --bin sig-util -- validate --mode quick` | VERIFICATION PASSED |
| `make demo-evidence-package` | `cargo run --quiet -p xtask -- workflow demo-evidence-package` | PASS |
| `make doc-link-check` | `cargo run --quiet -p xtask -- workflow doc-link-check` | PASS |
| `make release-bundle-check VERSION=1.6.0` | `python3 scripts/check_release_bundle.py --version 1.6.0` | PASS |
| Float quarantine (host) | `cargo check --workspace --no-default-features` | ok |
| Float quarantine (embedded) | `cargo check -p dpw4 --no-default-features --target thumbv7em-none-eabihf` | ok |
| Workspace tests | `cargo test --workspace --locked` | all pass (65+ tests, 4 ignored) |

### Retained Evidence (not re-executed)

| Item | Location | Status |
|---|---|---|
| Kani Tier-1 harnesses (22) | `docs/verification/releases/1.6.0/kani_evidence.txt` | retained-evidence, not re-executed |
| Kani Tier-2 harnesses (4) | not retained for 1.6.0 | excluded (documented) |
| Build reproducibility | `docs/verification/releases/1.6.0/release_reproducibility.txt` | retained-evidence |
| Authoritative surface tests | `docs/verification/releases/1.6.0/precision_authoritative_surface_test_evidence.txt` | retained-evidence |
| Firmware capture evidence | `docs/verification/releases/1.6.0/supplemental/firmware_evidence.md` | retained-evidence, supplemental only |

### Unknowns

| Item | Status | Notes |
|---|---|---|
| CI/CD pipeline | unknown | No CI configuration files observed in workspace root; CI evidence references exist but no `.github/workflows/` or equivalent |
| Published release tags | unknown | No git tag evidence gathered during this audit |
| External dependency audit | unknown | Dependencies are locked but no `cargo audit` or supply-chain check observed |

---

## Release Readiness Assessment

### Decision: **Ready**

### Release candidate: 1.6.0

### Release contract status

| Gate | Status | Evidence |
|---|---|---|
| `make gate` | PASS | observed-this-session |
| `make demo-evidence-package` | PASS | observed-this-session |
| `make doc-link-check` | PASS | observed-this-session |
| Retained bundle exists | PASS | `docs/verification/releases/1.6.0/` |
| `make release-bundle-check VERSION=1.6.0` | PASS | observed-this-session |
| Kani preflight retained | PASS | `docs/verification/releases/1.6.0/kani_evidence.txt` |

### Authority alignment

- `VERIFICATION_GUIDE.md` names `1.6.0` as active release baseline: **aligned**
- `VERIFICATION_GUIDE.md` routes to `docs/verification/releases/1.6.0/`: **aligned**
- `README.md` routes to `VERIFICATION_GUIDE.md`: **aligned**
- `docs/RELEASE_SURFACE.md` names `1.6.0`: **aligned**
- `CHANGELOG.md` has `[1.6.0]` entry: **aligned**
- Workspace version is `1.6.0`: **aligned**
- Lockfile versions are `1.6.0`: **aligned**

### Retained evidence status

All 10 non-firmware required files present and non-empty:
`README.md`, `index.md`, `cargo_check_dpw4_thumb_locked.txt`, `kani_evidence.txt`,
`make_demo_evidence_package.txt`, `make_doc_link_check.txt`, `make_gate.txt`,
`make_replay_tests.txt`, `precision_authoritative_surface_test_evidence.txt`,
`release_reproducibility.txt`.

Supplemental firmware evidence properly scoped under `supplemental/firmware_evidence.md`.

### Overclaim check

- No experimental component presented as released: **pass**
- No firmware release claim: **pass**
- No replay-host scope expansion claimed: **pass**
- `[Unreleased]` section contains only maintenance items: **pass**
- Python tooling language inconsistency (W-1) is a minor credibility concern, not an overclaim

### Exact blockers: none

### Smallest authoritative next step

Harmonize W-1 (Python tooling language) and W-4 (demo evidence packaging description) for cold-reviewer clarity. Neither blocks the tag.

### Tag readiness: **Ready to tag**

---

## Final Questions

### 1. What executable capabilities are actually implemented today?

- A deterministic phase-engine oscillator core (`dpw4`) with 5 waveform shapes (saw, pulse, triangle, sine, square), I64F64 fixed-point arithmetic, and hash-locked validation
- An authoritative `precision` CLI with `record`, `replay`, `diff`, and `envelope` commands
- A `sig-util` utility CLI with `validate`, `generate`, `artifacts`, `inspect`, `header-audit`, and `verify` commands
- An RPL0 artifact format (v0 and v1) with parser, encoder, and adversarial test suite
- A bounded `replay-host diff` for the retained `artifacts/rpl0/` proof corpus (1.5.0 historical)
- An experimental firmware capture path (`replay-fw-f446`) for STM32 F446 with interval CSV capture contract
- A Python tooling layer for artifact inspection, verification, hashing, comparison, and divergence localization
- Formal verification infrastructure: 22 Tier-1 + 4 Tier-2 Kani harnesses
- Build reproducibility infrastructure: dual-build identity checks, pinned toolchain, locked dependencies

### 2. How accurately does the repository communicate those capabilities?

Very accurately. The authority hierarchy (`VERIFICATION_GUIDE.md` > `README.md` > `docs/RELEASE_SURFACE.md`) is explicit and consistent. Release claims are narrowed and honest, with explicit "What is NOT claimed" sections. Experimental and reference components are clearly demarcated. The only minor imprecision is the Python tooling language inconsistency (W-1) and the demo evidence packaging description (W-3), neither of which constitutes overclaim.

### 3. What single sprint would most improve repository credibility and clarity?

Harmonize the Python tooling language across `docs/replay/tooling.md`, `docs/system_architecture_disclosure.md`, and `docs/RELEASE_SURFACE.md`; qualify the demo evidence packaging description in `docs/README.md`; and add a minimal `CONTRIBUTING.md`. This is all low-effort documentation cleanup that would eliminate the remaining cold-reviewer friction without any code changes.
