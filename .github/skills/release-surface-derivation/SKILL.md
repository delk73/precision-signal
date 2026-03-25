---
name: release-surface-derivation
description: "Use when deriving the actual implemented release surface from runnable binaries, verification workflows, specs, and documented user-facing paths without inflating roadmap items."
---

# Release Surface Derivation

## Purpose

Derive the actual implemented release surface from executable paths, verification workflows, and binding specifications.

This skill classifies capabilities conservatively. It keeps implemented existence, release classification, and path evidence separate; the repository auditor synthesizes final release conclusions.

## Use When

- The audit needs a `Derived Release Surface` section.
- You need to distinguish release capabilities from reference, experimental, roadmap, or unknown items.
- A repository describes many components but the implemented user-facing boundary is unclear.

## Shared Link Evaluation Contract

Use `doc-link-integrity` in `AUDIT` mode for all documentation-link evaluation performed by this skill.
Do not restate link-resolution rules here; defer all link-validity decisions to the shared skill.

## Bounded Method

1. Start from executable surfaces:
   - binaries
   - scripts with runnable entrypoints
   - Makefile targets
   - CI verification commands
2. Check whether each capability has at least one of:
   - an implemented documented runnable user-facing path
   - repository verification workflow coverage
   - a binding spec matched by implementation evidence
3. Cross-check specs under `docs/spec/` and normative docs only after locating implementation evidence.
4. Classify each capability using only these labels:
   - `Release`
   - `Reference`
   - `Experimental`
   - `Roadmap`
   - `Unknown`
5. Prevent roadmap inflation:
   - future-facing docs without implementation evidence stay `Roadmap`
   - implemented but unreleased or under-signposted surfaces may be `Reference`
   - missing evidence remains `Unknown`
6. Keep these fields separate for every capability:
   - implemented status
   - classification
   - user-facing path
   - verification path
   - evidence
   - notes
7. When a capability finding includes a documentation-reference defect, classify the reference as:
   - compliant
   - non-compliant / non-clickable
   - non-compliant / broken resolution
   - non-compliant / ambiguous target

## Required Output Contract

Produce a section titled `Derived Release Surface`.

For each capability, emit:

- `Capability`
- `Implemented`
- `Classification`
- `User-Facing Path`
- `Verification Path`
- `Evidence`
- `Notes`
- `Confidence`

Preferred row shape:

```text
Capability: <capability>
Implemented: <yes|no|unknown>
Classification: <Release|Reference|Experimental|Roadmap|Unknown>
User-Facing Path: <command or path or none|unknown>
Verification Path: <command or workflow or none|unknown>
Evidence:
- file: <path>
- symbol: <symbol if applicable>
- command: <command>
- command-output: <observed result summary if available>
- doc: <doc>
Notes: <short justification; label inference where needed>
Confidence: <direct|inference|unknown>
```

When a capability finding includes a link defect, also emit:

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

- Every nontrivial classification must cite at least one evidence source.
- Repository path references used for navigation in findings must be evaluated by whether they resolve correctly from the containing file; do not downgrade already-correct file-relative links for style alone.
- Do not classify a capability as `Release` unless at least one runnable user-facing path or one exercised verification path is cited, plus implementation evidence.
- A binding spec without matched implementation evidence cannot by itself justify `Release`.
- If classification depends on intent not proven by code, runnable command, or binding spec, downgrade to `Reference` or `Unknown`.
- Build presence in CI alone does not prove released operator-facing capability. CI-only evidence may justify `Implemented: yes`, but not necessarily `Release`.
- Prefer two independent evidence sources for `Release` where practical.
- Executable code and verification commands outrank descriptive documentation.
- Do not upgrade experimental code to `Release` because documentation is ambitious.
- Internal developer-only paths do not qualify as user-facing release evidence by themselves.
- Acceptable evidence includes:
   - `file: ...`
   - `symbol: ...`
   - `command: ...`
   - `command-output: <summary of observed behavior>`
   - `doc: ...`

Shared Conventions

- Use `direct` when implementation and path evidence directly support the classification, `inference` when the classification depends on limited interpretation, and `unknown` when evidence is insufficient.
- If evidence cannot be located within reasonable search bounds, say `unknown`.
- Prefer executable or structural evidence over descriptive documentation.
- Do not speculate to fill gaps.

## Non-Goals

- Do not audit wording quality across the entire documentation set.
- Do not enumerate every CLI subcommand unless needed to support a capability classification.
- Do not patch docs or code.
- Do not independently redefine repository maturity policy; classify from evidence only.
