---
name: cli-surface-discovery
description: "Use when auditing runnable entrypoints, command trees, subcommands, script CLIs, Cargo bins, or operator-facing command surfaces from implementation evidence."
---

# CLI Surface Discovery

## Purpose

Identify runnable CLI entrypoints, commands, and subcommands implemented in the repository.

This skill is limited to CLI surface discovery. It inventories command surfaces; it does not decide the repository release surface on its own.

## Use When

- The audit needs a CLI Surface Inventory.
- You need to identify implemented command trees before classifying provisional surface labels.
- You need to inspect Cargo binaries, `src/bin`, Python tooling, script entrypoints, firmware-facing entrypoints, or documented operator commands.

## Shared Link Evaluation Contract

Use `doc-link-integrity` in `AUDIT` mode for all documentation-link evaluation performed by this skill.
Do not restate link-resolution rules here; defer all link-validity decisions to the shared skill.

## Bounded Method

1. Start from structural evidence:
   - Cargo manifests
   - `src/bin/`
   - `fn main(...)` entrypoints
   - CLI frameworks such as `clap`, `argh`, or `structopt`
   - script parsers such as `argparse`
   - Python tool entrypoints under repository scripts or operator-tool directories
2. Enumerate command roots first.
3. Expand subcommands only where implementation evidence exists.
4. Check whether documented command entrypoints correspond to implemented code paths.
5. Record runnable status separately from classification.
6. Assign classification conservatively as a provisional surface label, not a final release-surface judgment:
   - `Release`
   - `Reference`
   - `Experimental`
   - `Unknown`
7. If classification depends on broader release-surface reasoning, label uncertain cases `Unknown` and defer synthesis to the repository auditor.
8. When an inventory finding includes a documentation-reference defect, classify the reference as:
   - compliant
   - non-compliant / non-clickable
   - non-compliant / broken resolution
   - non-compliant / ambiguous target

## Required Output Contract

Produce a section titled `CLI Surface Inventory`.

For each discovered surface, emit:

- `Command`
- `Entrypoint File`
- `Command Type`
- `Observed Runnable Status`
- `Verification Path`
- `Classification`
- `Evidence`
- `Notes`

Preferred row shape:

```text
Command: <command>
Entrypoint File: <path>
Command Type: <rust-bin|python-tool|script|firmware-entry|other>
Observed Runnable Status: <observed|not-observed|unknown>
Verification Path: <command or workflow or none|unknown>
Classification: <Release|Reference|Experimental|Unknown>
Evidence:
- file: <path>
- symbol: <symbol if applicable>
- command: <invocation if available>
- command-output: <observed summary if available>
- doc: <doc if applicable>
Notes: <implemented behavior and any caveats>
```

When an inventory finding includes a link defect, also emit:

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

- Every nontrivial inventory item must cite at least one evidence source.
- Repository path references used for navigation in findings must be evaluated by whether they resolve correctly from the containing file; do not downgrade already-correct file-relative links for style alone.
- Inventory existence is not the same as release classification.
- Do not invent subcommands or operator surfaces from prose alone.
- Do not classify a CLI surface as `Release` solely because it is implemented.
- A CLI surface may only be classified as `Release` if the repository explicitly presents it as a user-facing or operator-facing capability.
- Python operator tooling must be inventoried alongside Rust CLIs when they are user-visible executable surfaces.
- Where possible, cite the exact executable entrypoint, command invocation, and observed output summary.
- If a command is present in code but was not exercised and no verification path was found, mark runnable status `unknown`.
- Otherwise classify conservatively or mark `Unknown`.
- Acceptable evidence includes:
   - `file: ...`
   - `symbol: ...`
   - `command: ...`
   - `command-output: ...`
   - `doc: ...`

Shared Conventions

- Label indirect conclusions as `inference`.
- If evidence cannot be located within reasonable search bounds, say `unknown`.
- Prefer executable or structural evidence over descriptive documentation.
- Do not speculate to fill gaps.

## Non-Goals

- Do not redefine the repository release surface.
- Do not score repository quality.
- Do not patch files or change repository state.
- Do not perform claim-versus-reality judgment beyond noting whether a documented CLI path is implemented.
