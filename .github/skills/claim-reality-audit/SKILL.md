---
name: claim-reality-audit
description: "Use when comparing repository claims in README or architecture docs against implementation and verification evidence to produce a claim-reality gap register."
---

# Claim-Reality Audit

## Purpose

Compare repository capability claims to implementation and verification evidence.

This skill focuses on claim extraction and alignment checking. It does not rewrite claims and does not score the repository.

## Use When

- The audit needs a `Claim-Reality Gap Register`.
- README, architecture, or release-surface docs may overstate or loosely describe implemented capabilities.
- You need to determine whether a claim is exact, partial, contradicted, roadmap, duplicated, or unknown.

## Shared Link Evaluation Contract

Use `doc-link-integrity` in `AUDIT` mode for all documentation-link evaluation performed by this skill.
Do not restate link-resolution rules here; defer all link-validity decisions to the shared skill.

## Bounded Method

1. Inspect claim-heavy documents first:
   - root README
   - docs index pages
   - release-surface pages
   - architecture or system-disclosure docs
2. Extract concrete capability claims, not general aspirations.
3. Locate supporting implementation or verification evidence for each claim.
4. Compare each claim against the strongest available evidence in this order:
   - implementation
   - command or verification output
   - binding spec
   - descriptive docs
5. Assign one status per claim:
   - `exact`
   - `partial`
   - `contradicted`
   - `roadmap`
   - `duplicated`
   - `unknown`
6. Assign one confidence label per claim:
   - `direct`
   - `inference`
   - `unknown`
7. Keep notes short and evidence-based.
8. When a claim finding includes a documentation-reference defect, classify the reference as:
   - compliant
   - non-compliant / non-clickable
   - non-compliant / broken resolution
   - non-compliant / ambiguous target

## Required Output Contract

Produce a section titled `Claim-Reality Gap Register`.

For each claim, emit:

- `Claim`
- `Supporting Evidence`
- `Status`
- `Impact`
- `Confidence`
- `Notes`

Allowed values for `Impact`:

- `credibility`
- `onboarding`
- `release-risk`
- `maintainability`

Preferred row shape:

```text
Claim: <quoted or tightly paraphrased claim>
Supporting Evidence:
- file: <path>
- symbol: <symbol>
- command-output: <result summary>
Status: <exact|partial|contradicted|roadmap|duplicated|unknown>
Impact: <credibility|onboarding|release-risk|maintainability>
Confidence: <direct|inference|unknown>
Notes: <short explanation; label inference where needed>
```

When a claim finding includes a link defect, also emit:

- `Source File`
- `Cited Text`
- `Current Link Form`
- `Expected Target`
- `Link Status`
- `Failure Type`

Allowed values:

- `Link Status`: `compliant` | `non-compliant`
- `Failure Type`: `non-clickable` | `broken resolution` | `ambiguous target`

## Evidence Rules

- Every nontrivial gap item must cite at least one evidence source.
- Repository path references used for navigation in findings must be evaluated by whether they resolve correctly from the containing file; do not downgrade already-correct file-relative links for style alone.
- Claims without supporting implementation or verification evidence must not be treated as true by default.
- Prefer evidence in this order:
   1. executable code
   2. command or verification output
   3. binding spec
   4. descriptive docs
- If a claim can only be supported by descriptive documentation and not by implementation or verification evidence, it must not be marked `exact` unless the claim is purely descriptive about documentation structure.
- Bare `doc:` evidence must not be the sole support for an implementation claim unless the claim itself is about documentation structure or routing.
- Use `partial` when some implementation exists but the wording overreaches or collapses multiple maturity levels into one phrase.
- Use `contradicted` when implementation or verification evidence conflicts with the claim.
- Replace uncertainty with `unknown`; do not smooth unsupported cases into softer wording.
- Acceptable evidence includes:
   - `file: ...`
   - `symbol: ...`
   - `command: ...`
   - `command-output: ...`
   - `doc: ...`

Shared Conventions

- Use `direct` when the evidence directly supports the status, `inference` when the conclusion depends on constrained interpretation, and `unknown` when evidence is missing.
- If evidence cannot be located within reasonable search bounds, say `unknown`.
- Prefer executable or structural evidence over descriptive documentation.
- Do not speculate to fill gaps.

## Non-Goals

- Do not derive the full release surface from scratch unless needed to evaluate a claim.
- Do not rewrite or normalize repository terminology.
- Do not produce remediation patches.
- Do not broaden the audit into documentation topology mapping.
