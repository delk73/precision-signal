# CLI Contract

## 1. Authority
This file is the sole authority for the CLI schema and the related operational contract.
No other file is authoritative for the schema.
All downstream implementation and documentation must conform exactly to this contract.
Later schema changes require formal reopen of Chunk 0.

## 2. Command Set
The command set is fixed and consists of exactly `record`, `replay`, `diff`, and `envelope`.
No aliases are valid.
No hidden commands are valid.
No deprecated synonyms are valid.

## 3. Result Block
The result block is defined exactly as:

```text
RESULT: PASS|FAIL
COMMAND: record|replay|diff|envelope
TARGET: <input>
MODE: <runtime_mode|mock|none>
EQUIVALENCE: exact|diverged
FIRST_DIVERGENCE: none|step=<uint64> node=<node_id> cause=VAL_MISMATCH|TYPE_MISMATCH|OOB
ARTIFACT: artifacts/<run_id>
```

Field order is fixed.
All seven lines are required.
No additional lines are permitted.
`result.txt` must contain the same result block content as stdout, byte-for-byte.

## 4. Semantic Invariants
The following rules are exact semantic invariants:

```text
RESULT=PASS => EQUIVALENCE=exact AND FIRST_DIVERGENCE=none
RESULT=FAIL AND MODE!=mock => EQUIVALENCE=diverged AND FIRST_DIVERGENCE!=none
RESULT=FAIL AND MODE=mock => EQUIVALENCE=diverged AND FIRST_DIVERGENCE=none
```

Any result block that violates any invariant is invalid.
Mock mode is a contract-valid failure state.

## 5. Field Definitions
`TARGET` is the user-supplied path or identifier and must be emitted verbatim.
`MODE` permits exactly `runtime_mode`, `mock`, and `none`.
`mock` denotes zero-logic stub behavior.
`none` denotes that no mode applies.
No other `MODE` value is valid.
`node_id` lexical form is exactly `[A-Za-z0-9._:-]+`.

## 6. Divergence Cause Enum
The divergence cause enum values are exactly `VAL_MISMATCH`, `TYPE_MISMATCH`, and `OOB`.
`VAL_MISMATCH` means compared values differ while remaining type-comparable.
`TYPE_MISMATCH` means compared values differ by type.
`OOB` means out-of-bounds state, index, step, or trace position encountered during replay or comparison.

## 7. Artifact Layout
The artifact layout is defined exactly as:

```text
artifacts/<run_id>/
  result.txt
  trace.json
  meta.json
```

The layout is fixed.
All listed files are required.
No alternative authoritative layout exists.

## 8. run_id Format
The `run_id` format is defined exactly as:

```text
<utc_iso8601_ns>-<rand64_hex>
```

The timestamp component is UTC.
The timestamp component is nanosecond-resolution ISO 8601.
The suffix is a 64-bit random hexadecimal token.
No alternate `run_id` form is valid.

## 9. Publication Rule
The publication rule is defined exactly as:

```text
artifacts/.tmp_<run_id>/ -> artifacts/<run_id>/
```

The temporary and final directories must be siblings.
Both directories must reside under the same `artifacts/` parent.
Publication occurs only by atomic rename.
No writes may be performed directly into the final directory before publication.
No partially published final artifact directory may become visible.

## 10. Canonical Example
The canonical example path is exactly `examples/accumulator.json`.

## 11. Stability Scope
The following fields are excluded from determinism checks: `ARTIFACT`, `run_id`, `created_at`, `hostname`, `pid`.
The following fields are included in determinism checks: `COMMAND`, `TARGET`, `MODE`, `RESULT`, `EQUIVALENCE`, `FIRST_DIVERGENCE`.
Volatile artifact identity and metadata are excluded from semantic stability.

## 12. Freeze and Reopen Rule
Once accepted, Chunk 0 is frozen.
Downstream chunks must not modify the contract silently.
Any change to commands, result block, field semantics, enum values, artifact layout, `run_id`, publication rule, canonical example, or stability scope requires formal reopen of Chunk 0.
Formal reopen must include reason, impacted invariant, and downstream chunks affected.
