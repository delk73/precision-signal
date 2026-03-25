---
name: document-annotation-classification
description: "Use when classifying repository documents by role, authority, maturity relevance, release-surface relevance, and overlap risk so audit outputs use consistent document labels."
---

# Document Annotation Classification

## Purpose

Classify repository documents by role, authority, and maturity signaling, and standardize the annotation fields used by the other audit skills.

This skill analyzes document signaling only. It does not redefine repository maturity or release surface independently.

## Use When

- The audit needs a `Document Annotation Register`.
- You need to determine whether documents clearly signal normative status, descriptive scope, experimental scope, or roadmap intent.
- A repository mixes specs, architecture docs, WIP notes, and audit material without consistent labels.

## Shared Link Evaluation Contract

Use `doc-link-integrity` in `AUDIT` mode for all documentation-link evaluation performed by this skill.
Do not restate link-resolution rules here; defer all link-validity decisions to the shared skill.

## Bounded Method

1. Inspect document frontmatter, titles, intro sections, status labels, directory placement, and explicit wording.
2. Classify each document along these normalized axes:
   - `Document Type`: `normative` | `descriptive` | `roadmap` | `research` | `audit` | `unknown`
   - `Authority Level`: `canonical` | `supporting` | `duplicate` | `historical` | `unclear`
   - `Maturity Relevance`: `high` | `medium` | `low` | `none`
   - `Release-Surface Relevance`: `direct` | `indirect` | `none`
   - `Overlap Risk`: `high` | `medium` | `low`
3. Prefer explicit self-description first.
4. Historical audits and archived analysis docs must not be treated as current canonical routing documents.
5. A document may be high-value and still be duplicate from a routing perspective.
6. If a document’s stated role conflicts with stronger repository evidence, note the conflict in `Notes` without broadening this skill into claim-truth analysis.
7. Keep the classification evidence-first and conservative.
8. When a document annotation finding includes a documentation-reference defect, classify the reference as:
   - compliant
   - non-compliant / non-clickable
   - non-compliant / broken resolution
   - non-compliant / ambiguous target

## Required Output Contract

Produce a section titled `Document Annotation Register`.

For each document, emit:

- `Document`
- `Document Type`
- `Authority Level`
- `Maturity Relevance`
- `Release-Surface Relevance`
- `Overlap Risk`
- `Evidence`
- `Notes`

Preferred row shape:

```text
Document: <path>
Document Type: <normative|descriptive|roadmap|research|audit|unknown>
Authority Level: <canonical|supporting|duplicate|historical|unclear>
Maturity Relevance: <high|medium|low|none>
Release-Surface Relevance: <direct|indirect|none>
Overlap Risk: <high|medium|low>
Evidence:
- doc: <path>
- file: <path if implementation context is needed>
Notes: <short rationale; label inference where needed>
```

When a document annotation finding includes a link defect, also emit:

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
- Explicit labels in the document outrank directory-name inference and title tone.
- Directory placement may support, but not replace, document-level evidence.
- Do not infer authority level solely from directory placement.
- Do not treat a document as canonical solely because many other documents link to it.
- Directory placement may support classification but cannot establish authority without document-level evidence.
- Do not infer maturity relevance or release-surface relevance solely from directory placement or title tone.
- Acceptable evidence includes:
   - `doc: ...`
   - `file: ...`
   - `command: ...`
   - `command-output: ...`

Shared Conventions

- Label indirect conclusions as `inference`.
- If evidence cannot be located within reasonable search bounds, say `unknown`.
- Prefer executable or structural evidence over descriptive documentation.
- Do not speculate to fill gaps.

## Non-Goals

- Do not determine whether the repository actually implements the document’s claimed capability.
- Do not rewrite annotations or propose textual patches.
- Do not derive the repository release surface independently.
- Do not expand into full topology or claim-reality analysis except where needed to explain a signaling conflict.
