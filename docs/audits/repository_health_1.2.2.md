# Repository Health Audit — 1.2.2 (baseline refresh)

**Audit date:** 2026-03-26
**Auditor:** automated agent (Claude Opus 4.6)
**Workspace version:** 1.2.2
**Toolchain:** rustc 1.91.1 (pinned via rust-toolchain.toml)
**Branch:** feat/release-1.2.2
**HEAD:** 498943d
**Prior audit HEAD:** d03be96

---

## 1. Audit Scope Summary

This audit refreshes the repository health baseline for `precision-signal` at
workspace version 1.2.2 (HEAD `498943d`), incorporating changes since the prior
audit at HEAD `d03be96`.

Changes since prior audit:
- Cross-context invariance verification added (`docs/verification/CROSS_CONTEXT_INVARIANCE.md` and retained evidence under `docs/verification/cross_context/`)
- Supplementary `make gate-full` evidence file retained under `docs/verification/releases/1.2.2/`
- Fresh Kani Tier-1 evidence retained under `docs/verification/releases/1.2.2/kani_evidence.md`

Scope:
- Cargo workspace (10 crates, 4 default members)
- Two Rust CLI binaries: `precision`, `header_audit`
- Two released Python operator tools: `artifact_tool.py`, `artifact_diff.py`
- One experimental Rust binary: `replay-host`
- Firmware crate: `replay-fw-f446`
- Verification workflows: `make gate`, `make replay-tests`, `make release-bundle-check`
- Documentation set under `docs/`
- Retained release evidence under `docs/verification/releases/1.2.2/`

Prior audit: `docs/audits/repository_health_1.2.2.md` at HEAD `d03be96`.
All four original findings (W-1 through W-4) remain resolved.
Two supplementary findings (W-5, W-6) remain improved.
New findings identified in this session: W-7, W-8.

---

## 2. Metric Scores

| Metric | Score | Justification | Evidence |
|---|---:|---|---|
| Engineering integrity | 88 | Pinned toolchain, `#![forbid(unsafe_code)]`, determinism hashes locked to semver bumps, float quarantine enforced | file: `rust-toolchain.toml`; file: `VERIFICATION_GUIDE.md` §1.2; command-output: `make check-workspace => ok` |
| Determinism / reproducibility | 91 | 7 golden hashes pass, dual-build bit-identical, fixture-drift gate clean, cross-context invariance adds new evidence layer | command-output: `make gate => VERIFICATION PASSED`; command-output: `bash verify_release_repro.sh => PASS`; file: `docs/verification/CROSS_CONTEXT_INVARIANCE.md` |
| Specification quality | 85 | All 6 spec docs have versioning terminology blocks; RPL0 contract byte-level; MATH_CONTRACT dense with carry-forward note | file: `docs/spec/rpl0_artifact_contract.md`; file: `docs/MATH_CONTRACT.md` |
| Verification discipline | 88 | Multi-tier gate chain; `make release-bundle-check` passes cleanly; cross-context invariance check retained; Kani fresh for 1.2.2 | command-output: `make gate => PASS`; command-output: `make release-bundle-check VERSION=1.2.2 => PASS`; file: `docs/verification/releases/1.2.2/kani_evidence.md` |
| Codebase maintainability | 78 | Workspace well-factored (core crates zero-dep); some crates are stubs; 10 members with 4 default | file: `Cargo.toml` workspace members |
| Architecture clarity | 80 | Clear release/experimental boundary; workspace.md routes well | file: `docs/RELEASE_SURFACE.md`; file: `docs/architecture/workspace.md` |
| Documentation depth | 79 | Normative specs thorough; replay docs cover operator workflows; several verification docs are orphaned from index | file: `docs/README.md` (index routing); finding: W-7 |
| Documentation organization | 77 | Top-level routing clear but several new and existing docs are unreachable from the documentation index | file: `docs/README.md`; finding: W-7, W-8 |
| Repository presentation | 82 | README terse and correct; no inflated claims; release surface distinct from experimental | file: `README.md` |
| Developer onboarding | 85 | "First 5 Minutes" block; carry-forward notes eliminate version-label confusion; all versioning terminology blocks present in specs | file: `README.md` §First 5 Minutes; file: `docs/MATH_CONTRACT.md` carry-forward note |
| Conceptual coherence | 85 | Consistent "deterministic execution analysis infrastructure"; normative/advisory distinction maintained | file: `VERIFICATION_GUIDE.md` §1.1; file: `docs/README.md` |
| Research / innovation value | 76 | Phase-domain fixed-point oscillator with formal proofs; hardware-backed determinism; cross-context invariance check is novel; narrow domain | file: `docs/MATH_CONTRACT.md`; file: `docs/verification/CROSS_CONTEXT_INVARIANCE.md` |
| OSS trustworthiness signals | 82 | MIT license, pinned toolchain, hash-locked releases, retained evidence bundles, doc-link CI, all manifests repo-relative, cross-context evidence retained | file: `LICENSE`; file: `docs/verification/releases/1.2.2/replay_manifest_v1.txt` |

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
| `artifact_tool.py capture` | `scripts/artifact_tool.py` | python-tool | observed (hardware-dependent) | `make replay-check` | Release |
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
| Rust replay (v0 frames) | yes | Experimental | `cargo run -p replay-host -- diff` | cross-context comparison only | file: `docs/verification/CROSS_CONTEXT_INVARIANCE.md` | direct |
| Formal verification (Kani) | yes | Release | `bash verify_kani.sh` | retained evidence: `docs/verification/releases/1.2.2/kani_evidence.md` | file: `verify_kani.sh`; file: `docs/verification/releases/1.2.2/kani_evidence.md` | direct |
| Dual-build reproducibility | yes | Release | `bash verify_release_repro.sh` | command-output: `PASS — builds are bit-for-bit identical` | command-output: observed-this-session | direct |
| Cross-context invariance | yes | Reference | `docs/verification/CROSS_CONTEXT_INVARIANCE.md` | retained evidence: `docs/verification/cross_context/` | file: `docs/verification/CROSS_CONTEXT_INVARIANCE.md` | direct |

---

## 6. Claim-Reality Gap Register

| # | Claim | Source | Status | Impact | Confidence | Notes |
|---|---|---|---|---|---|---|
| 1 | `make gate` => VERIFICATION PASSED | README.md §First 5 Minutes | exact | — | direct | command-output: `make gate => VERIFICATION PASSED` (observed-this-session) |
| 2 | Released replay-facing operator tooling is the Python toolchain | docs/replay/tooling.md | exact | — | direct | `artifact_tool.py` and `artifact_diff.py` exercised via `make replay-tool-tests` |
| 3 | Rust replay is experimental and not part of release surface | docs/RELEASE_SURFACE.md, docs/replay/tooling.md | exact | — | direct | `replay-host` description: "not classified as released"; not in `make gate` |
| 4 | Hardware capture re-executed for 1.2.2 | docs/verification/releases/1.2.2/firmware_release_evidence.md | exact | — | retained-evidence | 5/5 runs identical, SHA256 `f79e71d...` consistent across all retained files |
| 5 | Deterministic repeatability: PASS (5/5 runs identical) | docs/verification/releases/1.2.2/firmware_release_evidence.md | exact | — | retained-evidence | sha256_summary.txt shows 6 identical hashes including baseline |
| 6 | MATH_CONTRACT locked at v1.2.1, carried forward to 1.2.2 | docs/MATH_CONTRACT.md | exact | — | direct | Carry-forward note and versioning terminology block present; axis ambiguity resolved |
| 7 | Spec docs applicable to release 1.2.2 | docs/spec/*.md | exact | — | direct | All spec docs carry "Applies to: release 1.2.2 (content unchanged)" and versioning terminology blocks |
| 8 | Cross-context invariance PASS | docs/verification/CROSS_CONTEXT_INVARIANCE.md | exact | — | direct | Retained evidence under `docs/verification/cross_context/`; comparison_summary.md records byte-identical cross-context results |
| 9 | Dual-build reproducibility | docs/verification/build_reproducibility.md | exact | — | direct | command-output: `bash verify_release_repro.sh => PASS` (observed-this-session) |

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
| docs/cli/precision.md | CLI reference | reference | public | none | supporting |
| docs/architecture/workspace.md | Workspace framing | deep-architecture | public | none | supporting |
| docs/system_architecture_disclosure.md | Architecture overview | descriptive | public | minor (architecture overview) | supporting |
| docs/verification/build_reproducibility.md | Build identity checks | workflow | public | none | supporting |
| docs/verification/CI_EVIDENCE.md | Historical CI evidence | historical-audit | public | none | historical |
| docs/verification/CLI_SURFACE_EVIDENCE.md | CLI promotion evidence | workflow | public | none | supporting |
| docs/verification/CROSS_CONTEXT_INVARIANCE.md | Cross-context replay invariance | workflow | public | none | supporting |
| docs/verification/chaos_probes.md | Advisory chaos probes | workflow | public | none | supporting |
| docs/verification/FIRMWARE_CAPTURE_EVIDENCE.md | Historical firmware capture evidence | historical-audit | public | none | historical |
| docs/verification/D-03_TriangleDPW4_Audit.md | Triangle DPW4 audit | historical-audit | public | none | historical |
| docs/verification/releases/1.2.2/ | Retained release evidence bundle | normative | public | none | canonical |
| docs/governance/DESIGN_AXIOMS.md | Governance axioms | normative | public | none | supporting |
| docs/spec/dpw_gain.md | Gain spec | normative | public | none | supporting |
| docs/spec/oscillator_api.md | Oscillator dispatch contract | normative | public | none | supporting |
| docs/spec/reference_invariants.md | Mathematical invariants | normative | public | none | supporting |
| docs/spec/pulse_implementation_spec.md | Pulse/square spec | normative | public | none | supporting |
| docs/spec/header_layout_addendum.md | Header layout | normative | public | none | supporting |
| docs/replay/ISR_ADVISORY.md | ISR hardening findings | reference | public | none | supporting |
| docs/replay/REPLAY_CAPTURE_CONTRACT_v0.md | Legacy v0 capture contract | historical-audit | public | none | historical |
| docs/architecture/performance/ | Performance benchmarking docs | historical-reference | internal | none | historical |
| docs/internal/ | Internal normalization notes | reference | internal | none | supporting |
| docs/roadmap/witness_model_direction.md | Post-1.2.2 architecture direction | roadmap | public | none | supporting |

---

## 8. Terminology Consistency Register

| Rule ID | File | Line | Classification | Severity | Evidence | Notes |
|---|---|---:|---|---|---|---|
| NAM-004 | docs/spec/rpl0_artifact_contract.md | 55 | ambiguous-version | info | "v1 parsing path" | Adequately disambiguated by Versioning Terminology block at top of file |
| NAM-004 | docs/spec/oscillator_api.md | 14 | ambiguous-version | info | "v1.x line" | Refers to release version range; context adequate with terminology block |
| NAM-004 | docs/verification/CROSS_CONTEXT_INVARIANCE.md | 9 | ambiguous-version | warn | "RPL0 v1 container artifacts" | New doc; context ("RPL0 ... container") partially disambiguates but no versioning terminology block |
| NAM-004 | docs/verification/cross_context/comparison_summary.md | 8 | ambiguous-version | warn | "RPL0 v1 fixture generator" | New doc; same partial disambiguation, no versioning terminology block |

No NAM-001 (artifact vX), NAM-002 (replay vX), NAM-003 (_vX.md artifact spec),
NAM-005 (missing terminology block in canonical specs), or NAM-006 (capability
statement) violations found in public-facing normative or release-surface documents.

NAM-005 does not apply to `CROSS_CONTEXT_INVARIANCE.md` or `comparison_summary.md`
because they are supporting evidence docs, not canonical specs. The NAM-004 findings
above are informational.

---

## 9. Prior Finding Disposition

| Finding | Prior Status | Current Status | Evidence |
|---|---|---|---|
| W-1: Retained manifest absolute run_dir | improved | **unchanged** | `replay_manifest_v1.txt` uses `run_dir=artifacts/replay_runs/...`; `make release-bundle-check VERSION=1.2.2 => PASS` |
| W-2: Stale v1.0.0-rc5 version headers + NAM-005 | improved | **unchanged** | All spec docs have `Document revision:` + `Applies to:` + Versioning Terminology blocks |
| W-3: MATH_CONTRACT v1.2.1 label gap | improved | **unchanged** | Carry-forward note, versioning terminology block, and "Applies to: release 1.2.2 (content unchanged)" present |
| W-4: Performance docs classification | improved | **unchanged** | Reclassified as "Historical reference" with terminology blocks |
| W-5: Supplementary `--mode full` evidence | improved | **unchanged** | `make gate-full => VERIFICATION PASSED`; retained: `docs/verification/releases/1.2.2/gate_full_evidence.md` |
| W-6: Kani verification freshness | improved | **unchanged** | `docs/verification/releases/1.2.2/kani_evidence.md` retained; Tier-1 23 harnesses PASS |

---

## 10. Strengths

1. **Determinism discipline is exemplary.** 7 golden hashes locked to semver bumps,
   hardware-backed 5/5 repeatability, fixture-drift gate prevents silent regression,
   dual-build reproducibility verified bit-for-bit identical.
   Evidence: `make gate => VERIFICATION PASSED` (observed-this-session);
   `bash verify_release_repro.sh => PASS` (observed-this-session);
   file: `docs/verification/releases/1.2.2/sha256_summary.txt`.

2. **Release/experimental boundary is well-drawn.** `RELEASE_SURFACE.md` explicitly
   classifies every surface; `replay-host` description says "not classified as released";
   `docs/replay/tooling.md` reinforces this. No inflation observed.

3. **Verification gate chain is layered and runnable.** `make gate`, `make replay-tests`,
   `make release-bundle-check`, `make fixture-drift-check` all pass.
   Evidence: all four observed-this-session.

4. **Documentation routing is clear at top level.** README → docs/README.md →
   RELEASE_SURFACE.md and VERIFICATION_GUIDE.md. Normative vs descriptive
   distinction maintained throughout.

5. **Normative specs are binding and precise.** `rpl0_artifact_contract.md` has
   byte-level layout, versioning terminology block, and hard invariants.
   `MATH_CONTRACT.md` has code-line references and Kani proof citations.

6. **Retained release evidence is structured and portable.** All manifests use
   repo-relative paths. `make release-bundle-check VERSION=1.2.2` passes cleanly.

7. **Cross-context invariance adds a new evidence layer.** The cross-context
   check independently validated that Python and Rust replay paths agree on
   divergence frame attribution across two independent context directories.
   Evidence: file: `docs/verification/CROSS_CONTEXT_INVARIANCE.md`.

---

## 11. Weaknesses

### Finding W-7 (new): Orphaned documentation files not reachable from index

Several public documentation files are not linked from `docs/README.md` or
any document in the index reading path:

- `docs/verification/CROSS_CONTEXT_INVARIANCE.md` — not indexed
- `docs/verification/chaos_probes.md` — not indexed
- `docs/verification/FIRMWARE_CAPTURE_EVIDENCE.md` — not indexed
- `docs/verification/D-03_TriangleDPW4_Audit.md` — not indexed
- `docs/replay/ISR_ADVISORY.md` — not indexed, not linked from any doc
- `docs/cli/precision.md` — not indexed (listed in prior audit topology but not actually linked)
- `docs/cli/examples.md` — not indexed
- `docs/demos/demo_visual.html` — not indexed
- `docs/roadmap/witness_model_direction.md` — not indexed

Evidence:
- file: `docs/README.md` — does not contain links to the above files
- grep: no inbound links found for `ISR_ADVISORY.md`, `CROSS_CONTEXT_INVARIANCE.md`,
  `chaos_probes.md`, `FIRMWARE_CAPTURE_EVIDENCE.md`, `D-03_TriangleDPW4_Audit.md`,
  `cli/precision.md`, `cli/examples.md`, `demo_visual.html`, or `witness_model_direction.md`
  in `docs/README.md`

Note: Some files (e.g., `CROSS_CONTEXT_INVARIANCE.md`) are structurally reachable
via `docs/verification/` directory listing but not via any clickable link from the
documentation index.

Impact: `onboarding`
Severity: `medium`
Classification: `topology`
Confidence: `direct`

Recommended Direction:
Add entries for the above files to the appropriate section of `docs/README.md`.
Group verification evidence docs under the existing Verification section.
Group CLI docs under a new CLI section. Group the roadmap doc under a new Roadmap section.
For `ISR_ADVISORY.md`, add it to the replay section of `docs/replay/README.md` or
the main index.

### Finding W-8 (new): New cross-context docs lack versioning terminology block

The new `docs/verification/CROSS_CONTEXT_INVARIANCE.md` and
`docs/verification/cross_context/comparison_summary.md` use `RPL0 v1` without
a versioning terminology block.

These are supporting evidence docs, not canonical specs, so NAM-005 does not
formally apply. However, for consistency with the repository's established
disambiguation practice, a short note clarifying that `v1` refers to format
version would align with spec-level hygiene.

Evidence:
- file: `docs/verification/CROSS_CONTEXT_INVARIANCE.md` line 9: "RPL0 v1 container artifacts"
- file: `docs/verification/cross_context/comparison_summary.md` line 8: "RPL0 v1 fixture generator"

Impact: `maintainability`
Severity: `low`
Classification: `terminology`
Confidence: `direct`

Recommended Direction:
Add a one-line note (e.g., "`v1` here refers to RPL0 format version 1") to the
scope section of both files.

---

## 12. Remediation Priorities

| Priority | Finding | Effort | Impact |
|---:|---|---|---|
| 1 | W-7: Index orphaned docs in `docs/README.md` | low (add ~10 index entries) | onboarding |
| 2 | W-8: Add versioning note to cross-context docs | trivial (two one-line additions) | maintainability |

---

## 13. Evidence and Unknowns

### Direct evidence (observed-this-session)

- command: `make gate` => `VERIFICATION PASSED` (7 determinism hashes matched)
- command: `make gate-full` => `VERIFICATION PASSED` (supplementary full-mode)
- command: `make test` => all workspace tests pass (64 tests, 4 ignored, 0 failed)
- command: `make replay-tests` => all parser + tool + demo suites PASS
- command: `make release-bundle-check VERSION=1.2.2` => `PASS` (no warnings)
- command: `make check-workspace` => `ok`
- command: `make fixture-drift-check` => `ok` (no drift)
- command: `python3 scripts/check_doc_links.py` => `PASS`
- command: `bash verify_release_repro.sh` => `PASS — builds are bit-for-bit identical` (SHA256: `c9cf726d...`)

### Retained evidence

- file: `docs/verification/releases/1.2.2/firmware_release_evidence.md` — hardware capture 5/5 PASS
- file: `docs/verification/releases/1.2.2/kani_evidence.md` — fresh Tier-1 Kani evidence for `1.2.2` (23 harnesses, 470s)
- file: `docs/verification/releases/1.2.2/gate_full_evidence.md` — supplementary `--mode full` retained record
- file: `docs/verification/releases/1.2.2/sha256_summary.txt` — 6 identical hashes
- file: `docs/verification/releases/1.2.2/replay_manifest_v1.txt` — manifest with `final_status=PASS`
- file: `docs/verification/CROSS_CONTEXT_INVARIANCE.md` — cross-context invariance PASS
- file: `docs/verification/cross_context/comparison_summary.md` — byte-identical cross-context results
- file: `docs/verification/CI_EVIDENCE.md` — CI run 23230032284 at v1.2.0-rc1

### Unknowns

- `make ci-local` was not run as a single compound command (individual components verified independently).
- No GitHub Actions workflow file was inspected (branch-local audit only).
- `bash verify_kani.sh` was not re-executed this session; retained evidence from prior session accepted.

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
- Cross-context invariance verification (reference evidence)
- Dual-build reproducibility check (`verify_release_repro.sh`)

### 2. How accurately does the repository communicate those capabilities?

Accurately for the release surface. All release claims match implementation and
verification evidence. The release/experimental boundary is explicitly drawn and
consistently maintained. No claim-reality contradictions were identified.

The primary gap is discoverability: nine documentation files are not reachable
from the docs index. This does not misrepresent capabilities but reduces
onboarding efficiency.

### 3. What single sprint would most improve repository credibility and clarity?

**Index completeness.** Adding the orphaned docs (W-7) to `docs/README.md`
would make all public verification evidence, CLI reference, and roadmap
material discoverable from the canonical reading path. This is a low-effort
change with direct onboarding impact. The optional versioning note (W-8) can
be addressed in the same pass.
