# NON-NORMATIVE / EXPERIMENTAL

This append-only log is exploratory only. It does not define current release surface,
verification authority, or normative invariants. If an item matures, promote it to
its target document; do not treat this log as authoritative.

## 2026-03-29 — Adversarial corpus generator [WIP-011]
Status: closed (PASS-constrained)
Owner: signal

Problem
We need a deterministic host-only adversarial search that tries to falsify the
WIP-009 residue-index rule inside the current probe value domain before any
broader domain expansion.

Hypothesis
If the WIP-009 rule captures the checked-in probe boundary for `q in {2, 3}`,
then predicted and observed `first_divergence_frame` should continue to match
across an explicitly bounded adversarial corpus family that includes early,
delayed, alternating, clustered, and locally reordered residue placements.

Constraints
- Host-only
- `q in {2, 3}` only
- No artifact contract change
- No replay semantic change
- No classification logic change
- No changelog update
- Keep helper experiment-local
- Deterministic search only

Canonical analysis
- pipeline reference:
  `experiments/quantization_probe/generate_probe_artifact.py`
- experiment-local helper:
  `PYTHONPATH=. python3 experiments/quantization_probe/analysis/wip011_adversarial_search.py`
- governing rule under test:
  `predicted_first_divergence_frame(corpus, q) = min i such that ((5 * corpus[i] + 3) & ((1 << q) - 1)) != 0`
- bounded search space:
  corpus length = `12`
  allowed sample values = `{1, 6}` from the current checked-in probe corpora
  generation family = exhaustive over every `12`-frame corpus in `{1, 6}^12`
  generation encoding = `12`-bit mask where bit `i = 1` maps frame `i` to `6`
  and bit `i = 0` maps frame `i` to `1`
  family size = `2^12 = 4096` corpora, producing `8192` checked `(corpus, q)`
  cases across `q in {2, 3}`
- required stressor coverage is included as subsets of the exhaustive family:
  early-residue placement
  delayed-residue placement
  alternating residue/non-residue patterns
  clustered residue bursts
  long zero-residue prefixes with late transitions
  local reorderings around the predicted boundary
- helper behavior:
  generate the bounded corpus family deterministically, compute the residue-index
  prediction, run the checked-in host pipeline, compare predicted vs observed
  first divergence, record repeatability, and emit compact JSON containing search
  metadata, per-case rows, and counterexamples if any exist

Evidence Produced
- Exhaustive search completed over all `4096` corpora in the bounded family
- All `4096` `Q2` cases matched prediction to observation exactly
- All `4096` `Q3` cases matched prediction to observation exactly
- No baseline repeatability failures were observed
- No quantized repeatability failures were observed
- No concrete counterexample was found in the tested search space
- One no-divergence corpus exists in the bounded family: the all-`1` corpus
  yields predicted `null` and observed `null` for both `Q2` and `Q3`
- Representative stressor witnesses produced by the helper include:
  early-residue placement: `B001 = [6, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]`
  delayed-residue placement: `B200 = [1, 1, 1, 1, 1, 1, 1, 1, 1, 6, 1, 1]`
  alternating residue/non-residue: `B555 = [6, 1, 6, 1, 6, 1, 6, 1, 6, 1, 6, 1]`
  clustered residue burst: `B003 = [6, 6, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]`
  long zero-residue prefix with late transition:
  `B100 = [1, 1, 1, 1, 1, 1, 1, 1, 6, 1, 1, 1]`
  local reordering around the predicted boundary:
  `B002 = [1, 6, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]`
- Observed classification mix was identical for `Q2` and `Q3` across this
  bounded family:
  `null: 1`, `persistent_offset: 12`, `rate_divergence: 4083`

Bounded validity
- No counterexample was found in the tested adversarial host search space
- This is bounded evidence only: the search covers length-`12` corpora over the
  current probe value domain `{1, 6}` and does not establish the rule as a
  universal property outside that family, outside `q in {2, 3}`, or under any
  changed pipeline semantics

Classification
- PASS-constrained

Next Decision
- Retain the residue-index rule as experiment-local and bounded
- If stronger falsification pressure is needed, widen the value domain next
  while keeping the same host-only comparison harness

Promotion Path
experiment-local retention only

## 2026-03-29 — Model robustness and counterexample search [WIP-010]
Status: closed (PASS-constrained)
Owner: signal

Problem
We need to try to falsify the WIP-009 residue-index rule by checking whether
predicted and observed `first_divergence_frame` remain equal on additional
host-only corpus shapes beyond `C1` and `C2`.

Hypothesis
If WIP-009 captures the real collapse trigger for the checked-in probe
pipeline, the first frame with non-zero affine residue should continue to match
the observed `Q2/Q3` first-divergence frame across a small but shape-diverse
host corpus family.

Constraints
- Host-only
- `q in {2, 3}` only
- No artifact contract change
- No replay semantic change
- No classification logic change
- No changelog update
- Keep implementation minimal and experiment-local

Canonical analysis
- pipeline reference:
  `experiments/quantization_probe/generate_probe_artifact.py`
- experiment-local helper:
  `python3 experiments/quantization_probe/analysis/wip010_model_robustness.py`
- governing rule under test:
  `predicted_first_divergence_frame(corpus, q) = min i such that ((5 * corpus[i] + 3) & ((1 << q) - 1)) != 0`
- added corpus family:
  `P1_front_loaded_six = [6, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]`
  `M1_monotonic_plateau_then_rise = [1, 1, 1, 1, 5, 5, 6, 6, 6, 6, 6, 6]`
  `O1_alternating_low_high = [1, 6, 1, 6, 1, 6, 1, 6, 1, 6, 1, 6]`
  `K1_plateau_then_jump = [1, 1, 1, 1, 1, 1, 6, 6, 6, 6, 6, 6]`
- helper behavior:
  reuse the checked-in host pipeline and artifact encoder, compute residue-index
  prediction per corpus and quant shift, check baseline/quantized repeatability,
  and reuse `scripts/artifact_diff.py` classification logic on encoded artifacts

Evidence Produced
- All tested baseline repeatability checks passed
- All tested quantized repeatability checks passed
- No prediction/observation mismatch was found in the added host corpus family
- The tested family includes:
  permutation-sensitive placement (`P1`)
  monotonic nondecreasing structure (`M1`)
  oscillation (`O1`)
  clustered plateau then jump (`K1`)
- `M1` shows that the model does not require `Q2/Q3` collapse to the same frame:
  `Q2` predicts and observes frame `6`, while `Q3` predicts and observes frame `4`

Validation
| Corpus | Shape | Q2 predicted | Q2 observed | Q2 match | Q2 classification | Q3 predicted | Q3 observed | Q3 match | Q3 classification |
| --- | --- | ---: | ---: | --- | --- | ---: | ---: | --- | --- |
| `P1_front_loaded_six` | permutation variant | 0 | 0 | exact | `persistent_offset` | 0 | 0 | exact | `persistent_offset` |
| `M1_monotonic_plateau_then_rise` | monotonic | 6 | 6 | exact | `rate_divergence` | 4 | 4 | exact | `rate_divergence` |
| `O1_alternating_low_high` | oscillatory | 1 | 1 | exact | `rate_divergence` | 1 | 1 | exact | `rate_divergence` |
| `K1_plateau_then_jump` | clustered/plateau | 6 | 6 | exact | `rate_divergence` | 6 | 6 | exact | `rate_divergence` |

Bounded validity
- For the tested host-only corpus family, using the checked-in
  `affine_transform -> accumulate -> clamp -> threshold` probe pipeline and
  integer samples drawn from the existing probe domain, the residue-index rule
  matches observed `first_divergence_frame` exactly for every checked `Q2/Q3`
  slice
- This WIP does not establish validity outside the tested host corpus family,
  outside `q in {2, 3}`, or for any changed pipeline semantics

Classification
- PASS-constrained

Next Decision
- Retain the rule as experiment-local and bounded to the tested host corpus
  family unless a future WIP needs a broader falsification search
- If broader coverage is required, the next useful stressor is a corpus family
  that intentionally exercises values whose affine residues differ between `Q2`
  and `Q3` while changing local ordering more aggressively

Promotion Path
experiment-local retention only

## 2026-03-29 — Boundary function modeling [WIP-009]
Status: closed (PASS)
Owner: signal

Problem
We need a minimal host-only rule that predicts the `Q2/Q3`
`first_divergence_frame` from corpus properties without changing artifact
generation, replay behavior, or classification logic.

Hypothesis
If the `Q2/Q3` boundary is corpus-dependent but shape-stable, the divergence
frame should correspond to a small corpus feature that captures where
quantization first loses information.

Constraints
- Host-only analysis
- No artifact contract change
- No replay semantic change
- No classification logic change
- No changelog update
- Keep implementation minimal and experiment-local

Canonical analysis
- corpora: `experiments/quantization_probe/corpus.txt` (`C1`),
  `experiments/quantization_probe/corpus_c2.txt` (`C2`)
- pipeline reference: `experiments/quantization_probe/generate_probe_artifact.py`
- host feature extraction:
  `python3 - <<'PY' ... load corpus, compute scalar stats, derivatives, affine-transform residues by quant_shift ... PY`
- known collapse-region targets:
  `C1 @ Q2/Q3 -> first_divergence_frame=4`
  `C2 @ Q2/Q3 -> first_divergence_frame=7`

Evidence Produced
- Aggregate scalar features are identical across `C1` and `C2`, so they do not
  explain the `4` vs `7` boundary shift

| Corpus | n | min | max | mean | variance | max \|Δ\| | avg Δ | avg \|Δ\| | Q2/Q3 first non-zero residue idx |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `C1` | 12 | 1 | 6 | `17/12` | `275/144` | 5 | `0/11` | `10/11` | 4 |
| `C2` | 12 | 1 | 6 | `17/12` | `275/144` | 5 | `0/11` | `10/11` | 7 |

- Affine-transformed sample stream from the checked-in pipeline is:
  `t_i = 5 * sample_i + 3`
- For `Q2/Q3`, the quantizer bucket is `2^q`, and the per-frame residue is:
  `r_i(q) = t_i mod 2^q`
- Observed `Q2/Q3` residues:
  `C1 -> [0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]`
  `C2 -> [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0]`
- The first non-zero residue index matches the known collapse-region boundary
  exactly for both corpora
- `Q4` is not part of the target fit, but the same residue rule explains the
  shape transition: because `2^4 = 16`, the common transformed value `8`
  already has non-zero residue, so the first non-zero residue index moves to
  frame `0`

Correlation
- `range`, `mean`, `variance`, `max |Δ|`, and average derivative magnitude have
  no explanatory power here because `C1` and `C2` are permutation-equivalent on
  those features
- The minimal explanatory variable is positional, not aggregate:
  the first frame whose transformed sample is not exactly preserved by the
  quantizer
- A derivative-position proxy also matches (`first index of |Δ| = 5`, plus one),
  but it does not encode `quant_shift` and is therefore less general than the
  residue rule

Proposed Model
- For the tested probe pipeline and collapse region `q in {2, 3}`:
  `predicted_first_divergence_frame(corpus, q) = min i such that ((5 * corpus[i] + 3) mod 2^q) != 0`
- Equivalent bit form:
  `predicted_first_divergence_frame(corpus, q) = min i such that ((5 * corpus[i] + 3) & ((1 << q) - 1)) != 0`
- If no such frame exists, the model is undefined for this WIP because no
  supporting corpus was tested

Validation
- `C1, Q2/Q3`: first non-zero residue at frame `4`; prediction `4`; observed `4`
- `C2, Q2/Q3`: first non-zero residue at frame `7`; prediction `7`; observed `7`
- Exact match on both available corpora

Classification
- PASS

Next Decision
- Keep the rule experiment-local unless more corpora are added
- Expand only if a future WIP needs to test whether the residue-index model
  survives additional corpus shapes

Promotion Path
experiment-local retention only

## 2026-03-26 — Witness-model v2 direction exploration [WIP-002]
Status: proposed
Owner: architecture

Problem
The current witness-model direction is not yet concrete enough to decide what artifact shape, proof boundary, or integration path should replace or extend the existing model.

Hypothesis
A narrower witness-model v2 can improve replay evidence clarity if it is scoped as an explicit artifact contract candidate before any normative migration.

Constraints
- Must not redefine release surface or replay invariants from `docs/spec/` or `docs/MATH_CONTRACT.md`
- Must remain compatible with existing verification authority in `VERIFICATION_GUIDE.md`

Planned Artifacts
- `docs/roadmap/witness_model_direction.md`
- candidate notes or prototypes referenced from future WIP entries

Evidence Produced
- existing direction note at `docs/roadmap/witness_model_direction.md`
- this append-only experiment seed for follow-on exploration

Next Decision
Decide whether witness-model v2 merits a concrete candidate artifact contract draft or should remain roadmap-only.

Promotion Path
`docs/spec/`, `docs/architecture/`, or retained release evidence, depending on what is validated.

## 2026-03-26 — BeagleBone Black hostile-board isolation bring-up [WIP-001]
Status: closed (PASS-constrained)
Owner: hardware

Problem
Board bring-up on a hostile or weakly controlled BeagleBone Black path needs a concise place to track isolation assumptions, failure modes, and whether the board is useful for capture work.

Hypothesis
Treating BeagleBone Black bring-up as a hostile-board experiment will clarify which isolation controls are required before any capture evidence from that path can be trusted.

Constraints
- Must not imply supported release hardware or verification authority
- Must not redefine current capture contracts outside the existing replay and verification docs

Planned Artifacts
- bring-up notes under `docs/wip/`
- retained command logs or observations linked from future WIP entries

Evidence Produced
- `docs/wip/bbb_bringup_evidence_BBB-001.md` (retained evidence record)
- `make gate`: VERIFICATION PASSED (on-target bit-exact validation)
- `make replay-tests`: PASS (after `python3-serial` dependency correction)

Next Decision
Onboarding complete for BBB-001. Proceed to experiment scaffold / workload definition.

Promotion Path
`docs/hardware/`, `docs/operations/`, `docs/replay/`, or release evidence if the path becomes validated.

## 2026-03-26 — BeagleBone Black prudent hostile-board bring-up [WIP-003]
Status: proposed
Owner: hardware

Problem
An unknown-source BeagleBone Black requires an auditable bring-up path that avoids trusting onboard state, avoids early networking, and produces retained evidence before any limited experimental use.

Hypothesis
Offline reimage to known-good microSD media, followed by isolated first boot and explicit surface reduction, yields an acceptable constrained experimental platform without implying trusted hardware.

Constraints
- Must not trust onboard eMMC state
- Must not use early network connectivity during initial isolation
- Must not create release-surface, specification, or verification authority claims

Planned Artifacts
- `docs/wip/bbb_prudent_bringup.md`
- `docs/wip/templates/bringup_evidence_checklist.md`

Evidence Produced
- WIP hostile-board bring-up procedure with T0-T5 gates
- compact retained-evidence checklist for operator use

Next Decision
Decide whether a specific board instance passes T5 for constrained experimental use, should remain on HOLD, or must be quarantined.

Promotion Path
`docs/hardware/` only after successful gated bring-up, retained evidence, and repeatability across later validation.

## 2026-03-27 — Quantization divergence witness experiment [WIP-004]
Status: closed (PASS-constrained)
Owner: signal
Phase 2 (matrix extension): appended host-only evidence; does not reopen phase 1.

Problem
We need a minimal, deterministic experiment that demonstrates divergence localization under quantization using the Precision signal/replay pipeline.

Hypothesis
A small fixed pipeline with baseline vs quantized paths and per-stage artifact emission will produce a reproducible first-divergence witness and classification.

Constraints
- Must be fully deterministic (no FP nondeterminism, no threading)
- Must use existing artifact + replay + diff tooling
- Must not modify release surface
- Must remain host-executable (BBB) without PRU

Planned Artifacts
- `experiments/quantization_probe/`
- baseline and quantized pipeline implementations
- fixed input corpus
- artifact outputs A/B
- diff output demonstrating first divergence

Evidence Produced
- BBB host Linux execution confirmed for the phase-1 witness path
- `make gate`: PASS on BBB host
- baseline artifact rerun hash: `67e309b08d7bf8db286869b2b81a23da297b7ccfd2ecd9e322830729e69a9e69`
- quantized artifact rerun hash: `fe992bec716077dc20eb94550d007022439fef871a1bf101a30727b2d18a8abf`
- first divergence: `frame_idx=4` (reported as `First divergence frame: 4`)
- `shape_class=persistent_offset`
- `primary_region=sample_payload`
- `evolution_class=bounded_persistent`
- controlled host matrix retained for `C1-Q2`, `C1-Q3`, `C1-Q4`, and `C2-Q3`
- matrix result summary:
  `C1-Q2` and `C1-Q3` stay at `frame_idx=4` with `shape_class=persistent_offset`
  `C1-Q4` moves first divergence to `frame_idx=0` with `shape_class=rate_divergence`
  `C2-Q3` moves first divergence to `frame_idx=7` with `shape_class=persistent_offset`
- retained experiment note, exact commands, and case hashes/classifications:
  `experiments/quantization_probe/README.md`

Next Decision
Rerun the same controlled matrix on BBB host if BBB-specific parity is required; otherwise keep the result experiment-local and avoid framework expansion.

Promotion Path
experiment-local retention only; reconsider `docs/replay/` or `docs/architecture/` only if a later phase produces broader validated evidence

## 2026-03-28 — Cross-surface quantization parity [WIP-006]
Status: closed (PASS-constrained)
Owner: signal

Problem
We need to verify whether the existing quantization divergence witness remains stable across the host and BBB execution surfaces without changing witness semantics or widening the artifact contract.

Hypothesis
For the same controlled probe input and exact command surface, host and BBB should preserve the same witness fields for the reduced parity matrix:
- `first_divergence_frame`
- `classification`
- `baseline_invariant`

Constraints
- No new witness semantics
- No artifact contract change
- No replay semantic change
- No taxonomy redesign
- Keep this WIP experimental and non-normative
- Prefer the smallest useful matrix

Canonical parity matrix
- corpus: `C1`
- baseline: `PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode baseline --corpus C1 --out /tmp/WIP006_C1_Q3_baseline_run{1,2}.rpl`
- quantized: `PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode quantized --corpus C1 --quant-shift 3 --out /tmp/WIP006_C1_Q3_quant_run{1,2}.rpl`
- repeatability checks: `cmp -s` on baseline pair and quantized pair
- hash check: `sha256sum /tmp/WIP006_C1_Q3_baseline_run1.rpl /tmp/WIP006_C1_Q3_baseline_run2.rpl /tmp/WIP006_C1_Q3_quant_run1.rpl /tmp/WIP006_C1_Q3_quant_run2.rpl`
- witness diff: `PYTHONPATH=. python3 scripts/artifact_diff.py /tmp/WIP006_C1_Q3_baseline_run1.rpl /tmp/WIP006_C1_Q3_quant_run1.rpl`
- required output fields: `first_divergence_frame`, `shape_class` as the classification source, and baseline repeat equality as `baseline_invariant`

Evidence Produced
- Host canonical matrix completed for `C1-Q3`
- host baseline repeat: PASS
- host quantized repeat: PASS
- host baseline hash: `67e309b08d7bf8db286869b2b81a23da297b7ccfd2ecd9e322830729e69a9e69`
- host quantized hash: `fe992bec716077dc20eb94550d007022439fef871a1bf101a30727b2d18a8abf`
- host witness fields:
  `first_divergence_frame=4`
  `classification=persistent_offset`
  `baseline_invariant=true`
- BBB baseline repeat: PASS
- BBB quantized repeat: PASS
- BBB baseline hash: `67e309b08d7bf8db286869b2b81a23da297b7ccfd2ecd9e322830729e69a9e69`
- BBB quantized hash: `fe992bec716077dc20eb94550d007022439fef871a1bf101a30727b2d18a8abf`
- BBB witness fields:
  `first_divergence_frame=4`
  `classification=persistent_offset`
  `baseline_invariant=true`
- Host and BBB matched exactly for the reduced canonical matrix

Classification
- PASS-constrained

MCU feasibility gate
- Deferred for this WIP
- Current MCU path is the STM32F446 capture firmware / demo feature lane, not the Python `quantization_probe` witness path
- Reusing the same witness cleanly on MCU would require opening a separate firmware-design lane, which is outside this WIP

Next Decision
- Expand matrix (Q2/Q3/Q4) only if broader parity is required
- MCU parity remains a separate firmware lane

Promotion Path
experiment-local retention only

## 2026-03-29 — Cross-corpus generalization probe [WIP-008]
Status: closed (PASS-constrained)
Owner: signal

Problem
We need to determine whether the quantization boundary observed in WIP-007 for `C1`
is structural or corpus-dependent by rerunning the same cross-surface matrix on `C2`.

Hypothesis
For the tested `C2` quantization domain, host and BBB should preserve identical
baseline hashes, quantized hashes, first-divergence frame, classification, and
baseline repeatability when the command surface varies only by `--quant-shift`.

Constraints
- No artifact contract change
- No replay semantic change
- No classification logic change
- No taxonomy redesign
- Keep this WIP experimental and non-normative
- Keep the matrix minimal and explicit

Canonical matrix
- corpus: `C2`
- baseline: `PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode baseline --corpus C2 --out /tmp/WIP008_C2_QX_baseline_run{1,2}.rpl`
- quantized: `PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode quantized --corpus C2 --quant-shift X --out /tmp/WIP008_C2_QX_quant_run{1,2}.rpl`
- repeatability checks: `cmp -s` on baseline pair and quantized pair
- hash check: `sha256sum /tmp/WIP008_C2_QX_baseline_run1.rpl /tmp/WIP008_C2_QX_quant_run1.rpl`
- witness diff: `PYTHONPATH=. python3 scripts/artifact_diff.py /tmp/WIP008_C2_QX_baseline_run1.rpl /tmp/WIP008_C2_QX_quant_run1.rpl`
- tested shifts: `X in {2,3,4}`

Evidence Produced
- Host matrix completed for `C2-Q2`, `C2-Q3`, and `C2-Q4`
- host `C2-Q2`: baseline repeat `PASS`, quantized repeat `PASS`, baseline hash `d6009946947b0e1b1ead89dac01112cda52bf116b711ec0728722e384f7e17d1`, quantized hash `a0fa69c57d1f9356e2fa3549d8c3233e8ee777730acecaba8a090ed0a2fe5724`, `first_divergence_frame=7`, `classification=persistent_offset`, `baseline_invariant=true`
- host `C2-Q3`: baseline repeat `PASS`, quantized repeat `PASS`, baseline hash `d6009946947b0e1b1ead89dac01112cda52bf116b711ec0728722e384f7e17d1`, quantized hash `a0fa69c57d1f9356e2fa3549d8c3233e8ee777730acecaba8a090ed0a2fe5724`, `first_divergence_frame=7`, `classification=persistent_offset`, `baseline_invariant=true`
- host `C2-Q4`: baseline repeat `PASS`, quantized repeat `PASS`, baseline hash `d6009946947b0e1b1ead89dac01112cda52bf116b711ec0728722e384f7e17d1`, quantized hash `f59e33f1bbaaa329fd58769fc51282c9619528284d5c32af718489439df04905`, `first_divergence_frame=0`, `classification=rate_divergence`, `baseline_invariant=true`
- BBB matrix completed for `C2-Q2`, `C2-Q3`, and `C2-Q4`
- BBB `C2-Q2`: baseline repeat `PASS`, quantized repeat `PASS`, baseline hash `d6009946947b0e1b1ead89dac01112cda52bf116b711ec0728722e384f7e17d1`, quantized hash `a0fa69c57d1f9356e2fa3549d8c3233e8ee777730acecaba8a090ed0a2fe5724`, `first_divergence_frame=7`, `classification=persistent_offset`, `baseline_invariant=true`
- BBB `C2-Q3`: baseline repeat `PASS`, quantized repeat `PASS`, baseline hash `d6009946947b0e1b1ead89dac01112cda52bf116b711ec0728722e384f7e17d1`, quantized hash `a0fa69c57d1f9356e2fa3549d8c3233e8ee777730acecaba8a090ed0a2fe5724`, `first_divergence_frame=7`, `classification=persistent_offset`, `baseline_invariant=true`
- BBB `C2-Q4`: baseline repeat `PASS`, quantized repeat `PASS`, baseline hash `d6009946947b0e1b1ead89dac01112cda52bf116b711ec0728722e384f7e17d1`, quantized hash `f59e33f1bbaaa329fd58769fc51282c9619528284d5c32af718489439df04905`, `first_divergence_frame=0`, `classification=rate_divergence`, `baseline_invariant=true`
- Host and BBB matched exactly for every completed `C2` shift in the WIP-008 matrix: identical baseline hash, quantized hash, `first_divergence_frame`, `classification`, and baseline repeatability result for `Q2`, `Q3`, and `Q4`
- Cross-corpus comparison against WIP-007 `C1`: the boundary shape is preserved because `Q2/Q3` collapse to the same witness and `Q4` changes witness class at frame `0`, but the boundary location differs by corpus because `C1-Q2/Q3` first diverge at frame `4` while `C2-Q2/Q3` first diverge at frame `7`

Boundary Classification
- No host/BBB mismatch observed across the tested `C2` quantization domain
- No failure boundary localized because no cross-surface divergence was observed
- The quantization boundary is constrained by corpus: shape parity holds across `C1` and `C2`, but the `Q2/Q3` first-divergence location shifts from frame `4` on `C1` to frame `7` on `C2`

Classification
- PASS-constrained

Next Decision
- Keep the result experiment-local unless broader parity beyond `C1/C2` is required
- Expand to additional corpora or surfaces only under a new WIP lane

Promotion Path
experiment-local retention only

## 2026-03-29 — WIP-007 matrix expansion and failure boundary mapping [WIP-007]
Status: closed (PASS)
Owner: signal

Problem
We need to determine whether the reduced WIP-006 cross-surface parity result for `C1`
is stable across the nearby quantization boundary (`Q2/Q3/Q4`) or only true at `Q3`.

Hypothesis
For the tested `C1` quantization domain, host and BBB should preserve identical
baseline hashes, quantized hashes, first-divergence frame, classification, and
baseline repeatability when the command surface varies only by `--quant-shift`.

Constraints
- No artifact contract change
- No replay semantic change
- No classification logic change
- No taxonomy redesign
- Keep this WIP experimental and non-normative
- Prefer evidence over interpretation

Canonical matrix
- corpus: `C1`
- baseline: `PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode baseline --corpus C1 --out /tmp/WIP007_C1_QX_baseline_run{1,2}.rpl`
- quantized: `PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode quantized --corpus C1 --quant-shift X --out /tmp/WIP007_C1_QX_quant_run{1,2}.rpl`
- repeatability checks: `cmp -s` on baseline pair and quantized pair
- hash check: `sha256sum /tmp/WIP007_C1_QX_baseline_run1.rpl /tmp/WIP007_C1_QX_quant_run1.rpl`
- witness diff: `PYTHONPATH=. python3 scripts/artifact_diff.py /tmp/WIP007_C1_QX_baseline_run1.rpl /tmp/WIP007_C1_QX_quant_run1.rpl`
- tested shifts: `X in {2,3,4}`

Evidence Produced
- Host matrix completed for `C1-Q2`, `C1-Q3`, and `C1-Q4`
- host `C1-Q2`: baseline repeat `PASS`, quantized repeat `PASS`, baseline hash `67e309b08d7bf8db286869b2b81a23da297b7ccfd2ecd9e322830729e69a9e69`, quantized hash `fe992bec716077dc20eb94550d007022439fef871a1bf101a30727b2d18a8abf`, `first_divergence_frame=4`, `classification=persistent_offset`, `baseline_invariant=true`
- host `C1-Q3`: baseline repeat `PASS`, quantized repeat `PASS`, baseline hash `67e309b08d7bf8db286869b2b81a23da297b7ccfd2ecd9e322830729e69a9e69`, quantized hash `fe992bec716077dc20eb94550d007022439fef871a1bf101a30727b2d18a8abf`, `first_divergence_frame=4`, `classification=persistent_offset`, `baseline_invariant=true`
- host `C1-Q4`: baseline repeat `PASS`, quantized repeat `PASS`, baseline hash `67e309b08d7bf8db286869b2b81a23da297b7ccfd2ecd9e322830729e69a9e69`, quantized hash `a1898d79ef3b55f8f60cdc4cb24467b25665f630a7fe0dc4a7a39318af228d83`, `first_divergence_frame=0`, `classification=rate_divergence`, `baseline_invariant=true`
- Host confirms the local boundary shape for `C1`: `Q2/Q3` collapse to the same witness, while `Q4` moves the first divergence to frame `0` and changes classification to `rate_divergence`
- BBB matrix completed for `C1-Q2`, `C1-Q3`, and `C1-Q4`
- BBB `C1-Q2`: baseline repeat `PASS`, quantized repeat `PASS`, baseline hash `67e309b08d7bf8db286869b2b81a23da297b7ccfd2ecd9e322830729e69a9e69`, quantized hash `fe992bec716077dc20eb94550d007022439fef871a1bf101a30727b2d18a8abf`, `first_divergence_frame=4`, `classification=persistent_offset`, `baseline_invariant=true`
- BBB `C1-Q3`: baseline repeat `PASS`, quantized repeat `PASS`, baseline hash `67e309b08d7bf8db286869b2b81a23da297b7ccfd2ecd9e322830729e69a9e69`, quantized hash `fe992bec716077dc20eb94550d007022439fef871a1bf101a30727b2d18a8abf`, `first_divergence_frame=4`, `classification=persistent_offset`, `baseline_invariant=true`
- BBB `C1-Q4`: baseline repeat `PASS`, quantized repeat `PASS`, baseline hash `67e309b08d7bf8db286869b2b81a23da297b7ccfd2ecd9e322830729e69a9e69`, quantized hash `a1898d79ef3b55f8f60cdc4cb24467b25665f630a7fe0dc4a7a39318af228d83`, `first_divergence_frame=0`, `classification=rate_divergence`, `baseline_invariant=true`
- Host and BBB matched exactly for every completed `C1` shift in the WIP-007 matrix: identical baseline hash, quantized hash, `first_divergence_frame`, `classification`, and baseline repeatability result for `Q2`, `Q3`, and `Q4`

Boundary Classification
- No host/BBB mismatch observed across the tested `C1` quantization domain
- No failure boundary localized because no cross-surface divergence was observed
- The local shift boundary for this corpus is still evidenced within each surface: `Q2/Q3` share one witness and `Q4` changes witness class at frame `0`

Classification
- PASS

Next Decision
- Keep the result experiment-local unless broader parity beyond `C1` is required
- Expand to additional corpora or surfaces only under a new WIP lane

Promotion Path
experiment-local retention only

## 2026-03-28 — Shared canonical layout constants [WIP-005]
Status: proposed

Problem
Probe and parser currently duplicate RPL0/EventFrame0 constants; current WIP-004 guard imports parser constants directly.

Hypothesis
A shared minimal constants module may remove duplication without changing runtime surfaces.

Constraints
- no change to artifact contract
- no replay semantic change
- no packaging expansion
- no normative promotion

Planned Artifacts
- one small shared constants module for RPL0/EventFrame0 layout values
- minimal follow-on updates in probe/parser Python consumers only if needed

Next Decision
Decide whether a shared constants module can remove duplication cleanly without adding runtime dependency edges or broadening implementation scope.

Promotion Path
scripts/ or shared internal module (non-normative)
