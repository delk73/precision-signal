# Repository Health Audit — 1.4.0 baseline

**Audit date:** 2026-04-02
**Auditor:** automated agent (Claude Opus 4.6)
**Workspace version:** 1.4.0
**Toolchain:** rustc 1.91.1 (pinned via rust-toolchain.toml)
**Branch:** main
**HEAD:** 6076d5f24147
**Prior audit:** docs/audits/repository_health_baseline.md (workspace 1.2.2, HEAD e788031)

---

## 1. Audit Scope Summary

This audit establishes the repository health baseline for `precision-signal`
at workspace version 1.4.0 (HEAD `6076d5f24147`), against the prior baseline
at workspace 1.2.2 (HEAD `e788031`).

Releases between baselines: 1.3.0, 1.3.1, 1.4.0.

Changes since prior baseline:
- `replay-fw-f446` reclassified from Release to Experimental (1.3.0)
- interval CSV capture contract added as canonical STM32 self-stimulus boundary
- end-to-end evidence packaging path (`make demo-evidence-package`) established
- demo divergence entrypoint (`make demo-divergence`) added
- bounded sine correctness claim added (1.4.0)
- triangle freeze composition invariant added (1.4.0)
- explicit verification-scope publication (`VERIFICATION_SCOPE.md`) added (1.4.0)
- retained release evidence bundles for 1.3.0, 1.3.1, and 1.4.0 added
- all spec docs carry-forward updated to "Applies to: release 1.4.0"
- `Cargo.toml` comment uses bare "v1" phrasing (NAM-004, low severity, pre-existing)

Scope:
- Cargo workspace (10 crates, 4 default members)
- Two Rust CLI binaries: `precision`, `header_audit`
- Two released Python operator tools: `artifact_tool.py`, `artifact_diff.py`
- One experimental Rust binary: `replay-host`
- Firmware crate: `replay-fw-f446` (Experimental)
- Verification workflows: `make gate`, `make replay-tests`, `make demo-evidence-package`, `make release-bundle-check`
- Documentation set under `docs/`
- Retained release evidence under `docs/verification/releases/1.4.0/`

---

## 2. Metric Scores

| Metric | Score | Justification | Evidence |
|---|---:|---|---|
| Engineering integrity | 89 | Pinned toolchain, `#![forbid(unsafe_code)]`, determinism hashes locked to semver bumps, float quarantine enforced, dual-build bit-identical, release-bundle coherence check passes | file: `rust-toolchain.toml`; file: `VERIFICATION_GUIDE.md` §1.2; command-output: `make gate => VERIFICATION PASSED`; command-output: `verify_release_repro.sh => PASS` |
| Determinism / reproducibility | 92 | 7 golden hashes pass, dual-build bit-identical (SHA256 `c3a051aa…`), fixture-drift gate clean, evidence package reproduces byte-for-byte | command-output: `make gate => VERIFICATION PASSED`; command-output: `bash verify_release_repro.sh => PASS`; command-output: `make demo-evidence-package => retained bundle matches` |
| Specification quality | 86 | All 6 spec docs have versioning terminology blocks and carry-forward to 1.4.0; RPL0 contract byte-level; MATH_CONTRACT dense with carry-forward note; new VERIFICATION_SCOPE.md adds explicit limits | file: `docs/spec/rpl0_artifact_contract.md`; file: `docs/MATH_CONTRACT.md`; file: `docs/verification/releases/1.4.0/VERIFICATION_SCOPE.md` |
| Verification discipline | 90 | Multi-tier gate chain; release-bundle-check passes; Kani evidence retained; explicit verification-scope statement with limits and non-goals; packaged proof route reproduces | command-output: `make gate => PASS`; command-output: `make release-bundle-check VERSION=1.4.0 => PASS`; file: `docs/verification/releases/1.4.0/kani_evidence.txt` |
| Codebase maintainability | 78 | Workspace well-factored (core crates zero-dep); some crates are stubs; 10 members with 4 default; unchanged from prior baseline | file: `Cargo.toml` workspace members |
| Architecture clarity | 82 | Clear release/experimental boundary; firmware reclassified to Experimental aligns classification with evidence; workspace.md routes well | file: `docs/RELEASE_SURFACE.md`; file: `docs/architecture/workspace.md` |
| Documentation depth | 84 | Normative specs thorough; verification-scope publication adds explicit limits; demo evidence packaging documented end-to-end; complete doc index | file: `docs/README.md`; file: `docs/demos/demo_evidence_packaging.md`; file: `docs/verification/releases/1.4.0/VERIFICATION_SCOPE.md` |
| Documentation organization | 83 | Top-level routing clear; all public docs reachable from index; link integrity passes; demo evidence packaging cleanly integrated | file: `docs/README.md`; command-output: `make doc-link-check => PASS` |
| Repository presentation | 84 | README terse, correct, and evidence-backed; manual release checklist present; no inflated claims; packaged proof route referenced | file: `README.md` |
| Developer onboarding | 87 | "First 5 Minutes" block works; `make gate => VERIFICATION PASSED` on first try; verification-scope document provides explicit limits for new readers | file: `README.md` §First 5 Minutes; file: `docs/verification/releases/1.4.0/VERIFICATION_SCOPE.md` |
| Conceptual coherence | 86 | Consistent "deterministic execution analysis infrastructure"; normative/advisory maintained; new correctness claims explicitly bounded | file: `VERIFICATION_GUIDE.md` §1.1; file: `docs/verification/releases/1.4.0/VERIFICATION_SCOPE.md` |
| Research / innovation value | 78 | Phase-domain fixed-point oscillator with formal proofs; hardware-backed determinism; bounded correctness claim for sine path is a novel evidence contribution; narrow domain | file: `docs/MATH_CONTRACT.md`; file: `docs/verification/releases/1.4.0/VERIFICATION_SCOPE.md` |
| OSS trustworthiness signals | 84 | MIT license, pinned toolchain, hash-locked releases, retained evidence bundles, doc-link CI, release-bundle coherence check, reproducibility verification | file: `LICENSE`; command-output: `make release-bundle-check VERSION=1.4.0 => PASS`; command-output: `bash verify_release_repro.sh => PASS` |

---

## 3. Implementation Reality Map

| Component | Implemented | Classification | User Path | Verification Path | Evidence |
|---|---|---|---|---|---|
| `precision validate` | yes | Release | `make gate` | `make gate` → 7 golden hashes | file: `crates/dpw4/src/bin/precision.rs` enum Commands::Validate |
| `precision generate` | yes | Release | `precision generate --shape saw …` | CLI_SURFACE_EVIDENCE.md | file: `crates/dpw4/src/bin/precision.rs` enum Commands::Generate |
| `precision artifacts` | yes | Release | `precision artifacts --out <dir>` | CLI_SURFACE_EVIDENCE.md | file: `crates/dpw4/src/bin/precision.rs` enum Commands::Artifacts |
| `precision inspect` | yes | Release | `precision inspect --file <path>` | CLI_SURFACE_EVIDENCE.md | file: `crates/dpw4/src/bin/precision.rs` enum Commands::Inspect |
| `precision verify` | yes | Release | `precision verify --file <path>` | CLI_SURFACE_EVIDENCE.md | file: `crates/dpw4/src/bin/precision.rs` enum Commands::Verify |
| `header_audit` | yes | Release | `header_audit <file>` | CLI_SURFACE_EVIDENCE.md | file: `crates/dpw4/src/bin/header_audit.rs` |
| `artifact_tool.py` | yes | Release | `python3 scripts/artifact_tool.py verify …` | `make replay-tool-tests` | file: `scripts/artifact_tool.py` |
| `artifact_diff.py` | yes | Release | `python3 scripts/artifact_diff.py …` | `make replay-tool-tests` | file: `scripts/artifact_diff.py` |
| `replay-host diff` | yes | Experimental | `cargo run -p replay-host -- diff …` | not in release gate | file: `crates/replay-host/src/main.rs` |
| `replay-fw-f446` | yes | Experimental | `make flash-ur` (hardware-dependent) | retained evidence: `docs/verification/releases/1.2.2/` | file: `crates/replay-fw-f446/`; doc: `docs/RELEASE_SURFACE.md` |
| `dpw4` core library | yes | Release | library API | `make test`; `make gate` | file: `crates/dpw4/src/lib.rs` |
| `geom-signal` library | yes | Release | library API | `make test`; Kani proofs | file: `crates/geom-signal/src/lib.rs` |
| `geom-spatial` library | yes | Release | library API | `make test` | file: `crates/geom-spatial/src/lib.rs` |
| `replay-core` | yes | Experimental | library API | `make test`; Kani wire layout proofs | file: `crates/replay-core/src/lib.rs` |
| `replay-embed` | yes | Experimental | none | `make test` | file: `crates/replay-embed/` |
| `replay-cli` | yes | Experimental | none | `make test` | file: `crates/replay-cli/` |
| `audit-float-boundary` | yes | Reference | build tool | `make check-workspace` | file: `crates/audit-float-boundary/` |

---

## 4. CLI Surface Inventory

| Command | Entrypoint | Type | Runnable | Verification Path | Classification |
|---|---|---|---|---|---|
| `precision validate --mode quick` | `crates/dpw4/src/bin/precision.rs` | rust-bin | observed | `make gate` | Release |
| `precision validate --mode full` | `crates/dpw4/src/bin/precision.rs` | rust-bin | observed | `make gate-full` | Release |
| `precision generate` | `crates/dpw4/src/bin/precision.rs` | rust-bin | observed | `docs/verification/CLI_SURFACE_EVIDENCE.md` | Release |
| `precision artifacts` | `crates/dpw4/src/bin/precision.rs` | rust-bin | observed | `docs/verification/CLI_SURFACE_EVIDENCE.md` | Release |
| `precision inspect` | `crates/dpw4/src/bin/precision.rs` | rust-bin | observed | `docs/verification/CLI_SURFACE_EVIDENCE.md` | Release |
| `precision verify` | `crates/dpw4/src/bin/precision.rs` | rust-bin | observed | `docs/verification/CLI_SURFACE_EVIDENCE.md` | Release |
| `header_audit` | `crates/dpw4/src/bin/header_audit.rs` | rust-bin | observed | `docs/verification/CLI_SURFACE_EVIDENCE.md` | Release |
| `artifact_tool.py verify` | `scripts/artifact_tool.py` | python-tool | observed | `make replay-tool-tests` | Release |
| `artifact_tool.py hash` | `scripts/artifact_tool.py` | python-tool | observed | `make replay-tool-tests` | Release |
| `artifact_tool.py compare` | `scripts/artifact_tool.py` | python-tool | observed | `make replay-tool-tests` | Release |
| `artifact_tool.py inspect` | `scripts/artifact_tool.py` | python-tool | observed | `make replay-tool-tests` | Release |
| `artifact_tool.py capture` | `scripts/artifact_tool.py` | python-tool | observed (hardware) | `make replay-check` | Release |
| `artifact_tool.py lock-baseline` | `scripts/artifact_tool.py` | python-tool | observed (hardware) | `make replay-check` | Release |
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
| End-to-end evidence packaging | yes | Release | `make demo-evidence-package` | `make demo-evidence-package` => retained bundle matches | command-output: observed-this-session | direct |
| Dual-build reproducibility | yes | Release | `bash verify_release_repro.sh` | `PASS — builds are bit-for-bit identical` (SHA256 `c3a051aa…`) | command-output: observed-this-session | direct |
| Formal verification (Kani Tier-1) | yes | Release | `bash verify_kani.sh` | retained evidence: `docs/verification/releases/1.4.0/kani_evidence.txt` | file: retained-evidence | direct |
| Bounded sine correctness | yes | Release | unit test in `dpw4` | retained evidence: `cargo_test_sine_bounded_correctness.txt` | file: `docs/verification/releases/1.4.0/cargo_test_sine_bounded_correctness.txt` | direct |
| Triangle freeze composition | yes | Release | Kani proof + unit test | retained evidence: `kani_evidence.txt` + `cargo_test_triangle_control_surface.txt` | file: `docs/verification/releases/1.4.0/` | direct |
| Hardware capture (F446) | yes | Experimental | `make flash-ur` | retained evidence: `docs/verification/releases/1.2.2/` | file: retained-evidence | direct |
| Rust replay (v0 frames) | yes | Experimental | `cargo run -p replay-host -- diff` | cross-context comparison only | file: `docs/verification/CROSS_CONTEXT_INVARIANCE.md` | direct |
| Cross-context invariance | yes | Reference | `docs/verification/CROSS_CONTEXT_INVARIANCE.md` | retained evidence: `docs/verification/cross_context/` | file: retained-evidence | direct |

---

## 6. Claim-Reality Gap Register

| # | Claim | Source | Status | Impact | Confidence | Notes |
|---|---|---|---|---|---|---|
| 1 | `make gate` => VERIFICATION PASSED | README.md §First 5 Minutes | exact | — | direct | command-output: `make gate => VERIFICATION PASSED` (observed-this-session) |
| 2 | Released replay-facing operator tooling is the Python toolchain | docs/replay/tooling.md | exact | — | direct | `artifact_tool.py` and `artifact_diff.py` exercised via `make replay-tool-tests` |
| 3 | Rust replay is experimental and not part of release surface | docs/RELEASE_SURFACE.md, docs/replay/tooling.md | exact | — | direct | `replay-host` description: "not classified as released"; not in `make gate` |
| 4 | `replay-fw-f446` is experimental | docs/RELEASE_SURFACE.md | exact | — | direct | Firmware crate README: "active experimental STM32 self-stimulus capture path"; reclassified from Release in 1.3.0 |
| 5 | Packaged proof route reproduces retained bundle byte-for-byte | README.md, docs/demos/demo_evidence_packaging.md | exact | — | direct | command-output: `make demo-evidence-package => retained bundle matches` (observed-this-session) |
| 6 | Bounded sine correctness over 4097-point domain | docs/verification/releases/1.4.0/VERIFICATION_SCOPE.md | exact | — | direct | Retained evidence: `cargo_test_sine_bounded_correctness.txt`; claim explicitly states empirical, not global |
| 7 | Triangle freeze preserves emitted sample | VERIFICATION_SCOPE.md, CHANGELOG.md 1.4.0 | exact | — | direct | Kani proof `proof_triangle_freeze_egress_invariant` + runtime test retained |
| 8 | MATH_CONTRACT locked at v1.2.1, carried forward to 1.4.0 | docs/MATH_CONTRACT.md | exact | — | direct | Carry-forward note, versioning terminology block, "Applies to: release 1.4.0 (content unchanged)" |
| 9 | All spec docs applicable to 1.4.0 | docs/spec/*.md | exact | — | direct | All 6 spec docs carry "Applies to: release 1.4.0 (content unchanged)" |
| 10 | Dual-build reproducibility | docs/verification/build_reproducibility.md | exact | — | direct | command-output: `bash verify_release_repro.sh => PASS` — SHA256 `c3a051aa…` (observed-this-session) |
| 11 | Documentation link integrity | docs/README.md | exact | — | direct | command-output: `make doc-link-check => PASS` (observed-this-session) |
| 12 | "Verification-depth release only; no new capability surface" | CHANGELOG.md 1.4.0 Notes | exact | — | direct | No new CLI commands, no artifact format change; only correctness claims and proof evidence added |
| 13 | workspace version = 1.4.0 | Cargo.toml, VERIFICATION_GUIDE.md | exact | — | direct | `workspace.package.version = "1.4.0"`; VERIFICATION_GUIDE.md Version header: "1.4.0" |
| 14 | "The public v1 release boundary is the deterministic replay infrastructure" | Cargo.toml line 2 | partial | credibility | direct | NAM-004: bare "v1" is ambiguous (format version vs release version); low severity, pre-existing from prior baselines |

---

## 7. Documentation Topology Map

| Document | Purpose | Category | Public | Overlap | Overlap Severity | Authority |
|---|---|---|---|---|---|---|
| README.md | Entry routing | entry | public | none | — | canonical |
| docs/README.md | Documentation index | index | public | none | — | canonical |
| docs/RELEASE_SURFACE.md | Release classification and routing | release-classification | public | none | — | canonical |
| VERIFICATION_GUIDE.md | Release contract and conformance governance | normative | public | none | — | canonical |
| docs/MATH_CONTRACT.md | Arithmetic and signal-path contract | normative | public | none | — | canonical |
| docs/spec/rpl0_artifact_contract.md | Binary artifact format spec | normative | public | none | — | canonical |
| docs/spec/dpw_gain.md | Gain model invariants | normative | public | none | — | supporting |
| docs/spec/oscillator_api.md | Oscillator dispatch contract | normative | public | none | — | supporting |
| docs/spec/reference_invariants.md | Mathematical reference invariants | normative | public | none | — | supporting |
| docs/spec/pulse_implementation_spec.md | Pulse/square spec | normative | public | none | — | supporting |
| docs/spec/header_layout_addendum.md | Header layout spec | normative | public | none | — | supporting |
| docs/replay/tooling.md | Replay-tooling boundary | release-classification | public | minor with RELEASE_SURFACE.md | minor | supporting |
| docs/replay/README.md | Replay subsystem index | index | public | none | — | supporting |
| docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md | STM32 self-stimulus capture contract | normative | public | none | — | canonical |
| docs/replay/DIVERGENCE_SEMANTICS.md | Divergence explanation contract | normative | public | none | — | canonical |
| docs/demos/demo_evidence_packaging.md | Canonical packaged proof route | workflow | public | none | — | canonical |
| docs/verification/releases/1.4.0/ | Retained release evidence bundle | normative | public | none | — | canonical |
| docs/verification/releases/1.4.0/VERIFICATION_SCOPE.md | Proof coverage, limits, and non-goals | normative | public | none | — | canonical |
| docs/verification/CLI_SURFACE_EVIDENCE.md | CLI promotion evidence | workflow | public | none | — | supporting |
| docs/verification/build_reproducibility.md | Build identity checks | workflow | public | none | — | supporting |
| docs/verification/CROSS_CONTEXT_INVARIANCE.md | Cross-context replay invariance | workflow | public | none | — | supporting |
| docs/cli/precision.md | CLI reference | reference | public | none | — | supporting |
| docs/cli/examples.md | CLI usage examples | reference | public | none | — | supporting |
| docs/architecture/workspace.md | Workspace framing | deep-architecture | public | none | — | supporting |
| docs/system_architecture_disclosure.md | Architecture overview | descriptive | public | minor (architecture overview) | minor | supporting |
| docs/governance/DESIGN_AXIOMS.md | Governance axioms | normative | public | none | — | supporting |
| docs/governance/DEBT.md | Tracked design debt | reference | public | none | — | supporting |
| docs/verification/CI_EVIDENCE.md | Historical CI evidence | historical-audit | public | none | — | historical |
| docs/verification/FIRMWARE_CAPTURE_EVIDENCE.md | Historical firmware capture evidence | historical-audit | public | none | — | historical |
| docs/verification/D-03_TriangleDPW4_Audit.md | Retained TriangleDPW4 audit | historical-audit | public | none | — | historical |
| docs/audits/repository_health_baseline.md | Prior repository health baseline (1.2.2) | historical-audit | public | none | — | historical |

---

## 8. Terminology Consistency Register

| Rule ID | File | Line | Classification | Severity | Evidence | Notes |
|---|---|---:|---|---|---|---|
| NAM-004 | Cargo.toml | 2 | ambiguous-version | warn | "The public v1 release boundary" — bare "v1" could mean format version or release version | Pre-existing from prior baselines; low impact since Cargo.toml comment is not user-facing documentation |

All prior NAM violations from the 1.2.2 baseline remain resolved.
No NAM-001 (artifact vX), NAM-002 (replay vX), NAM-003 (_vX.md artifact spec),
NAM-005 (missing terminology block), or NAM-006 (capability statement) violations
found in public-facing documents.

Crate descriptions using "v1 release surface" (e.g., `replay-core`, `replay-host`,
`replay-embed`, `replay-cli` Cargo.toml) use the phrase to describe exclusion from
the release surface rather than as a capability claim. These are low-severity and
do not distort capability perception.

---

## 9. Prior Finding Disposition

| Finding | Prior Status (1.2.2 baseline) | Current Status (1.4.0) | Evidence |
|---|---|---|---|
| W-1: Retained manifest absolute run_dir | unchanged | **unchanged** | `make release-bundle-check VERSION=1.4.0 => PASS`; repo-relative paths used |
| W-2: Stale version headers + NAM-005 | unchanged | **unchanged** | All spec docs updated to "Applies to: release 1.4.0 (content unchanged)" |
| W-3: MATH_CONTRACT v1.2.1 label gap | unchanged | **unchanged** | Carry-forward note explicit: "locked at release 1.2.1 and is carried forward unchanged for release 1.4.0" |
| W-4: Performance docs classification | unchanged | **unchanged** | Reclassified as historical reference; stable |
| W-5: Supplementary `--mode full` evidence | unchanged | **unchanged** | `make gate-full` available; not re-executed this session |
| W-6: Kani verification freshness | unchanged | **improved** | Fresh retained evidence: `docs/verification/releases/1.4.0/kani_evidence.txt` |
| W-7: Orphaned documentation files | resolved | **unchanged** | All public docs reachable from index; `make doc-link-check => PASS` |
| W-8: Cross-context terminology | resolved | **unchanged** | Terminology notes remain in place |

New observations (1.4.0):
- N-1: `Cargo.toml` line 2 bare "v1" phrasing (NAM-004, warn) — pre-existing, low severity
- N-2: `replay-fw-f446` correctly reclassified from Release to Experimental in 1.3.0 — alignment improvement

---

## 10. Strengths

1. **Determinism discipline remains exemplary.** 7 golden hashes pass, dual-build
   bit-identical (SHA256 `c3a051aa71cbb24140c475102ccda430ef8f02da783a8e37eb52f8dbc67b9204`),
   fixture-drift gate clean, evidence package reproduces retained bundle byte-for-byte.
   Evidence: `make gate => VERIFICATION PASSED`; `bash verify_release_repro.sh => PASS`;
   `make demo-evidence-package => retained bundle matches` (all observed-this-session).

2. **Release/experimental boundary is well-drawn and improved.** `replay-fw-f446`
   reclassified to Experimental in 1.3.0 aligns classification with evidence.
   `docs/RELEASE_SURFACE.md` explicitly classifies every surface.
   Evidence: file: `docs/RELEASE_SURFACE.md`; file: `crates/replay-fw-f446/README.md`.

3. **Verification gate chain is comprehensive and layered.** `make gate`,
   `make replay-tests`, `make demo-evidence-package`, `make release-bundle-check`,
   `make fixture-drift-check`, `make doc-link-check` all pass.
   Evidence: all observed-this-session.

4. **Explicit verification-scope publication is a strong transparency signal.**
   `VERIFICATION_SCOPE.md` states exactly what is proven, what is empirical, what is
   excluded, and what the release does not claim. This is best-practice for correctness
   communication.
   Evidence: file: `docs/verification/releases/1.4.0/VERIFICATION_SCOPE.md`.

5. **Retained release evidence chain is well-structured.** Release bundles for
   1.2.0, 1.2.1, 1.2.2, 1.3.0, 1.3.1, and 1.4.0 are retained immutably.
   `make release-bundle-check VERSION=1.4.0` passes cleanly.
   Evidence: command-output: observed-this-session.

6. **Documentation routing is clear.** README → docs/README.md →
   RELEASE_SURFACE.md and VERIFICATION_GUIDE.md. Normative vs descriptive
   distinction maintained. All links resolve.
   Evidence: command-output: `make doc-link-check => PASS`.

7. **Packaged proof route adds a strong reproducibility layer.** The
   `make demo-evidence-package` path packages Phase 1 through Phase 5 replay
   evidence into one verifiable bundle with byte-for-byte retained comparison.
   Evidence: command-output: observed-this-session.

8. **Correctness claims are appropriately bounded.** The sine correctness claim is
   explicitly empirical over a stated finite domain. The triangle freeze claim is
   explicitly composition-level for the freeze branch only. Neither overstates scope.
   Evidence: file: `docs/verification/releases/1.4.0/VERIFICATION_SCOPE.md`.

---

## 11. Weaknesses

No high-severity findings identified!

### Residual observations (low severity)

W-L1: `Cargo.toml` line 2 comment uses bare "v1" ("The public v1 release boundary").
- Impact: credibility (marginal)
- Severity: low
- Classification: terminology
- Confidence: direct
- Direction: Disambiguate to "format v1" or "release v1" in the comment.

W-L2: `replay-core`, `replay-embed`, `replay-cli` remain minimal stubs.
- Impact: maintainability (marginal)
- Severity: low
- Classification: implementation
- Confidence: direct
- Direction: Consistent with "experimental" classification; no action needed unless
  these crates are promoted.

W-L3: `make ci-local` not run as a single compound command this session.
- Impact: unknown (possibly none)
- Severity: low
- Classification: implementation
- Confidence: inference
- Direction: All components were verified independently; compound execution is a
  completeness check only.

---

## 12. Remediation Priorities

No high- or medium-priority remediation required.

Low-priority (optional):
1. Disambiguate "v1" in `Cargo.toml` line 2 comment (NAM-004, low churn).
2. Consider adding a compound `make ci-local` execution to the retained release evidence.

---

## 13. Evidence and Unknowns

### Direct evidence (observed-this-session)

- command: `make check-workspace` => `Finished dev profile … in 0.05s`
- command: `make gate` => `VERIFICATION PASSED` (7 determinism hashes matched)
- command: `make test` => all workspace tests pass (133 passed, 4 ignored, 0 failed)
- command: `make replay-tests` => all parser + tool + demo suites PASS
- command: `make fixture-drift-check` => clean (no drift)
- command: `make demo-evidence-package` => `retained bundle matches`; `replay diff identical`; `replay diff perturbed: first divergence at frame 17`
- command: `make release-bundle-check VERSION=1.4.0` => `PASS: retained release bundle coherence`
- command: `make doc-link-check` => `PASS: documentation link integrity`
- command: `bash verify_release_repro.sh` => `PASS — builds are bit-for-bit identical` (SHA256: `c3a051aa71cbb24140c475102ccda430ef8f02da783a8e37eb52f8dbc67b9204`)
- command: `python3 scripts/check_no_replay_backedges.py` => `{"status":"ok","violations":0}`
- command: `bash scripts/check_ci_pins.sh` => `CI pin check OK`

### Retained evidence

- file: `docs/verification/releases/1.4.0/kani_evidence.txt` — Tier-1 Kani evidence for 1.4.0
- file: `docs/verification/releases/1.4.0/make_gate.txt` — retained gate output
- file: `docs/verification/releases/1.4.0/make_demo_evidence_package.txt` — retained evidence packaging output
- file: `docs/verification/releases/1.4.0/cargo_test_sine_bounded_correctness.txt` — bounded sine correctness
- file: `docs/verification/releases/1.4.0/cargo_test_triangle_control_surface.txt` — triangle control surface test
- file: `docs/verification/releases/1.4.0/release_reproducibility.txt` — dual-build identity
- file: `docs/verification/releases/1.4.0/verify_release_repro.txt` — reproducibility script output
- file: `docs/verification/releases/1.4.0/VERIFICATION_SCOPE.md` — proof coverage, limits, non-goals
- file: `docs/verification/releases/1.2.2/` — hardware-backed retained evidence (immutable)
- file: `docs/verification/releases/1.3.1/` — patch-release retained bundle (immutable)

### Unknowns

- `bash verify_kani.sh` was not re-executed this session; retained evidence from the 1.4.0 release bundle accepted.
- `make ci-local` was not run as a single compound command (individual components verified independently).
- No GitHub Actions workflow file was inspected (branch-local audit only).
- Heavy Tier-2 Kani proofs were not executed; disposition documented in VERIFICATION_SCOPE.md.

---

## Final Question

### 1. What executable capabilities are actually implemented today?

- `precision validate` (canonical determinism gate, 7 golden hashes)
- `precision generate` / `artifacts` / `inspect` / `verify` (operator CLI surface)
- `header_audit` (Fletcher-32 header audit tool)
- `artifact_tool.py` (released Python operator CLI: verify, hash, capture, inspect, compare, lock-baseline)
- `artifact_diff.py` (released Python divergence-localization tool)
- `replay-host diff` (experimental Rust v0-frame replay)
- `replay-fw-f446` (experimental firmware capture, hardware-dependent)
- End-to-end evidence packaging (`make demo-evidence-package`)
- Quantization divergence witness (`make demo-divergence`)
- Parser and tool regression suites (13+ test scripts)
- Formal verification harness runner (`verify_kani.sh`) with explicit Tier-1/Tier-2 split
- Dual-build reproducibility check (`verify_release_repro.sh`)
- Release bundle coherence checker (`check_release_bundle.py`)
- Documentation link integrity checker (`check_doc_links.py`)
- Fixture drift checker (`make fixture-drift-check`)

### 2. How accurately does the repository communicate those capabilities?

Accurately. All release claims match implementation and verification evidence.
The release/experimental boundary is explicitly drawn and consistently maintained.
No claim-reality contradictions identified. The firmware path reclassification to
Experimental (1.3.0) improved alignment. The 1.4.0 verification-scope publication
is a strong transparency signal that explicitly bounds what is proven vs empirical.

One marginal terminology finding (bare "v1" in Cargo.toml comment) is low-severity
and does not distort capability perception for operators or reviewers.

### 3. What single sprint would most improve repository credibility and clarity?

No high-priority remediation remains. The most productive next focus would be:
- Advancing the experimental Rust replay surface (`replay-host`) toward release
  classification with schema-aware replay semantics
- Or expanding the bounded correctness evidence to additional waveform paths
  beyond sine

Both are forward-looking improvements, not current deficiencies.
