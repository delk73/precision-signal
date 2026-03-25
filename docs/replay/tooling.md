# Replay Tooling

This document defines the replay-tooling boundary.
Release status is classified in [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md).
Verification authority is defined in [VERIFICATION_GUIDE.md](../../VERIFICATION_GUIDE.md).
The normative artifact contract is [docs/spec/rpl0_artifact_contract.md](../spec/rpl0_artifact_contract.md).
Released replay-facing operator tooling is the Python toolchain.
Rust replay is present in the workspace and is not classified as released by
[docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md).
Its currently documented implementation scope uses RPL0 format version 0
replay, RPL0 format version 1 container parsing, and legacy 16-byte
`EventFrame0` replay semantics.

## Released Python Tooling

- [scripts/artifact_tool.py](../../scripts/artifact_tool.py): released operator CLI for capture, inspection,
  verification, hashing, and compare workflows
- [scripts/artifact_diff.py](../../scripts/artifact_diff.py): released divergence-localization tool for demo and
  diagnosis

## Experimental Rust Replay

- `replay-host` (experimental Rust replay engine with RPL0 format version 0
  replay, RPL0 format version 1 container parsing, and legacy 16-byte
  `EventFrame0` replay semantics)

### Experimental Rust Replay

```bash
cargo run -p replay-host -- diff artifacts/demo_v4/header_schema_B.rpl artifacts/demo_v4/header_schema_sample_payload_B.rpl
```

Current capabilities:

- RPL0 format version 0 replay
- RPL0 format version 1 container parsing
- legacy 16-byte `EventFrame0` replay semantics
- Rust replay is present in the workspace and is not classified as released by
  [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md)

Limitations:

- no schema-aware replay semantics are provided by the experimental Rust path
- Python replay tooling remains the released operator tooling

## Example Commands

```bash
python3 scripts/artifact_tool.py verify artifacts/run.bin --signal-model phase8
python3 scripts/artifact_tool.py hash artifacts/run.bin
python3 scripts/artifact_diff.py artifacts/demo/run_A.rpl artifacts/demo/run_B.rpl
```

## Validation Coverage

Parser validation suites:

- `scripts/test_artifact_parser_adversarial.py`
- `scripts/test_artifact_parser_valid_v1.py`
- `scripts/test_artifact_parser_mutation_corpus.py`

Tool regression suites:

- `scripts/test_artifact_tool_verify.py`
- `scripts/test_artifact_tool_hash.py`
- `scripts/test_artifact_diff.py`
- `scripts/test_demo_v3_fixtures.py`

Reference workflow documents:

- [docs/replay/RPL0_PARSER_TRACEABILITY.md](RPL0_PARSER_TRACEABILITY.md)
- [docs/replay/ARTIFACT_VALIDATION_WORKFLOW.md](ARTIFACT_VALIDATION_WORKFLOW.md)
- [docs/replay/CI_GATES.md](CI_GATES.md)

## Local Replay Validation Gates

```bash
make parser-tests
make replay-tool-tests
make replay-tests
```

These gates cover replay artifact handling, parser validity, and operator tool
behavior.
