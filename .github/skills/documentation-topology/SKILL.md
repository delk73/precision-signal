---
name: documentation-topology
description: "Use when mapping documentation structure, canonical reading paths, overlaps, duplicated explanations, and normative-versus-descriptive document routing for repository audits."
---

# Documentation Topology

## Purpose

Map documentation structure and identify overlaps, duplication, and reading-path issues.

This skill is structural. It maps documents and roles; it does not decide whether claims are true unless required to categorize purpose.

## Use When

- The audit needs a `Documentation Topology Map`.
- The repository has many docs and the canonical reading path is unclear.
- You need to identify duplicated architecture or release-surface explanations.

## Shared Link Evaluation Contract

Use `doc-link-integrity` in `AUDIT` mode for all documentation-link evaluation performed by this skill.
Do not restate link-resolution rules here; defer all link-validity decisions to the shared skill.

## Bounded Method

1. Inspect top-level documentation directories and high-level index pages.
2. Identify canonical routing documents, normative documents, descriptive documents, audits, and research or WIP areas.
3. Identify the canonical reading path through the documentation set and flag where multiple documents compete for the same routing role.
4. Map each document to a primary purpose.
5. Note overlap and overlap severity where multiple docs compete for:
   - entry routing
   - release-surface explanation
   - architecture overview
6. Distinguish structure from correctness:
   - this skill identifies overlap and routing issues
   - it does not decide claim truth unless needed to classify category
7. When evaluating documentation references, classify each reference as:
   - compliant
   - non-compliant / non-clickable
   - non-compliant / broken resolution
   - non-compliant / ambiguous target

## Required Output Contract

Produce a section titled `Documentation Topology Map`.

For each important document or document group, emit:

- `Document`
- `Purpose`
- `Category`
- `Public or Internal`
- `Overlap`
- `Overlap Severity`
- `Authority Level`
- `Notes`

Preferred row shape:

```text
Document: <path>
Purpose: <primary purpose>
Category: <entry|index|release-classification|normative|deep-architecture|workflow|historical-audit|reference|unknown>
Public or Internal: <public|internal|unknown>
Overlap: <none|entry-routing|release-surface-explanation|architecture-overview|mixed>
Overlap Severity: <minor|material|primary|unknown>
Authority Level: <canonical|supporting|duplicate|unclear>
Notes: <routing role, competing documents, or reading-path issue>
```

When a topology finding includes a link defect, also emit:

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

- Every nontrivial topology item must cite at least one evidence source.
- Use document headers, intro sections, and directory placement as structural evidence.
- Repository path references used for navigation in findings must be evaluated by whether they resolve correctly from the containing file; do not downgrade already-correct file-relative links for style alone.
- Topology findings should describe routing overlap and authority collisions, not re-litigate implementation truth unless needed for context.
- If multiple public documents explain the same maturity boundary, flag that as topology overlap even if the wording is individually correct.
- If overlap or overlap severity is inferred rather than explicitly stated, label it `inference`.
- If a document’s role cannot be determined confidently, say `unknown`.
- Do not assume a document is canonical just because it is long.
- Acceptable evidence includes:
   - `doc: ...`
   - `file: ...`
   - `command: ...`
   - `command-output: ...`

Shared Conventions

- Label indirect conclusions as `inference`.
- If evidence cannot be located within reasonable search bounds, say `unknown`.
- Prefer document-local structural evidence over broader repository interpretation.
- Do not speculate to fill gaps.

## Non-Goals

- Do not evaluate whether document claims are implemented unless needed for category disambiguation.
- Do not classify release maturity of capabilities.
- Do not mutate documentation.
- Do not score the repository.
