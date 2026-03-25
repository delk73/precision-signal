---
name: naming-consistency-audit
description: "Use when auditing version-axis naming consistency, ambiguous v1 usage, artifact vs release vs document version mixing, incorrect capability statements like supports v1, filename inconsistencies such as _v1.md, or missing versioning terminology blocks in canonical specs."
---

# Naming Consistency Audit

## Purpose

Detect version-axis violations across the repository and normalize how audit outputs describe naming and terminology defects.

This skill is for terminology and naming audits only. It does not determine overall release classification by itself.

## Use When

- The audit needs a `Terminology Consistency Register`.
- Versioned terms such as `v1`, `artifact v1`, `replay v1`, `supports v1`, or `_v1.md` may be used ambiguously.
- Repository documents may be mixing format version, release version, and document revision language.
- Canonical specs may be missing an explicit terminology block that defines their version axis.

## Shared Link Evaluation Contract

Use `doc-link-integrity` in `AUDIT` mode for all documentation-link evaluation performed by this skill.
Do not restate link-resolution rules here; defer all link-validity decisions to the shared skill.

## Version-Axis Model

Use these terms distinctly:

- `format version`: version of a wire format, artifact format, protocol, or schema contract
- `release version`: shipped repository or product release identifier
- `document revision`: revision of a document itself
- `capability statement`: claim about implemented support or behavior

Do not collapse these axes into a bare `vX` phrase unless the local context defines the axis unambiguously.

## Enforced Rules

| Rule ID | Condition | Severity |
|---|---|---|
| NAM-001 | `artifact vX` used | `error` |
| NAM-002 | `replay vX` used | `error` |
| NAM-003 | `_vX.md` used for artifact spec filename | `error` |
| NAM-004 | ambiguous bare `vX` usage | `warn` |
| NAM-005 | missing versioning terminology block in canonical specs | `warn` |
| NAM-006 | capability statement uses version label instead of explicit format/capability language | `error` |

## Classification Labels

Use one classification per row:

- `ambiguous-version`
- `axis-mixing`
- `artifact-spec-filename`
- `capability-statement`
- `missing-terminology-block`

## Bounded Method

1. Start with canonical routing and normative documents:
   - root README
   - docs index pages
   - release-surface docs
   - files under `docs/spec/`
   - other normative contracts such as `docs/MATH_CONTRACT.md` and `VERIFICATION_GUIDE.md`
2. Search for versioned phrases and filename patterns first:
   - `artifact v[0-9]+`
   - `replay v[0-9]+`
   - `supports v[0-9]+`
   - bare `v[0-9]+`
   - `_v[0-9]+.md`
3. For each match, determine which version axis is actually intended:
   - format version
   - release version
   - document revision
   - unknown
4. Flag any statement that collapses the axis into a shorthand phrase when the local sentence could be read more than one way.
5. For capability statements, distinguish:
   - `supports format v1 artifacts`
   - `implements the v1 parser`
   - `supports v1`
   The third form is insufficient and should be treated as a violation when it describes capability.
6. For canonical specs that define a versioned contract, check whether they include a short terminology block that states which version axis the document uses.
7. Do not infer that `_v1.md` is wrong for every file. Apply NAM-003 specifically to artifact spec filenames where the filename itself is standing in for the format identifier rather than a document revision label.
8. Keep findings structural and evidence-based. Do not rewrite the text during the audit.
9. When a naming finding includes a documentation-reference defect, classify the reference as:
   - compliant
   - non-compliant / non-clickable
   - non-compliant / broken resolution
   - non-compliant / ambiguous target

## Canonical Spec Terminology Block Check

For NAM-005, treat a canonical spec as compliant when it contains an explicit section or short introductory block that distinguishes at least the applicable axes, such as:

- what `v1` means in this document
- whether it refers to format version, release version, or document revision
- whether the document is normative for that versioned contract

Equivalent wording is acceptable. The block does not need a fixed heading, but it must remove axis ambiguity.

## Required Output Contract

Produce a section titled `Terminology Consistency Register`.

Emit a structured table with these columns:

- `Rule ID`
- `File`
- `Line`
- `Classification`
- `Severity`
- `Evidence`
- `Notes`

Preferred row shape:

```text
| Rule ID | File | Line | Classification | Severity | Evidence | Notes |
|---|---|---:|---|---|---|---|
| NAM-004 | docs/replay/README.md | 29 | ambiguous-version | warn | `normative RPL0 artifact contract v1` | Bare `v1` appears without explicitly naming the axis in surrounding prose. |
```

`File` must be a repository-relative path.

`Line` must be a single 1-based line number for the strongest evidence location.

When a naming finding includes a link defect, also emit a supplemental block after the row with:

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

- Every row must cite direct textual evidence from the file.
- Repository path references used for navigation in findings must be evaluated by whether they resolve correctly from the containing file; do not downgrade already-correct file-relative links for style alone.
- Prefer canonical and public-facing docs first, then supporting docs, then internal docs.
- A filename-only finding is allowed for NAM-003 if the filename is itself the evidence.
- Use `warn` only for ambiguity or missing terminology blocks.
- Use `error` for collapsed capability claims, explicit axis-mixing phrases, or artifact spec filename violations.
- If local context fully disambiguates a bare `vX`, do not emit NAM-004.
- If a capability statement is clearly scoped, such as `supports RPL0 format version 1 parsing`, do not emit NAM-006.
- If evidence is insufficient to determine the intended axis, emit NAM-004 rather than inventing a stronger violation.

## Non-Goals

- Do not classify overall release maturity.
- Do not determine implementation truth beyond identifying wording defects in capability statements.
- Do not rewrite filenames or document text during the audit.
- Do not broaden the audit into full claim-reality analysis unless another skill requires it.