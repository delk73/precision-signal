---
name: doc-link-integrity
description: "Use when auditing, repairing, or reviewing public documentation links so targets resolve correctly, labels stay reader-facing, and navigation remains clickable."
---

# Doc Link Integrity

## Purpose

Evaluate and optionally repair public documentation navigation so links are:

- clickable
- source-resolving
- reader-facing in label form
- mechanically consistent

This skill is the single shared authority for documentation-link evaluation in this repository.
Other agents and skills must delegate link findings to this skill instead of redefining separate link-normalization rules.

## Use When

- You need to audit public documentation links without editing files.
- You need to apply minimal mechanical fixes to broken or non-clickable documentation references.
- You need to review a documentation diff for link-target correctness and label hygiene.
- Another audit skill encounters documentation-link findings and needs a shared policy source.

## Core Invariants

1. A Markdown link target is valid only if it resolves correctly from the directory of the file that contains it.
2. A Markdown link label must be repo-facing reader text, not source-relative traversal text.
3. Where a public doc is clearly routing the reader to another doc, non-clickable prose references are defects.
4. Correctness outranks style. Do not flag a correct file-relative link merely because it is not written in `docs/...` form.

## Defect Classes

- `broken_target`
  - Markdown link exists.
  - Target does not resolve from the source file.
- `bad_label`
  - Label contains `../` or `./`.
  - Label exposes traversal instead of repo-facing path or name.
- `non_clickable_navigation`
  - Prose file or path reference is used for navigation without a Markdown link.
- `ambiguous_target`
  - Intended destination cannot be determined safely.
- `compliant`
  - Link resolves correctly and label is acceptable.

## Scope Rules

### In Scope

- `README.md`
- `docs/**/*.md`

### Out of Scope

- `docs/wip/**` unless linked from a public doc
- code comments
- code fences
- logs
- literal examples unless explicitly marked as active navigation text
- historical audit docs unless explicitly targeted

## Evaluation Rules

For each candidate reference:

1. Identify source file `F`.
2. Identify current label `L` and target `T` if the reference is Markdown-linked.
3. Determine whether the reference is:
   - Markdown link
   - prose navigation
   - literal or example text
4. If Markdown-linked, compute whether `T` resolves from `dirname(F)`.
5. If prose navigation, determine whether it is clearly intended as user-facing routing.
6. Code fences, logs, and literal example paths are not defects unless they are explicitly presented as active navigation.
7. Assign exactly one defect class.

## Mode Contract

This skill operates in three modes.

### AUDIT

Objective:
- Produce a defect register only. No edits.

Required output for each finding:
- source file
- quoted reference text
- defect class
- why it fails
- intended target if safely inferable
- corrected label and target form if mechanically inferable
- confidence

Audit decision rule:
- Sampled evidence must be labeled as sampled.
- Exhaustive claims require an exhaustive pass.
- Do not recommend style normalization that breaks resolution.

Deliverable:
- `Decision`: `pass` or `fail`
- `Findings`: grouped by defect class
- `High-Leverage Fix Set`: smallest file set with the biggest routing impact
- `Unknowns`: ambiguous references not safely resolvable

### EDIT

Objective:
- Apply minimal mechanical fixes using the same invariants as audit mode.

Allowed changes:
- fix broken Markdown targets
- normalize bad labels
- convert non-clickable navigation prose into Markdown links

Forbidden changes:
- rewriting content
- changing the document authority model
- path-style cleanup for aesthetics only
- converting literal example paths into links
- touching compliant links unnecessarily

Edit rule:
1. Compute the target from the source file location.
2. Preserve the correct target if only the label is bad.
3. Normalize the label to a repo-facing form.
4. Change only the smallest necessary span.

Deliverable:
- `Decision`: `pass` or `fail`
- `Files Modified`
- `Per-File Changes`
  - broken targets fixed
  - labels normalized
  - prose references converted
- `Skipped`: unresolved ambiguous targets

### REVIEW

Objective:
- Verify whether a proposed diff or completed patch satisfies the shared invariants.

Review checks:
- does each changed target resolve from the source file?
- does each label avoid traversal artifacts?
- was a correct link changed unnecessarily?
- was prose converted only where navigation intent is clear?
- did scope remain mechanical?

Deliverable:
- `Decision`: `approve` or `revise`
- `Correct`: invariant-satisfying changes
- `Defects`: exact remaining failures
- `Follow-up Scope`: smallest patch needed for approval

## Canonical Label Policy

Labels should be repo-facing.

Use the shortest repo-facing label that remains unambiguous in context.

Preferred examples:
- `VERIFICATION_GUIDE.md`
- `docs/RELEASE_SURFACE.md`
- `docs/replay/tooling.md`
- `docs/architecture/workspace.md`
- `docs/verification/releases/`

Disallowed examples:
- `../RELEASE_SURFACE.md`
- `../../VERIFICATION_GUIDE.md`
- `../replay/tooling.md`

Targets remain source-relative when that is what makes them resolve correctly.

## Script Link Policy

Public docs may link into `scripts/` when the script is itself the operator-facing tool or direct implementation evidence.
Prefer canonical prose docs over script links when a stable documentation target already exists for the same reader task.
Do not convert incidental script mentions or literal command examples into navigation links unless the document is clearly routing the reader there.

## Integration Rule

- `Repository Auditor` must use this skill for all documentation-link findings.
- `docs-consistency` must use this skill for audit, edit, and review passes.
- Other documentation audit skills may report link defects, but they must defer target-correctness and label-validity decisions to this skill.

## Handoff Prompt Template

You are operating the shared `doc-link-integrity` skill in `{MODE}` mode for the `precision-dpw` repository.

Your job is to evaluate and optionally repair public documentation navigation using these invariants:

1. A link target is valid only if it resolves correctly from the directory of the file that contains it.
2. A link label must be repo-facing reader text and must not expose traversal such as `../` or `./`.
3. Non-clickable prose references are defects when they are clearly intended as navigation.
4. Correctness outranks style. Do not flag or rewrite a correct file-relative link merely because it is not written in `docs/...` form.

Scope:

- `README.md`
- `docs/**/*.md`
- exclude `docs/wip/**` unless linked from a public doc
- ignore code fences, logs, and literal examples unless explicitly targeted

Mode behavior:

- `AUDIT`: produce a defect register only
- `EDIT`: apply minimal mechanical fixes only
- `REVIEW`: verify changed links against the same invariants

Defect classes:

- `broken_target`
- `bad_label`
- `non_clickable_navigation`
- `ambiguous_target`
- `compliant`

In `EDIT` mode:

- fix broken targets by computing source-relative paths
- normalize labels to repo-facing forms
- convert non-clickable navigation prose into Markdown links
- do not rewrite content beyond minimal link repair
- do not touch already-compliant links unnecessarily

In `REVIEW` mode:

- approve only if all changed links resolve correctly, labels are clean, and scope remained mechanical

Return:

- Decision
- Files Modified or Findings
- Per-file changes or per-finding defects
- Skipped or Unknowns
- Verification against invariants
