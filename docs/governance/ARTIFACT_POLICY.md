# Purpose

- Define which artifacts may be retained in this repository and which artifacts are transient.
- Reduce contributor ambiguity without changing the release contract, retained-bundle authority, or current repository contents.
- Defer release evidence requirements to `VERIFICATION_GUIDE.md` and retained bundle contents under `docs/verification/releases/`.

# Definitions

- Artifact: a generated file or generated directory produced by verification, replay, packaging, capture, audit, documentation, or release workflows.
- Retained artifact: an artifact intentionally committed because it is part of a canonical retained release bundle, a required support record, or an already-established public repository record.
- Transient artifact: an artifact generated for local development, intermediate workflow state, ad hoc investigation, or repeatable regeneration that is not required to remain in the repository.
- Release bundle: a versioned retained record under `docs/verification/releases/<version>/`.
- Support record: retained material that supports interpretation of a release bundle or authoritative document without expanding release scope.

# Retained Artifacts (In-Repo)

- Artifacts retained under `docs/verification/releases/<version>/` when required by the release contract and the specific retained bundle.
- Supporting retained files that are already part of the documented release-record layout for a versioned bundle.
- Public documentation artifacts intentionally kept for repository reading paths, evidence routing, or historical reference when already classified by existing authority documents.
- Historical retained artifacts that are already committed and routed by the repository’s existing documentation structure.
- Required support artifacts must stay bounded to their documented role and must not be described as broader release authority.

# Non-Retained Artifacts (Out-of-Scope)

- Local build outputs, temporary outputs, caches, logs, scratch files, and ad hoc investigation outputs.
- Generated artifacts under `target/`, local run directories under `artifacts/`, and other reproducible workflow outputs unless an authority document explicitly requires their retention.
- Duplicate copies of retained evidence stored outside the documented retained location.
- Intermediate packaging outputs, one-off captures, exploratory replay outputs, and manually collected local evidence not called for by a retained bundle.
- Any generated artifact whose repository role is unclear.

# Retention Rules

- Retain artifacts only when an existing authority document or retained bundle explicitly requires them to be in-repo.
- Limit retained artifacts to canonical release bundles and required bounded support material.
- Do not commit transient artifacts.
- Do not copy forward artifacts from one release bundle into another unless the active task explicitly requires it and the authority documents support it.
- Do not broaden release meaning by retaining extra artifacts without explicit authority.
- If an artifact can be regenerated and is not required as retained evidence or documented support material, treat it as transient.
- If authority and repository placement disagree, defer to `VERIFICATION_GUIDE.md`, then `docs/verification/releases/`, then release-classification routing in `docs/RELEASE_SURFACE.md`.

# Contributor Responsibilities

- Classify generated artifacts before committing them.
- Commit retained artifacts only to their documented repository location.
- Leave transient artifacts out of commits.
- Keep support artifacts explicitly bounded in wording and placement.
- When uncertain whether an artifact belongs in-repo, choose the narrower interpretation and do not commit it until the governing authority is clear.
- Avoid bundling artifact retention changes with unrelated code or documentation edits unless the task explicitly couples them.

# Non-Goals

- This document does not change current repository contents.
- This document does not create new enforcement, automation, CI behavior, or storage mechanisms.
- This document does not redefine release evidence requirements.
- This document does not expand the release surface or change release classification.
- This document does not replace `VERIFICATION_GUIDE.md`, `docs/RELEASE_SURFACE.md`, or versioned retained release bundles.
