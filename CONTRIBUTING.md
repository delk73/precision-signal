# Purpose

- Contribute changes without violating the release contract, release classification, determinism guarantees, or retained-evidence rules.

# Authority Hierarchy

- Precedence: `VERIFICATION_GUIDE.md` > `docs/RELEASE_SURFACE.md` > retained release records > this file
- Treat `VERIFICATION_GUIDE.md` as the canonical release contract.
- Treat `docs/RELEASE_SURFACE.md` as the release-classification anchor.
- Treat retained evidence under `docs/verification/releases/` as release-scoped record material.
- Do not let this file override those authorities.

# Required Commands

- Run `make gate` as the canonical validation step for contributor changes.
- Do not substitute other commands for `make gate` when validating release correctness.
- Run narrower local checks only as support for development, not as a substitute for `make gate`.
- Do not claim release readiness from ad hoc commands when `make gate` has not passed.

# Release Surface Discipline

- Do not expand the release surface implicitly through wording, routing, examples, bundle contents, or tool references.
- Keep support, reference, historical, and experimental material explicitly bounded when editing release-adjacent docs.
- Do not describe non-canonical tooling as part of the active operator surface unless the existing authority documents already do so.
- When a change could broaden release meaning, stop and align it to the existing narrowed contract.

# Evidence Rules

- Retain evidence only where the existing repository workflow and retained-record layout already place it.
- Do not copy forward evidence from older releases into a new retained bundle.
- Do not regenerate or modify retained evidence unless explicitly required by the task and validated against the authority documents.
- Do not present support evidence as release authority.

# PR Scope Rules

- Submit narrowly scoped PRs.
- Change only the files needed for the stated task.
- Separate release-surface, evidence-layout, code, and broader documentation work unless the task explicitly couples them.
- Do not bundle cleanup, refactors, or wording churn into a targeted repair.

# Fail-Closed Principle

- If wording, evidence placement, or tool classification is uncertain, choose the narrower interpretation.
- If a document could be read as broadening the active release claim, rewrite it or stop.
- If a retained bundle says more than the retained record proves, narrow it.
- If uncertain which document controls, defer to `VERIFICATION_GUIDE.md`, then `docs/RELEASE_SURFACE.md`, then the retained release record.
