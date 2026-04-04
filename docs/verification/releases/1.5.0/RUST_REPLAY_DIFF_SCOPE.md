# Rust Replay Diff Scope (1.5.0)

## Released Claim

Release `1.5.0` classifies one bounded Rust replay command as released:
`replay-host diff`.

This release claim is limited to the retained `artifacts/rpl0/` proof corpus and
to the exact accepted artifact class mechanically demonstrated by that corpus.
It does not generalize beyond the retained proof inputs.

## Exact Command Invocations Retained

Identical comparison:

```bash
cargo run -q -p replay-host -- diff artifacts/rpl0/run_20260331T150000Z.import1.rpl artifacts/rpl0/run_20260331T150000Z.import1.rpl
```

Observed stdout is retained in `replay_host_diff_identical.txt`.

Known divergence comparison:

```bash
cargo run -q -p replay-host -- diff artifacts/rpl0/run_20260331T150000Z.import1.rpl artifacts/rpl0/run_20260331T150000Z.frame017_plus1.rpl
```

Observed stdout is retained in `replay_host_diff_frame17.txt`.

Negative CLI case:

```bash
cargo run -q -p replay-host -- diff artifacts/rpl0/run_20260331T150000Z.import1.rpl
```

Observed non-zero stderr is retained in `replay_host_diff_missing_arg.txt`.

Focused `replay-host` test invocation retained for this proof:

```bash
cargo test -p replay-host operator_path_reports_ -- --nocapture
```

Observed output is retained in `cargo_test_replay_host_operator_path.txt`.
That retained file, not the future stability of the filter string, is the
release record for this proof run.

## Accepted Artifact Class Mechanically Demonstrated By The Corpus

The retained `artifacts/rpl0/` manifest demonstrates the proof corpus accepted
by the current binary:

- checked-in `.rpl` artifacts under `artifacts/rpl0/`
- canonical artifact length `160152`
- `frame_count = 10000`
- `frame_size = 16`
- `schema_len = 0`
- retained deterministic perturbation at frame index `17`

The current code path behind this released slice remains the existing
implementation boundary:

- RPL0 format version 0 replay support in code
- RPL0 `version = 1` container parsing in code
- legacy 16-byte `EventFrame0` replay semantics in code

This bundle retains only the command behavior mechanically demonstrated on the
`artifacts/rpl0/` proof corpus above.

## Observed Command Behavior

The current binary was observed to:

- emit the exact stdout retained in `replay_host_diff_identical.txt` for the
  identical proof input
- emit the exact stdout retained in `replay_host_diff_frame17.txt` for the
  retained frame-17 perturbation proof input
- exit non-zero and emit the exact usage/error text retained in
  `replay_host_diff_missing_arg.txt` for the missing-argument case
- report four matching operator-path tests in
  `cargo_test_replay_host_operator_path.txt`, including the retained
  identical-input and retained frame-17 perturbation paths exercised during this
  release cut

## Explicitly Unreleased Remainder

`1.5.0` does not release:

- `replay-host import-interval-csv`
- `replay-host validate-interval-csv`
- any broader `replay-host` claim outside the retained `artifacts/rpl0/` proof corpus
- schema-aware Rust replay semantics
- `replay-fw-f446`
