# Repository Health Audit — 1.2.2

**Audit date:** 2026-03-25
**Auditor:** automated agent (Claude Opus 4.6)
**Workspace version:** 1.2.2
**Toolchain:** rustc 1.91.1 (pinned via rust-toolchain.toml)
**Branch:** feat/release-1.2.2
**HEAD:** 6111fde

---

## 1. Audit Scope Summary

This audit covers the `precision-signal` repository at workspace version 1.2.2.
Evidence is drawn from executable code, build/test/verification commands, normative
specs, and public documentation.

Scope:
- Cargo workspace (10 crates, 4 default members)
- Two Rust CLI binaries: `precision`, `header_audit`
- Two released Python operator tools: `artifact_tool.py`, `artifact_diff.py`
- One experimental Rust binary: `replay-host`
- Firmware crate: `replay-fw-f446`
- Verification workflows: `make gate`, `make replay-tests`, `make release-bundle-check`
- Documentation set under `docs/`
- Retained release evidence under `docs/verification/releases/1.2.2/`

Prior audit: The existing `docs/audits/repository_health_1.2.2.md` was a terse
change record (29 lines) from a documentation remediation pass, not a structured
health audit baseline. This audit replaces it as the first full baseline for 1.2.2.

---

## 2. Metric Scores

| Metric | Score | Justification | Evidence |
|---|---:|---|---|
| Engineering integrity | 88 | Pinned toolchain, `#![forbid(unsafe_code)]`, determinism hashes locked to semver bumps | file: `rust-toolchain.toml`; file: `VERIFICATION_GUIDE.md` §1.2 |
| Determinism / reproducibility | 90 | 7 golden hashes pass, 5/5 hardware runs identical, fixture-drift gate | command-output: `make gate => VERIFICATION PASSED`; file: `docs/verification/releases/1.2.2/sha256_summary.txt` |
| Specification quality | 82 | RPL0 contract has versioning terminology block, byte-level layout, hard invariants; MATH_CONTRACT is dense and locked | file: `docs/spec/rpl0_artifact_contract.md`; file: `docs/MATH_CONTRACT.md` |
| Verification discipline | 87 | Multi-tier gate (cargo test + parser suites + tool regression + determinism gate + fixture drift + doc-link); Kani harness runner retained | command-output: `make gate => PASS`; command-output: `make replay-tests => PASS` |
| Codebase maintainability | 78 | Workspace is well-factored (core crates zero-dep); some crates are stubs; 10 members with only 4 default | file: `Cargo.toml` workspace members |
| Architecture clarity | 80 | Clear release/experimental boundary in RELEASE_SURFACE.md; workspace.md routes well; system_architecture_disclosure is accurate | file: `docs/RELEASE_SURFACE.md`; file: `docs/architecture/workspace.md` |
| Documentation depth | 80 | Normative specs are thorough; replay docs cover operator workflows; architecture docs exist for most surfaces | file: `docs/README.md` (index routing) |
| Documentation organization | 76 | Good top-level routing through docs/README.md; some deep docs (performance/) are frozen at v1.0.0-rc5; internal/ has normalization notes | file: `docs/README.md`; file: `docs/architecture/performance/` |
| Repository presentation | 82 | README is terse and correctly routes; no inflated claims; release surface is distinct from experimental | file: `README.md` |
| Developer onboarding | 84 | "First 5 Minutes" block with `make gate`; VERIFICATION_GUIDE anchors the release contract | file: `README.md` §First 5 Minutes |
| Conceptual coherence | 85 | Consistent use of "deterministic execution analysis infrastructure"; normative/advisory distinction maintained | file: `VERIFICATION_GUIDE.md` §1.1; file: `docs/README.md` |
| Research / innovation value | 75 | Phase-domain fixed-point oscillator with formal proofs; hardware-backed determinism; novel but narrow domain | file: `docs/MATH_CONTRACT.md`; file: `verify_kani.sh` |
| OSS trustworthiness signals | 80 | MIT license, pinned toolchain, hash-locked releases, retained evidence bundles, doc-link CI; no CI badge or GitHub Actions visible on branch | file: `LICENSE`; file: `docs/verification/CI_EVIDENCE.md` |

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
| `replay-host diff` | yes | Experimental | `cargo run -p replay-host -- diff ...` | not in release gate | file: `crates/replay-host/src/main.rs`; command-output: `first divergence at frame 0` |
| `replay-fw-f446` | yes | Release | `make flash-ur` | retained evidence: `docs/verification/releases/1.2.2/firmware_release_evidence.md` | file: `crates/replay-fw-f446/` |

---

## 4. CLI Surface Inventory

| Command | Entrypoint | Type | Runnable | Verification Path | Classification |
|---|---|---|---|---|---|
| `precision validate --mode quick` | `crates/dpw4/src/bin/precision.rs` | rust-bin | observed | `make gate` | Release |
| `precision validate --mode full` | `crates/dpw4/src/bin/precision.rs` | rust-bin | observed (not exercised this session) | none observed | Release |
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
| Formal verification (Kani) | yes | Release | `bash verify_kani.sh` | retained from prior evidence | file: `verify_kani.sh`; doc: `docs/verification/CI_EVIDENCE.md` | retained-evidence |

---

## 6. Claim-Reality Gap Register

| # | Claim | Source | Status | Impact | Confidence | Notes |
|---|---|---|---|---|---|---|
| 1 | `make gate` => VERIFICATION PASSED | README.md §First 5 Minutes | exact | — | direct | command-output: `make gate => VERIFICATION PASSED` (observed-this-session) |
| 2 | Released replay-facing operator tooling is the Python toolchain | docs/replay/tooling.md | exact | — | direct | `artifact_tool.py` and `artifact_diff.py` exercised via `make replay-tool-tests` |
| 3 | Rust replay is experimental and not part of release surface | docs/RELEASE_SURFACE.md, docs/replay/tooling.md | exact | — | direct | `replay-host` description: "outside the v1 release surface"; not in `make gate` |
| 4 | Hardware capture re-executed for 1.2.2 | docs/verification/releases/1.2.2/firmware_release_evidence.md | exact | — | retained-evidence | 5/5 runs identical, SHA256 `f79e71d...` consistent across all retained files |
| 5 | Deterministic repeatability: PASS (5/5 runs identical) | docs/verification/releases/1.2.2/firmware_release_evidence.md | exact | — | retained-evidence | sha256_summary.txt shows 6 identical hashes including baseline |
| 6 | MATH_CONTRACT frozen at v1.2.1 | docs/MATH_CONTRACT.md title | partial | credibility | direct | Workspace is 1.2.2; contract title says v1.2.1; status says LOCKED. Likely intentional (no math changes in 1.2.2) but the version gap is visible to reviewers |
| 7 | Spec docs frozen at v1.0.0-rc5 | docs/spec/dpw_gain.md, docs/spec/oscillator_api.md, docs/spec/header_layout_addendum.md | partial | onboarding | direct | Document version headers say v1.0.0-rc5 while workspace is 1.2.2. Content may still be accurate but version label is stale |

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
| docs/architecture/workspace.md | Workspace framing | deep-architecture | public | none | supporting |
| docs/system_architecture_disclosure.md | Architecture overview | descriptive | public | minor (architecture overview) | supporting |
| docs/verification/build_reproducibility.md | Build identity checks | workflow | public | none | supporting |
| docs/verification/CI_EVIDENCE.md | Historical CI evidence | historical-audit | public | none | historical |
| docs/verification/CLI_SURFACE_EVIDENCE.md | CLI promotion evidence | workflow | public | none | supporting |
| docs/verification/releases/1.2.2/ | Retained release evidence bundle | normative | public | none | canonical |
| docs/spec/dpw_gain.md | Gain spec | normative | public | none | supporting |
| docs/spec/oscillator_api.md | Oscillator dispatch contract | normative | public | none | supporting |
| docs/architecture/performance/ | Performance benchmarking docs | roadmap | internal | none | historical |
| docs/internal/ | Internal normalization notes | reference | internal | none | supporting |

---

## 8. Terminology Consistency Register

| Rule | File | Line | Classification | Severity | Evidence | Notes |
|---|---|---:|---|---|---|---|
| NAM-004 | docs/spec/rpl0_artifact_contract.md | 52 | ambiguous-version | info | "v1 parsing path" | Adequately disambiguated by Versioning Terminology block at top of same file |
| NAM-004 | docs/spec/dpw_gain.md | 2 | ambiguous-version | warn | "Version: v1.0.0-rc5" | Document version vs release version unclear; stale label |
| NAM-004 | docs/spec/oscillator_api.md | 2 | ambiguous-version | warn | "Version: v1.0.0-rc5" | Same as above |
| NAM-004 | docs/spec/header_layout_addendum.md | 2 | ambiguous-version | warn | "Version: v1.0.0-rc5" | Same as above |
| NAM-004 | docs/spec/oscillator_api.md | 6 | ambiguous-version | info | "v1.x line" | Refers to release version range; context adequate |
| NAM-005 | docs/spec/dpw_gain.md | — | missing-terminology-block | warn | No versioning terminology block | Compare to rpl0_artifact_contract.md which has one |
| NAM-005 | docs/spec/oscillator_api.md | — | missing-terminology-block | warn | No versioning terminology block | Same |
| NAM-005 | docs/spec/header_layout_addendum.md | — | missing-terminology-block | warn | No versioning terminology block | Same |

No NAM-001 (artifact vX), NAM-002 (replay vX), NAM-003 (_vX.md artifact spec),
or NAM-006 (capability statement) violations found in public-facing documents.
`FW_F446_CAPTURE_v1.md` uses `v1` in filename but refers to capture contract
version, not artifact spec filename — NAM-003 does not apply.

---

## 9. Strengths

1. **Determinism discipline is exemplary.** 7 golden hashes locked to semver bumps,
   hardware-backed 5/5 repeatability, fixture-drift gate prevents silent regression.
   Evidence: `make gate => VERIFICATION PASSED` (observed-this-session);
   `docs/verification/releases/1.2.2/sha256_summary.txt`.

2. **Release/experimental boundary is well-drawn.** `RELEASE_SURFACE.md` explicitly
   classifies every surface; `replay-host` description says "outside the v1 release
   surface"; `docs/replay/tooling.md` reinforces this. No inflation observed.

3. **Verification gate chain is layered and runnable.** `make ci-local` chains
   `doc-link-check fw test replay-tests gate fixture-drift-check` — covering
   doc integrity, compilation, unit tests, parser/tool regression, determinism,
   and fixture stability in a single command.

4. **Documentation routing is clear.** README → docs/README.md → RELEASE_SURFACE.md
   and VERIFICATION_GUIDE.md. Normative vs descriptive distinction is maintained
   consistently across index pages.

5. **Normative specs are binding and precise.** `rpl0_artifact_contract.md` has
   byte-level layout, versioning terminology block, and hard invariants.
   `MATH_CONTRACT.md` has code-line references and Kani proof citations.

6. **Retained release evidence is structured.** `docs/verification/releases/1.2.2/`
   contains firmware evidence, hash inventory, manifest, and SHA256 summary.
   `make release-bundle-check VERSION=1.2.2` validates bundle coherence.

---

## 10. Weaknesses

### Finding W-1: Retained manifest has absolute non-portable run_dir

The `replay_manifest_v1.txt` in the 1.2.2 release bundle contains
`run_dir=artifacts/replay_runs/run_20260325T184038Z`,
an absolute host-specific path. `make release-bundle-check` flags this as WARN.

Evidence:
- file: `docs/verification/releases/1.2.2/replay_manifest_v1.txt` line 13
- command-output: `make release-bundle-check VERSION=1.2.2 => WARN: run_dir is absolute and non-portable`

Impact: `credibility`
Severity: `medium`
Classification: `documentation`
Confidence: `direct`
Recommended Direction: Emit `run_dir` as repo-relative path in capture tooling.

### Finding W-2: Spec docs carry stale v1.0.0-rc5 version headers

Three spec documents under `docs/spec/` carry `Version: v1.0.0-rc5` in their
headers. The workspace is at 1.2.2. Even if the spec content has not changed,
the stale version label creates an axis-mixing signal for reviewers.

Evidence:
- file: `docs/spec/dpw_gain.md` line 2
- file: `docs/spec/oscillator_api.md` line 2
- file: `docs/spec/header_layout_addendum.md` line 2

Impact: `onboarding`
Severity: `low`
Classification: `terminology`
Confidence: `direct`
Recommended Direction: Either update the version header to the current release
or add an explicit note distinguishing document revision from release version.
Adding a versioning terminology block (per NAM-005) would resolve both issues.

### Finding W-3: MATH_CONTRACT.md title says v1.2.1 while workspace is 1.2.2

The math contract title line reads "DPW4 Math Contract — v1.2.1" and its status
line reads "v1.2.1 (LOCKED)". The workspace version is 1.2.2. The contract is
intentionally frozen (no math changes in 1.2.2), but the visible version
discrepancy is a reviewer question.

Evidence:
- file: `docs/MATH_CONTRACT.md` line 1 ("v1.2.1")
- file: `docs/MATH_CONTRACT.md` line 279 ("v1.2.1 (LOCKED)")
- file: `Cargo.toml` line 24 (`version = "1.2.2"`)

Impact: `onboarding`
Severity: `low`
Classification: `terminology`
Confidence: `direct`
Recommended Direction: Add a one-line note clarifying the contract was locked at
1.2.1 and carried forward unchanged into 1.2.2, or update the title to
"v1.2.2 (content unchanged from v1.2.1)".

### Finding W-4: Performance docs are roadmap material frozen at v1.0.0-rc5

`docs/architecture/performance/CONTROL_SCHEDULER_BENCHMARKING.md` and
`HOT_PATH_EXECUTION_AND_BENCHMARKING.md` describe planned features explicitly
marked as not implemented. Their v1.0.0-rc5 status headers are accurate to
their era but may mislead readers about current scope.

Evidence:
- file: `docs/architecture/performance/CONTROL_SCHEDULER_BENCHMARKING.md` line 7-10
- file: `docs/architecture/performance/HOT_PATH_EXECUTION_AND_BENCHMARKING.md` line 7-14

Impact: `onboarding`
Severity: `low`
Classification: `documentation`
Confidence: `direct`
Recommended Direction: Add a directory-level note or move to `docs/roadmap/`.
No action required if these are considered internal reference material.

---

## 11. Remediation Priorities

| Priority | Finding | Effort | Impact |
|---:|---|---|---|
| 1 | W-1: run_dir portability in retained manifest | low (tooling change) | credibility |
| 2 | W-2: Stale v1.0.0-rc5 version headers + NAM-005 | low (3 file headers + terminology blocks) | onboarding |
| 3 | W-3: MATH_CONTRACT version label gap | trivial (one-line note) | onboarding |
| 4 | W-4: Performance docs classification | trivial (directory note or move) | onboarding |

---

## 12. Evidence and Unknowns

### Direct evidence (observed-this-session)

- command: `make gate` => `VERIFICATION PASSED` (7 determinism hashes)
- command: `make replay-tests` => all parser + tool suites PASS
- command: `make release-bundle-check VERSION=1.2.2` => `PASS` with `WARN` on run_dir
- command: `make test` => all workspace tests pass (49 tests, 3 ignored)
- command: `python3 scripts/check_doc_links.py` => `PASS`
- command: `cargo run -p replay-host -- diff artifacts/demo_v4/header_schema_B.rpl artifacts/demo_v4/header_schema_sample_payload_B.rpl` => `first divergence at frame 0`
- command: `cargo run --locked --release -p dpw4 --features cli --bin precision -- --version` => `precision 1.2.2`

### Retained evidence

- file: `docs/verification/releases/1.2.2/firmware_release_evidence.md` — hardware capture 5/5 PASS
- file: `docs/verification/releases/1.2.2/sha256_summary.txt` — 6 identical hashes
- file: `docs/verification/releases/1.2.2/replay_manifest_v1.txt` — manifest with `final_status=PASS`
- file: `docs/verification/CI_EVIDENCE.md` — CI run 23230032284 at v1.2.0-rc1

### Unknowns

- Kani formal verification was not re-executed this session. `verify_kani.sh` exists
  and is documented in VERIFICATION_GUIDE.md §3.1. Retained CI evidence at v1.2.0-rc1
  records it as passing.
- `verify_release_repro.sh` (dual-build identity check) was not re-executed.
  It is documented as supporting-only in `docs/verification/build_reproducibility.md`.
- `make ci-local` was not run as a single command (individual components were verified).
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

Accurately. The README is terse and correctly routes to VERIFICATION_GUIDE.md
and RELEASE_SURFACE.md. The release/experimental boundary is explicitly drawn
and consistently maintained across routing documents. No inflated claims were
found. Minor terminology drift exists in spec version headers (W-2, W-3) but
no claim-reality contradictions were identified.

### 3. What single sprint would most improve repository credibility and clarity?

**Normalize version headers across specs and retained evidence.**
In one pass: (1) make run_dir repo-relative in capture tooling output,
(2) add versioning terminology blocks to the three v1.0.0-rc5 spec docs,
(3) add a carry-forward note to MATH_CONTRACT.md. This clears all four
findings with minimal churn and no behavioral changes.
