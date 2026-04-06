# CLI Contract

## 1. Authority
This file is the sole authority for the CLI schema and the related operational contract, including invocation, process termination semantics, stream separation, result emission conditions, result block schema, artifact contract, and stability scope.
No other file is authoritative for the schema or operational contract.
All downstream implementation and documentation must conform exactly to this contract.


## 2. Command Set
The command set is fixed and consists of exactly `record`, `replay`, `diff`, and `envelope`.
No aliases are valid.
No hidden commands are valid.
No deprecated synonyms are valid.

## 3. Invocation Grammar
The invocation grammar is defined exactly as `precision <command> <target> --mode <mode>`.
The command position is fixed.
The target position is fixed.
The spelling `--mode` is fixed.
The mode domain is fixed to `runtime_mode|mock|none`.
No aliases are valid.
No alternate authoritative invocation grammar exists.

## 4. Result Block
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
For contract-valid invocations, stdout is reserved for this result block only.

## 5. Semantic Invariants
The following rules are exact semantic invariants:

```text
RESULT=PASS => EQUIVALENCE=exact AND FIRST_DIVERGENCE=none
RESULT=FAIL AND MODE!=mock => EQUIVALENCE=diverged AND FIRST_DIVERGENCE!=none
RESULT=FAIL AND MODE=mock => EQUIVALENCE=diverged AND FIRST_DIVERGENCE=none
```

Any result block that violates any invariant is invalid.
Mock mode is a contract-valid failure state.

## 6. Field Definitions
`TARGET` is the user-supplied path or identifier and must be emitted verbatim.
`MODE` permits exactly `runtime_mode`, `mock`, and `none`.
`mock` denotes zero-logic stub behavior.
`none` denotes that no mode applies.
No other `MODE` value is valid.
`node_id` lexical form is exactly `[A-Za-z0-9._:-]+`.

## 7. Divergence Cause Enum
The divergence cause enum values are exactly `VAL_MISMATCH`, `TYPE_MISMATCH`, and `OOB`.
`VAL_MISMATCH` means compared values differ while remaining type-comparable.
`TYPE_MISMATCH` means compared values differ by type.
`OOB` means out-of-bounds state, index, step, or trace position encountered during replay or comparison.

## 8. Exit Code Contract
`PASS` exits with code `0`.
Contract-valid `FAIL` exits with code `1`.
Contract-invalid invocation or unrecoverable internal failure exits with code `2`.
Where a result block exists, the exit code must agree with it.

## 9. stdout and stderr Contract
Stdout is reserved for the result block only.
No logs, diagnostics, warnings, or progress output are permitted on stdout.
Diagnostics, warnings, and fatal messages go to stderr.
Stderr is not part of the result block schema.

## 10. Fatal Failure Contract
If invocation parsing succeeds and target acquisition begins, the tool must attempt to emit a valid `FAIL` result block and publish an artifact before exiting `1` whenever the failure is representable within the result contract.
If invocation parsing fails before a contract-valid result can be formed, or an unrecoverable internal failure prevents result-block emission, the process must exit `2`.
Parse errors, missing required arguments, invalid command spelling, and illegal mode values are contract-invalid invocations.
Artifact publication is not required for code-`2` failures.

## 11. Command Applicability
All four commands must emit the same seven-line result block for every contract-valid invocation.
No command-specific schema variants are valid.
`replay`, `diff`, and `envelope` use full divergence semantics normally.
`record` must still emit `EQUIVALENCE` and `FIRST_DIVERGENCE` under a fixed rule.
If `record` does not inherently compare traces, it must populate `EQUIVALENCE=exact` and `FIRST_DIVERGENCE=none`.
Any contract-valid `FAIL` for `record` must still satisfy the general `FAIL` invariant.

## 12. Artifact Layout
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

## 13. run_id Format
The `run_id` format is defined exactly as:

```text
<utc_iso8601_ns>-<rand64_hex>
```

The timestamp component is UTC.
The timestamp component is nanosecond-resolution ISO 8601.
The suffix is a 64-bit random hexadecimal token.
No alternate `run_id` form is valid.

## 14. Publication Rule
The publication rule is defined exactly as:

```text
artifacts/.tmp_<run_id>/ -> artifacts/<run_id>/
```

The temporary and final directories must be siblings.
Both directories must reside under the same `artifacts/` parent.
Publication occurs only by atomic rename.
No writes may be performed directly into the final directory before publication.
No partially published final artifact directory may become visible.
Artifact publication is required for contract-valid `PASS` and `FAIL` results.
Artifact publication is not required for code-`2` failures.

## 15. Canonical Example
The canonical example path is exactly `examples/accumulator.json`.

## 16. Stability Scope
The following fields are excluded from determinism checks: `ARTIFACT`, `run_id`, `created_at`, `hostname`, `pid`.
The following fields are included in determinism checks: `COMMAND`, `TARGET`, `MODE`, `RESULT`, `EQUIVALENCE`, `FIRST_DIVERGENCE`.
Volatile artifact identity and metadata are excluded from semantic stability.

## 17. Freeze and Reopen Rule
Once accepted, Chunk 0 is frozen.
Downstream chunks must not modify the contract silently.
Any change to invocation, process termination semantics, stream separation, or result emission conditions requires formal reopen of Chunk 0.
Any change to commands, result block, field semantics, enum values, artifact layout, `run_id`, publication rule, canonical example, or stability scope requires formal reopen of Chunk 0.
Formal reopen must include reason, impacted invariant, and downstream chunks affected.
