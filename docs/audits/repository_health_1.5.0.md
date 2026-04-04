# Repository Health Audit — 1.5.0 baseline

> Historical audit snapshot only. This document is reference material, not
> release authority. It may describe gaps later closed by subsequent
> documentation cleanups. Use `VERIFICATION_GUIDE.md`,
> `docs/RELEASE_SURFACE.md`, and `docs/verification/releases/` for current
> release truth.

**Audit date:** 2026-04-04
**Auditor:** automated agent (GitHub Copilot / Claude Sonnet 4.6)
**Workspace version:** 1.5.0
**Toolchain:** rustc 1.91.1 (pinned via rust-toolchain.toml)
**Branch:** main
**HEAD:** cfd9e932e21d
**Prior audit:** docs/audits/repository_health_1.4.0.md (workspace 1.4.0, dated 2026-04-02)

---

## 1. Audit Scope Summary

This audit establishes the repository health baseline for `precision-signal`
at workspace version 1.5.0 (HEAD `cfd9e932e21d`), against the prior baseline
at workspace 1.4.0.

Releases between baselines: 1.5.0 (one minor release; no patch releases between
1.4.0 and 1.5.0).

Changes since prior baseline:

- Minor release: `replay-host diff` promoted from Experimental to Release,
  bounded to the retained `artifacts/rpl0/` proof corpus only
- New retained release bundle: `docs/verification/releases/1.5.0/` with 15
  files including 3 command transcripts, Kani evidence, gate and tool outputs,
  reproducibility record, and scope document
- `docs/RELEASE_SURFACE.md`: version references updated; `replay-host diff`
  added to Release block; experimental Rust replay description refined
- `docs/replay/tooling.md`: "Released Rust Replay" section added for `diff`
  command; experimental remainder section added below
- `README.md`: 1.5.0 release paragraph added for `replay-host diff` promotion
- `VERIFICATION_GUIDE.md`: version header updated 1.4.0 → 1.5.0; reviewer
  routing updated; proof boundary reference updated; section 3.6 updated
- `Cargo.toml` and `Cargo.lock`: workspace version bumped 1.4.0 → 1.5.0
  across all 10 crates
- `docs/verification/releases/index.md`: 1.5.0 active, 1.4.0 moved to
  historical
- `.github/agents/repository-auditor.agent.md`: "Release Readiness Audit Mode"
  section added
- No code behavior changes; no artifact format changes; no golden hash changes

Scope:

- 10-crate Cargo workspace (4 default members)
- Two Rust CLI binaries: `precision`, `header_audit`
- One promoted Rust binary (bounded Release): `replay-host diff`
- Two released Python operator tools: `artifact_tool.py`, `artifact_diff.py`
- Documentation set under `docs/`
- Retained release evidence under `docs/verification/releases/1.5.0/`

---

## 2. Metric Scores

| Metric | Score | Justification | Evidence |
|---|---:|---|---|
| Engineering integrity | 89 | Pinned toolchain, float quarantine, dual-build bit-identical (SHA256 `7da5387c…`); `replay-host diff` promotion scope-bounded and evidenced; no hash regeneration required for minor release | file: `rust-toolchain.toml`; file: `VERIFICATION_GUIDE.md` §1.2; retained: `docs/verification/releases/1.5.0/release_reproducibility.txt` |
| Determinism / reproducibility | 92 | 7 golden hashes pass; dual-build SHA256 `7da5387c710c98b62ae876722350efdf4d9d1919211db9841c13933b9e3df714`; retained bundle reproduces byte-for-byte | retained: `docs/verification/releases/1.5.0/make_gate.txt` (VERIFICATION PASSED); retained: `docs/verification/releases/1.5.0/release_reproducibility.txt` |
| Specification quality | 85 | Spec docs carry-forward not updated to 1.5.0; VERIFICATION_GUIDE.md §3.6 routes to `docs/verification/releases/1.5.0/` for sine correctness that lives in `docs/verification/releases/1.4.0/VERIFICATION_SCOPE.md` | file: `docs/MATH_CONTRACT.md`; file: `docs/spec/rpl0_artifact_contract.md`; `VERIFICATION_GUIDE.md` line 178 |
| Verification discipline | 91 | Kani evidence fresh at 1.5.0 (24 Tier-1 harnesses all pass; 3 replay-core proofs included); `replay-host diff` backed by 4 operator-path tests; bundle-check passes | retained: `docs/verification/releases/1.5.0/kani_evidence.txt`; retained: `docs/verification/releases/1.5.0/cargo_test_replay_host_operator_path.txt` |
| Codebase maintainability | 78 | Same workspace structure; no crate changes; stubs unchanged; consistent with Experimental classification | file: `Cargo.toml` workspace members |
| Architecture clarity | 83 | `replay-host diff` promotion precisely bounded; experimental remainder separated in `docs/replay/tooling.md`; release/experimental boundary tightened | file: `docs/RELEASE_SURFACE.md`; file: `docs/replay/tooling.md` |
| Documentation depth | 84 | `RUST_REPLAY_DIFF_SCOPE.md` adds precise scope statement for promoted command; sine correctness routing gap is low-severity | file: `docs/verification/releases/1.5.0/RUST_REPLAY_DIFF_SCOPE.md` |
| Documentation organization | 83 | Link integrity passes; routing clean; topology consistent with prior baseline | retained: `docs/verification/releases/1.5.0/make_doc_link_check.txt` |
| Repository presentation | 85 | README updated with bounded 1.5.0 promotion paragraph; no inflated claims; manual release checklist retained | file: `README.md` |
| Developer onboarding | 87 | "First 5 Minutes" block unchanged; `make gate` continues to return VERIFICATION PASSED | retained: `docs/verification/releases/1.5.0/make_gate.txt` |
| Conceptual coherence | 86 | "deterministic execution analysis infrastructure" consistent; normative/advisory maintained; bounded release claim does not distort concept | file: `VERIFICATION_GUIDE.md` §1.1 |
| Research / innovation value | 79 | Bounded Rust replay evidence demonstrates a narrow but precise release pattern useful as a model for future surface promotion | file: `docs/verification/releases/1.5.0/RUST_REPLAY_DIFF_SCOPE.md` |
| OSS trustworthiness signals | 85 | Release bundle chain extended through 1.5.0; scope document backs every new release claim precisely; release-bundle-check passes | retained: `docs/verification/releases/1.5.0/make_release_bundle_check.txt`; retained: `docs/verification/releases/1.5.0/RUST_REPLAY_DIFF_SCOPE.md` |

---

## 3. Implementation Reality Map

| Component | Implemented | Classification | User Path | Verification Path | Evidence | Freshness |
|---|---|---|---|---|---|---|
| `precision validate` | yes | Release | `make gate` | `make gate` → 7 golden hashes | file: `crates/dpw4/src/bin/precision.rs` enum Commands::Validate | retained-evidence |
| `precision generate` | yes | Release | `precision generate --shape saw …` | `docs/verification/CLI_SURFACE_EVIDENCE.md` | file: `crates/dpw4/src/bin/precision.rs` enum Commands::Generate | retained-evidence |
| `precision artifacts` | yes | Release | `precision artifacts --out <dir>` | `docs/verification/CLI_SURFACE_EVIDENCE.md` | file: `crates/dpw4/src/bin/precision.rs` enum Commands::Artifacts | retained-evidence |
| `precision inspect` | yes | Release | `precision inspect --file <path>` | `docs/verification/CLI_SURFACE_EVIDENCE.md` | file: `crates/dpw4/src/bin/precision.rs` enum Commands::Inspect | retained-evidence |
| `precision verify` | yes | Release | `precision verify --file <path>` | `docs/verification/CLI_SURFACE_EVIDENCE.md` | file: `crates/dpw4/src/bin/precision.rs` enum Commands::Verify | retained-evidence |
| `header_audit` | yes | Release | `header_audit <file>` | `docs/verification/CLI_SURFACE_EVIDENCE.md` | file: `crates/dpw4/src/bin/header_audit.rs` | retained-evidence |
| `artifact_tool.py` | yes | Release | `python3 scripts/artifact_tool.py verify …` | `make replay-tests` → PASS | file: `scripts/artifact_tool.py` | retained-evidence |
| `artifact_diff.py` | yes | Release | `python3 scripts/artifact_diff.py …` | `make replay-tests` → PASS | file: `scripts/artifact_diff.py` | retained-evidence |
| `replay-host diff` | yes | Release (bounded) | `cargo run -q -p replay-host -- diff <a.rpl> <b.rpl>` | 4 operator-path tests + retained transcripts | file: `crates/replay-host/src/main.rs`; retained: `docs/verification/releases/1.5.0/RUST_REPLAY_DIFF_SCOPE.md` | retained-evidence |
| `replay-host import-interval-csv` | yes (in binary) | Experimental | exposed in CLI usage text | none | retained: `docs/verification/releases/1.5.0/replay_host_diff_missing_arg.txt` (usage line) | retained-evidence |
| `replay-host validate-interval-csv` | yes (in binary) | Experimental | exposed in CLI usage text | none | retained: `docs/verification/releases/1.5.0/replay_host_diff_missing_arg.txt` (usage line) | retained-evidence |
| `replay-fw-f446` | yes | Experimental | `make flash-ur` (hardware-dependent) | retained evidence: `docs/verification/releases/1.2.2/` | file: `crates/replay-fw-f446/` | retained-evidence |
| `dpw4` core library | yes | Release | library API | `make gate`; Kani Tier-1 proofs | file: `crates/dpw4/src/lib.rs` | retained-evidence |
| `geom-signal` library | yes | Release | library API | Kani proofs (`proof_sqrt_no_panic`, `proof_sin_cos_no_panic`) | file: `crates/geom-signal/src/lib.rs` | retained-evidence |
| `geom-spatial` library | yes | Release | library API | workspace tests | file: `crates/geom-spatial/src/lib.rs` | retained-evidence |
| `replay-core` | yes | Experimental | library API | Kani wire layout proofs (3 harnesses) | file: `crates/replay-core/src/lib.rs` | retained-evidence |
| `replay-embed` | yes | Experimental | none | workspace tests | file: `crates/replay-embed/` | retained-evidence |
| `replay-cli` | yes | Experimental | none | workspace tests | file: `crates/replay-cli/` | retained-evidence |
| `audit-float-boundary` | yes | Reference | build tool | `make check-workspace` | file: `crates/audit-float-boundary/` | retained-evidence |

---

## 4. CLI Surface Inventory

| Command | Entrypoint | Type | Runnable | Classification | Verification Path | Freshness |
|---|---|---|---|---|---|---|
| `precision validate --mode quick` | `crates/dpw4/src/bin/precision.rs` | rust-bin | observed | Release | `make gate` | retained-evidence |
| `precision validate --mode full` | `crates/dpw4/src/bin/precision.rs` | rust-bin | observed | Release | `make gate-full` | retained-evidence |
| `precision generate` | `crates/dpw4/src/bin/precision.rs` | rust-bin | observed | Release | `docs/verification/CLI_SURFACE_EVIDENCE.md` | retained-evidence |
| `precision artifacts` | `crates/dpw4/src/bin/precision.rs` | rust-bin | observed | Release | `docs/verification/CLI_SURFACE_EVIDENCE.md` | retained-evidence |
| `precision inspect` | `crates/dpw4/src/bin/precision.rs` | rust-bin | observed | Release | `docs/verification/CLI_SURFACE_EVIDENCE.md` | retained-evidence |
| `precision verify` | `crates/dpw4/src/bin/precision.rs` | rust-bin | observed | Release | `docs/verification/CLI_SURFACE_EVIDENCE.md` | retained-evidence |
| `header_audit` | `crates/dpw4/src/bin/header_audit.rs` | rust-bin | observed | Release | `docs/verification/CLI_SURFACE_EVIDENCE.md` | retained-evidence |
| `artifact_tool.py verify/hash/compare/inspect` | `scripts/artifact_tool.py` | python-tool | observed | Release | `make replay-tests` | retained-evidence |
| `artifact_tool.py capture/lock-baseline` | `scripts/artifact_tool.py` | python-tool | observed (hardware) | Release | hardware path | retained-evidence |
| `artifact_diff.py` | `scripts/artifact_diff.py` | python-tool | observed | Release | `make replay-tests` | retained-evidence |
| `replay-host diff` | `crates/replay-host/src/main.rs` | rust-bin | observed | Release (bounded to `artifacts/rpl0/` corpus) | 4 operator-path tests + retained transcripts | retained-evidence |
| `replay-host import-interval-csv` | `crates/replay-host/src/main.rs` | rust-bin | present in binary | Experimental | none in release gate | retained-evidence |
| `replay-host validate-interval-csv` | `crates/replay-host/src/main.rs` | rust-bin | present in binary | Experimental | none in release gate | retained-evidence |

---

## 5. Derived Release Surface

| Capability | Classification | User Path | Verification Path | Evidence | Confidence | Freshness |
|---|---|---|---|---|---|---|
| Deterministic validation gate | Release | `make gate` | `make gate` => VERIFICATION PASSED | retained: `docs/verification/releases/1.5.0/make_gate.txt` | direct | retained-evidence |
| Forensic artifact generation | Release | `precision generate`, `precision artifacts` | `docs/verification/CLI_SURFACE_EVIDENCE.md` | file: `docs/verification/CLI_SURFACE_EVIDENCE.md` | direct | retained-evidence |
| Artifact inspection/verification | Release | `precision inspect`, `precision verify`, `header_audit` | `docs/verification/CLI_SURFACE_EVIDENCE.md` | file: `docs/verification/CLI_SURFACE_EVIDENCE.md` | direct | retained-evidence |
| Python replay operator tooling | Release | `artifact_tool.py`, `artifact_diff.py` | `make replay-tests` => PASS (38 items) | retained: `docs/verification/releases/1.5.0/make_replay_tests.txt` | direct | retained-evidence |
| End-to-end evidence packaging | Release | `make demo-evidence-package` | retained bundle matches byte-for-byte | retained: `docs/verification/releases/1.5.0/make_demo_evidence_package.txt` | direct | retained-evidence |
| Dual-build reproducibility | Release | `bash verify_release_repro.sh` | bit-identical (SHA256 `7da5387c710c98b62ae876722350efdf4d9d1919211db9841c13933b9e3df714`) | retained: `docs/verification/releases/1.5.0/verify_release_repro.txt` | direct | retained-evidence |
| Formal verification (Kani Tier-1) | Release | `bash verify_kani.sh` | 24 harnesses all PASS; Tier-2 skipped (standard) | retained: `docs/verification/releases/1.5.0/kani_evidence.txt` | direct | retained-evidence |
| Rust replay diff (bounded) | Release (new at 1.5.0) | `cargo run -q -p replay-host -- diff <a.rpl> <b.rpl>` | 4 operator-path tests PASS; 3 retained transcripts | retained: `docs/verification/releases/1.5.0/RUST_REPLAY_DIFF_SCOPE.md`; retained: `docs/verification/releases/1.5.0/cargo_test_replay_host_operator_path.txt` | direct | retained-evidence |
| Bounded sine correctness (inherited) | Release | unit test in `dpw4` | retained: `docs/verification/releases/1.4.0/cargo_test_sine_bounded_correctness.txt` | file: `docs/verification/releases/1.4.0/VERIFICATION_SCOPE.md` | direct | retained-evidence |
| Hardware capture (F446) | Experimental | `make flash-ur` | retained: `docs/verification/releases/1.2.2/` | file: `crates/replay-fw-f446/` | direct | retained-evidence |
| Schema-aware Rust replay | Experimental | none | none | file: `docs/RELEASE_SURFACE.md` Experimental block | direct | retained-evidence |

---

## 6. Claim-Reality Gap Register

| # | Claim | Source | Status | Impact | Confidence | Notes |
|---|---|---|---|---|---|---|
| 1 | `make gate` => VERIFICATION PASSED | `README.md` §First 5 Minutes | `exact` | — | direct | retained: `docs/verification/releases/1.5.0/make_gate.txt`: VERIFICATION PASSED; workspace=1.5.0 lock=1.5.0 |
| 2 | `replay-host diff` released, bounded to `artifacts/rpl0/` corpus | `CHANGELOG.md` [1.5.0]; `docs/RELEASE_SURFACE.md`; `docs/replay/tooling.md` | `exact` | — | direct | Scope document precise; 3 command transcripts retained; 4 operator-path tests pass |
| 3 | Broader `replay-host` commands remain Experimental | `docs/RELEASE_SURFACE.md`; `docs/replay/tooling.md` | `exact` | — | direct | `import-interval-csv`, `validate-interval-csv` explicitly Experimental in both documents |
| 4 | `replay-fw-f446` is Experimental | `docs/RELEASE_SURFACE.md` | `exact` | — | direct | No change to firmware classification since 1.3.0 reclassification |
| 5 | Packaged proof route reproduces retained bundle byte-for-byte | `README.md`; `docs/demos/demo_evidence_packaging.md` | `exact` | — | direct | retained: `docs/verification/releases/1.5.0/make_demo_evidence_package.txt`: "retained bundle matches" |
| 6 | Workspace version = 1.5.0 across all crates | `Cargo.toml`; `VERIFICATION_GUIDE.md` | `exact` | — | direct | `workspace.package.version = "1.5.0"`; lock bumped all 10 crates; `make_gate.txt`: `version_consistency: workspace=1.5.0 lock=1.5.0` |
| 7 | Dual-build reproducibility | `docs/verification/build_reproducibility.md` | `exact` | — | direct | retained: SHA256 `7da5387c…` both builds identical |
| 8 | Documentation link integrity passes | `docs/README.md` | `exact` | — | direct | retained: `docs/verification/releases/1.5.0/make_doc_link_check.txt`: PASS |
| 9 | "Minor release because this cut adds one new released command surface" | `CHANGELOG.md` [1.5.0] Notes | `exact` | — | direct | Only `replay-host diff` promoted; no CLI, artifact format, or golden hash changes |
| 10 | Active release-scoped proof boundary in `docs/verification/releases/1.5.0/` | `VERIFICATION_GUIDE.md` §3.3 | `partial` | credibility | direct | Directory exists and contains `RUST_REPLAY_DIFF_SCOPE.md`; no `VERIFICATION_SCOPE.md` for inherited sine correctness bounds (see F-1) |
| 11 | "The active release (1.5.0) retains one explicit bounded correctness claim for the released sine path … documented in `docs/verification/releases/1.5.0/`" | `VERIFICATION_GUIDE.md` §3.6 | `partial` | credibility | direct | Sine correctness evidence lives in `docs/verification/releases/1.4.0/VERIFICATION_SCOPE.md`; 1.5.0 bundle has no sine scope document (see F-1) |
| 12 | Spec document carry-forward not updated to 1.5.0 | `docs/MATH_CONTRACT.md`; all `docs/spec/*.md` | `exact` (as carry-forward) | — | direct | Carry-forward pattern intentional for content-unchanged docs; absence consistent with prior releases; see F-2 |
| 13 | NAM-004: bare "v1" in Cargo.toml line 2 comment | `Cargo.toml` line 2 | `partial` | credibility (marginal) | direct | Pre-existing; low severity; unchanged from 1.4.0 baseline |

---

## 7. Documentation Topology Map

| Document | Purpose | Category | Public | Overlap | Overlap Severity | Authority | Change vs 1.4.0 |
|---|---|---|---|---|---|---|---|
| `README.md` | Entry routing | entry | public | none | — | canonical | 1.5.0 paragraph added |
| `docs/README.md` | Documentation index | index | public | none | — | canonical | unchanged |
| `docs/RELEASE_SURFACE.md` | Release classification and routing | release-classification | public | none | — | canonical | 1.5.0 version refs; `replay-host diff` added |
| `VERIFICATION_GUIDE.md` | Release contract and conformance governance | normative | public | none | — | canonical | Version header + routing updated; §3.6 updated; firmware procedure noise present at end (W-L4) |
| `docs/MATH_CONTRACT.md` | Arithmetic and signal-path contract | normative | public | none | — | canonical | unchanged (carry-forward at 1.4.0) |
| `docs/spec/rpl0_artifact_contract.md` | Binary artifact format spec | normative | public | none | — | canonical | unchanged (carry-forward at 1.4.0) |
| `docs/spec/dpw_gain.md` | Gain model invariants | normative | public | none | — | supporting | unchanged |
| `docs/spec/oscillator_api.md` | Oscillator dispatch contract | normative | public | none | — | supporting | unchanged |
| `docs/spec/reference_invariants.md` | Mathematical reference invariants | normative | public | none | — | supporting | unchanged |
| `docs/spec/pulse_implementation_spec.md` | Pulse/square spec | normative | public | none | — | supporting | unchanged |
| `docs/spec/header_layout_addendum.md` | Header layout spec | normative | public | none | — | supporting | unchanged |
| `docs/replay/tooling.md` | Replay-tooling boundary | release-classification | public | minor with RELEASE_SURFACE.md | minor | supporting | "Released Rust Replay" section added; experimental remainder section added |
| `docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md` | STM32 self-stimulus capture contract | normative | public | none | — | canonical | unchanged |
| `docs/replay/DIVERGENCE_SEMANTICS.md` | Divergence explanation contract | normative | public | none | — | canonical | unchanged |
| `docs/demos/demo_evidence_packaging.md` | Canonical packaged proof route | workflow | public | none | — | canonical | unchanged |
| `docs/verification/releases/1.5.0/README.md` | Release evidence inventory | normative | public | none | — | canonical | new |
| `docs/verification/releases/1.5.0/RUST_REPLAY_DIFF_SCOPE.md` | Bounded Rust replay release claim | normative | public | none | — | canonical | new |
| `docs/verification/releases/1.5.0/` | Retained release evidence bundle (15 files) | normative | public | none | — | canonical | new |
| `docs/verification/releases/1.4.0/` | Prior retained release evidence | historical | public | none | — | historical | now historical |
| `docs/verification/CLI_SURFACE_EVIDENCE.md` | CLI promotion evidence | workflow | public | none | — | supporting | unchanged |
| `docs/verification/build_reproducibility.md` | Build identity checks | workflow | public | none | — | supporting | unchanged |
| `docs/verification/CROSS_CONTEXT_INVARIANCE.md` | Cross-context replay invariance | workflow | public | none | — | supporting | unchanged |
| `docs/governance/DESIGN_AXIOMS.md` | Governance axioms | normative | public | none | — | supporting | unchanged |
| `docs/governance/DEBT.md` | Tracked design debt | reference | public | none | — | supporting | unchanged |
| `docs/verification/CI_EVIDENCE.md` | Historical CI evidence | historical-audit | public | none | — | historical | unchanged |
| `docs/audits/repository_health_1.4.0.md` | Prior repository health baseline | historical-audit | public | none | — | historical | unchanged |

---

## 8. Terminology Consistency Register

| Rule ID | File | Line | Classification | Severity | Evidence | Status vs 1.4.0 |
|---|---|---:|---|---|---|---|
| NAM-004 | `Cargo.toml` | 2 | ambiguous-version | warn | "The public v1 release boundary" — bare "v1" continues to be ambiguous between format version and release version | unchanged |
| NAM-007 | `VERIFICATION_GUIDE.md` | 178 | version routing | medium | "documented in `docs/verification/releases/1.5.0/`" implies sine correctness scope lives in 1.5.0 bundle; the documentation lives in `docs/verification/releases/1.4.0/VERIFICATION_SCOPE.md` | new at 1.5.0 |

All prior NAM violations from the 1.4.0 baseline remain resolved (NAM-001, NAM-002,
NAM-003, NAM-005, NAM-006). No new ambiguous capability-distorting version language
introduced in public-facing release documents.

The `replay-host` usage message exposes experimental subcommands
(`validate-interval-csv`, `import-interval-csv`) as a binary surface. Consistent
with Experimental classification in all release documents; noted under F-3.

---

## 9. Prior Finding Disposition

| Finding | Prior Status (1.4.0) | Current Status (1.5.0) | Evidence |
|---|---|---|---|
| W-1: Retained manifest absolute run_dir | unchanged | **unchanged** | `make release-bundle-check VERSION=1.5.0 => PASS` |
| W-2: Stale version headers + NAM-005 | unchanged (carry-forward, intentional) | **unchanged** | Spec docs carry "Applies to: release 1.4.0 (content unchanged)"; no 1.5.0 update — see F-2 |
| W-3: MATH_CONTRACT v1.2.1 label gap | unchanged (explicit carry-forward) | **unchanged** | Locked at 1.2.1, carried forward; stable |
| W-4: Performance docs classification | unchanged | **unchanged** | Reclassified as historical reference; stable |
| W-5: Supplementary `--mode full` evidence | unchanged | **unchanged** | `make gate-full` available; not re-executed this session |
| W-6: Kani verification freshness | improved (1.4.0) | **improved** | Fresh Kani evidence retained: `docs/verification/releases/1.5.0/kani_evidence.txt` |
| W-7: Orphaned documentation files | resolved | **unchanged** | `make doc-link-check => PASS` |
| W-8: Cross-context terminology | resolved | **unchanged** | Stable |
| N-1: NAM-004 bare "v1" in Cargo.toml | warn, pre-existing | **unchanged** | Low severity |
| N-2: `replay-fw-f446` Experimental alignment | improved (1.3.0) | **unchanged** | Stable |

New observations (1.5.0):

- F-1 (new, medium): VERIFICATION_GUIDE.md §3.6 sine correctness routing gap
- F-2 (new, low): Spec document carry-forward not updated to 1.5.0
- F-3 (new, low): Experimental subcommands visible in `replay-host` usage text
- F-4 (new, low, pre-existing): "One block phased" firmware procedure at end of VERIFICATION_GUIDE.md

---

## 10. Strengths

1. **Bounded release promotion is a model for precision-scoped evidence.** The
   `replay-host diff` promotion is backed by 3 exact command transcripts, 4
   operator-path test outputs, a corpus manifest, a scope document naming the
   unreleased remainder, and a `make release-bundle-check` pass. No capability
   claim exceeds retained proof.
   Evidence: file: `docs/verification/releases/1.5.0/RUST_REPLAY_DIFF_SCOPE.md`;
   retained: `docs/verification/releases/1.5.0/cargo_test_replay_host_operator_path.txt`

2. **Determinism discipline remains exemplary.** 7 golden hashes pass, dual-build
   bit-identical (SHA256 `7da5387c710c98b62ae876722350efdf4d9d1919211db9841c13933b9e3df714`),
   no hash regeneration required for this minor release (correct per §1.2 governance).
   Evidence: retained: `docs/verification/releases/1.5.0/make_gate.txt`;
   retained: `docs/verification/releases/1.5.0/release_reproducibility.txt`

3. **Kani coverage extended to `replay-core`.** Three new Tier-1 harnesses
   (`proof_v0_wire_size_constants`, `proof_encode_header0_wire_layout_and_le`,
   `proof_encode_event_frame0_wire_layout_and_le`) pass for `replay-core`, providing
   wire layout and LE encoding proofs that directly underpin the released
   `replay-host diff` surface. Total Tier-1 count: 24 harnesses, all pass.
   Evidence: retained: `docs/verification/releases/1.5.0/kani_evidence.txt`

4. **Release/experimental boundary explicitly sharpened.** `docs/replay/tooling.md`
   now names the exact released command, the exact corpus, and lists three distinct
   experimental-remainder categories with specific command names.
   Evidence: file: `docs/replay/tooling.md` §Experimental Rust Replay

5. **Retained evidence chain extended immutably.** 1.5.0 bundle complete and clean;
   older bundles verified unmodified per `docs/verification/releases/1.5.0/README.md`.
   Evidence: retained: `docs/verification/releases/1.5.0/make_release_bundle_check.txt` PASS

6. **Agent governance improvement.** Release Readiness Audit Mode added to
   `.github/agents/repository-auditor.agent.md`, formalizing the audit contract
   with explicit authority order, no-go conditions, and output structure.
   Evidence: file: `.github/agents/repository-auditor.agent.md`

---

## 11. Weaknesses

### F-1 — Sine Correctness Routing Gap in VERIFICATION_GUIDE.md §3.6

Finding:
`VERIFICATION_GUIDE.md` §3.6 states the bounded sine correctness claim is
"documented in `docs/verification/releases/1.5.0/`." That directory contains no
sine scope document. The actual documentation lives in
`docs/verification/releases/1.4.0/VERIFICATION_SCOPE.md`. A cold reviewer
following the §3.6 pointer cannot locate the claimed proof bounds.

Evidence:
file: `VERIFICATION_GUIDE.md` line 178;
file: `docs/verification/releases/1.5.0/` (no `VERIFICATION_SCOPE.md`);
file: `docs/verification/releases/1.4.0/VERIFICATION_SCOPE.md`

Impact: `credibility` | `onboarding`

Severity: `medium`

Classification: `documentation`

Confidence: `direct`

Recommended Direction:
Update §3.6 to explicitly route to
`docs/verification/releases/1.4.0/VERIFICATION_SCOPE.md` for the inherited sine
claim, or add a minimal carry-forward scope note to the 1.5.0 bundle.

---

### F-2 — Spec Document Carry-Forward Not Updated to 1.5.0

Finding:
All six spec documents under `docs/spec/` and `docs/MATH_CONTRACT.md` still carry
"Applies to: release 1.4.0 (content unchanged)". The established discipline adds a
carry-forward line per release when content is unchanged.

Evidence:
file: `docs/MATH_CONTRACT.md` (carry-forward note at 1.4.0 only);
file: `docs/spec/rpl0_artifact_contract.md` (same pattern)

Impact: `onboarding`

Severity: `low`

Classification: `documentation`

Confidence: `direct`

Recommended Direction:
Add "Applies to: release 1.5.0 (content unchanged)" carry-forward entries.
Low churn; consistent with established discipline.

---

### F-3 — Experimental Subcommands Visible in Released Binary Usage Text

Finding:
The `replay-host` binary exposes `validate-interval-csv` and
`import-interval-csv` in the usage/error text emitted when `diff` is invoked
with missing arguments, as retained in
`docs/verification/releases/1.5.0/replay_host_diff_missing_arg.txt`. All release
documents correctly classify these as Experimental, but the binary output could
mislead users about their support status.

Evidence:
retained: `docs/verification/releases/1.5.0/replay_host_diff_missing_arg.txt`;
file: `docs/RELEASE_SURFACE.md` Experimental block

Impact: `onboarding`

Severity: `low`

Classification: `implementation`

Confidence: `direct`

Recommended Direction:
Acceptable at low severity given unambiguous release document classification.
No urgent action required. If experimental surface grows, consider annotating
the usage text or adding a scope note to the released binary's help output.

---

### W-L4 — Firmware Procedure Noise in VERIFICATION_GUIDE.md (pre-existing)

Finding:
A raw firmware procedure block ("One block phased") is appended after the formal
Conclusion section of `VERIFICATION_GUIDE.md`. It is not part of the release
authority and is confusing noise in the canonical contract document.

Evidence:
file: `VERIFICATION_GUIDE.md` lines ~479–538 (firmware procedure block)

Impact: `onboarding`

Severity: `low`

Classification: `documentation`

Confidence: `direct`

Recommended Direction:
Relocate to `docs/verification/hardware_procedures.md` or delete.
Post-1.5.0 cleanup item; not a release blocker.

---

### Residual low severity (unchanged from 1.4.0)

W-L1 (NAM-004): `Cargo.toml` line 2 bare "v1" comment — low severity, pre-existing.

---

## 12. Remediation Priorities

| Priority | Finding | Severity | Churn | Recommended Action |
|---|---|---|---|---|
| 1 | F-1: Sine correctness routing gap in VERIFICATION_GUIDE.md §3.6 | medium | very low | Update §3.6 to explicitly route to `docs/verification/releases/1.4.0/VERIFICATION_SCOPE.md` for sine domain |
| 2 | F-2: Spec carry-forward not updated to 1.5.0 | low | low | Add 1.5.0 carry-forward lines to `docs/MATH_CONTRACT.md` and all `docs/spec/*.md` |
| 3 | W-L4: Firmware procedure noise in VERIFICATION_GUIDE.md | low | low | Relocate or delete "One block phased" block |
| 4 | W-L1 (NAM-004): bare "v1" in Cargo.toml | warn | low | Disambiguate to "format v1" or remove comment |
| 5 | F-3: Experimental CLI surface in released binary usage text | low | medium | Note in scope document or defer; no urgent action |

---

## 13. Evidence and Unknowns

### Direct evidence (observed this session)

- command: `make gate` (run in terminal, exit code 0): VERIFICATION PASSED

### Retained evidence (primary basis for this audit)

- command: `make gate` transcript: `docs/verification/releases/1.5.0/make_gate.txt`
  — VERIFICATION PASSED; workspace=1.5.0 lock=1.5.0; 7 golden hashes matched
- command: `make demo-evidence-package`: `docs/verification/releases/1.5.0/make_demo_evidence_package.txt`
  — generated bundle; retained bundle matches; replay diff identical; first divergence at frame 17
- command: `make doc-link-check`: `docs/verification/releases/1.5.0/make_doc_link_check.txt`
  — PASS
- command: `make release-bundle-check VERSION=1.5.0`: `docs/verification/releases/1.5.0/make_release_bundle_check.txt`
  — PASS
- command: `make replay-tests`: `docs/verification/releases/1.5.0/make_replay_tests.txt`
  — 38 PASS items (all parser, tool, demo, fixture suites)
- command: `bash verify_release_repro.sh`: `docs/verification/releases/1.5.0/verify_release_repro.txt`
  — PASS; SHA256 `7da5387c710c98b62ae876722350efdf4d9d1919211db9841c13933b9e3df714`
- command: `cargo test -p replay-host operator_path_reports_`: `docs/verification/releases/1.5.0/cargo_test_replay_host_operator_path.txt`
  — 4 tests pass, 0 fail
- command: `bash verify_kani.sh`: `docs/verification/releases/1.5.0/kani_evidence.txt`
  — 24 Tier-1 harnesses PASS; Tier-2 skipped (standard, footnoted)
- command: `cargo check -p dpw4 --target thumbv7em-none-eabihf --locked`: `docs/verification/releases/1.5.0/cargo_check_dpw4_thumb_locked.txt`
  — Finished
- command: `cargo run -q -p replay-host -- diff … (identical)`: `docs/verification/releases/1.5.0/replay_host_diff_identical.txt`
  — "no divergence"
- command: `cargo run -q -p replay-host -- diff … (frame17)`: `docs/verification/releases/1.5.0/replay_host_diff_frame17.txt`
  — "first divergence at frame 17"
- command: `cargo run -q -p replay-host -- diff … (missing arg)`: `docs/verification/releases/1.5.0/replay_host_diff_missing_arg.txt`
  — non-zero exit; usage text exposing experimental subcommands

### Unknowns

- `make ci-local` not run as a single compound command this session; individual
  components verified via retained evidence.
- GitHub Actions workflow files not inspected (branch-local audit only).
- Heavy Tier-2 Kani proofs not executed this session; disposition: retained
  evidence from `kani_evidence.txt` shows Tier-2 skipped with WARN; per standard
  policy; does not affect release admissibility.
- `docs/spec/*.md` and `docs/MATH_CONTRACT.md` carry-forward state inferred from
  diff.txt showing no changes to those files.

---

## Final Question Answers

**1. What executable capabilities are actually implemented today?**

All capabilities from the 1.4.0 baseline remain implemented unchanged. One new
capability is now Release-classified: `replay-host diff`, bounded to the retained
`artifacts/rpl0/` proof corpus. Two additional `replay-host` subcommands
(`import-interval-csv`, `validate-interval-csv`) are present in the binary but
Experimental. The full precision CLI (5 subcommands plus `header_audit`), Python
tooling (`artifact_tool.py`, `artifact_diff.py`), and `make demo-evidence-package`
evidence pipeline are all intact and verified.

**2. How accurately does the repository communicate those capabilities?**

Very accurately, with one medium gap (F-1). The `replay-host diff` promotion is
exceptionally well-scoped across all three classification documents. The single
inaccuracy is `VERIFICATION_GUIDE.md` §3.6's pointer to the 1.5.0 bundle for sine
correctness documentation that actually lives in the 1.4.0 bundle. All other
claims are exact. No overclaim; no underclaim.

**3. What single sprint would most improve repository credibility and clarity?**

Fix F-1 (update §3.6 sine routing pointer to `docs/verification/releases/1.4.0/VERIFICATION_SCOPE.md`),
add 1.5.0 carry-forward lines to spec documents (F-2), and remove the "One block
phased" firmware procedure from `VERIFICATION_GUIDE.md` (W-L4). All three together
are a 30-minute documentation sprint that eliminates every remaining medium or low
finding.

---
