# Replay Tooling

This document is support/reference guidance for replay tooling and local replay
validation. It is not active CLI authority and it does not define the active
operator surface.

Use:

- [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md) for release classification
- [docs/authority/cli_contract.md](../authority/cli_contract.md) for the sole
  active CLI contract
- [docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md](INTERVAL_CAPTURE_CONTRACT_v1.md)
  for the sole active STM32 capture contract
- [docs/verification/releases/index.md](../verification/releases/index.md) for
  historical release records and retained verification material

Terminology used in this document:

- `artifact` refers to the authoritative published `precision` provenance
  artifact directory
- `RPL` refers to the portable replay binary format, including `.rpl` files,
  parser behavior, and portable replay validation

Replay uses a two-stage model: Stage 1 is the interval CSV capture contract for
firmware, defined in
[docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md](INTERVAL_CAPTURE_CONTRACT_v1.md).
Stage 2 is the RPL0 portable replay format for replay and diff, defined in
[docs/spec/rpl0_format_contract.md](../spec/rpl0_format_contract.md). These
are separate contracts, not one unified format.

The Python tooling layer (`artifact_tool.py`, `artifact_diff.py`) is retained as
historical support/reference tooling. Rust replay remains mostly experimental,
with one bounded historical `replay-host diff` slice retained under
[docs/verification/releases/1.5.0/RUST_REPLAY_DIFF_SCOPE.md](../verification/releases/1.5.0/RUST_REPLAY_DIFF_SCOPE.md).

## Historically Released Python Tooling

- [scripts/artifact_tool.py](../../scripts/artifact_tool.py): historically
  released replay-facing support tool for capture, inspection, verification,
  hashing, and compare workflows; retained as support/reference tooling
- [scripts/artifact_diff.py](../../scripts/artifact_diff.py): historically
  released replay-facing divergence-localization support tool for demo and
  diagnosis; retained as support/reference tooling

## Historical Rust Replay Slice

- bounded `replay-host diff`

### Historical `replay-host diff`

```bash
cargo run -q -p replay-host -- diff artifacts/rpl0/run_20260331T150000Z.import1.rpl artifacts/rpl0/run_20260331T150000Z.frame017_plus1.rpl
```

Retained scope:

- the retained `artifacts/rpl0/` proof corpus only
- the exact accepted RPL input class mechanically demonstrated by that corpus and
  retained under [docs/verification/releases/1.5.0/RUST_REPLAY_DIFF_SCOPE.md](../verification/releases/1.5.0/RUST_REPLAY_DIFF_SCOPE.md)
- current replay behavior under the existing RPL0/EventFrame0 implementation
  boundary already present in code

## Experimental Rust Replay

- `replay-host import-interval-csv`
- `replay-host validate-interval-csv`
- any broader `replay-host` capability outside the retained `artifacts/rpl0/`
  proof corpus and accepted RPL input class
- schema-aware Rust replay semantics
- `replay-fw-f446`

Current implementation boundary behind the released `diff` slice:

- RPL0 format version 0 replay
- RPL0 format version 1 container parsing
- legacy 16-byte `EventFrame0` replay semantics

Limitations:

- the `1.5.0` release does not generalize `replay-host diff` beyond the
  retained `artifacts/rpl0/` proof corpus
- no schema-aware replay semantics are provided by the Rust path
- the Python tooling layer remains historical support/reference tooling and is
  not part of the canonical operator surface

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

These gates cover replay artifact handling, parser validity, and support-tool
behavior.
