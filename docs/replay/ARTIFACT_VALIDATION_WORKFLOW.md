# RPL Validation Workflow

This document defines the operator-facing validation path for the RPL format subsystem.
Scope: parser structure checks, canonical hash behavior, and RPL file comparison behavior.

## What Is Validated

- Parser structure and boundary checks (`RPL0` magic, version dispatch, header/schema/frame bounds).
- Canonical hashing behavior for the RPL prefix (`[header][schema][frame_data]`).
- RPL file comparison behavior (identical files, header mismatch, first frame divergence).
- Strict vs hash trailing-byte policy:
  - `verify` is strict and rejects trailing bytes.
  - `hash` may ignore trailing bytes and hashes only the canonical RPL prefix.

## Local Validation Commands

- Parser hardening suites:
  - `make parser-tests`
- Toolchain CLI regression suites:
  - `make replay-tool-tests`
- Full RPL/toolchain local gate:
  - `make replay-tests`

Reference direct commands:
- `python3 scripts/test_artifact_parser_adversarial.py`
- `python3 scripts/test_artifact_parser_valid_v1.py`
- `python3 scripts/test_artifact_parser_mutation_corpus.py`
- `python3 scripts/test_artifact_tool_verify.py`
- `python3 scripts/test_artifact_tool_hash.py`
- `python3 scripts/test_compare_artifact.py`

## CI Gates

RPL/toolchain coverage in CI is enforced by:

- `Replay Baseline Verify (phase8)`
  - `python3 scripts/artifact_tool.py verify artifacts/baseline.bin --signal-model phase8`
  - `python3 scripts/artifact_tool.py hash artifacts/baseline.bin --expect <pinned-sha256>`
- `Replay Artifact Parser Adversarial Tests`
- `Replay Artifact Parser Valid v1 Coverage`
- `Replay Artifact Parser Mutation Corpus`
- `Replay Artifact Tool Verify CLI Regression`
- `Replay Artifact Tool Hash CLI Regression`
- `Replay Artifact Compare CLI Regression`
- `Replay Baseline Metadata Consistency`

## Intended Operator Workflow

Use this sequence when validating a captured RPL file:

1. Verify structural validity (strict):
   - `python3 scripts/artifact_tool.py verify <artifact.bin> --signal-model phase8`
2. Compute canonical identity hash:
   - `python3 scripts/artifact_tool.py hash <artifact.bin>`
3. Compare against known baseline:
   - `python3 scripts/compare_artifact.py artifacts/baseline.bin <artifact.bin>`

For local pre-commit replay validation, run:

- `make replay-tests`

## Stability Status

RPL/toolchain hardening is closed for this sprint:

- Parser traceability matrix is in place ([docs/replay/RPL0_PARSER_TRACEABILITY.md](RPL0_PARSER_TRACEABILITY.md)).
- Parser coverage includes adversarial, acceptance-space, and mutation boundary suites.
- Toolchain coverage includes verify/hash/compare CLI regression suites.
- Local and CI replay validation entry points are wired and deterministic.
