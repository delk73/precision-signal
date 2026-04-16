# Replay CI Gates

The following CI checks enforce replay crate boundaries, mechanical constraints,
and baseline artifact integrity.
Step names and commands match current `.github/workflows/ci.yml`.

An embedded firmware build oracle is defined for `replay-fw-f446` in CI.
It asserts:
- ELF exists at `target/thumbv7em-none-eabihf/debug/replay-fw-f446`
- `.text` section is present
- vector section is present (`.vector_table` or `.isr_vector`)
- entry point is non-zero

Flashing and hardware validation remain local/operator steps.
Firmware binary emission is verified locally via Makefile targets (`make fw-bin`, `make flash`, `make flash-compare`).

## Required Boundary/Mechanical Gates

- `P5B Artifact Boundary — AST Hygiene (WARN; CORE-LEAK=STOP)`
  - `cargo run --locked -p audit-float-boundary -- --mode phase5b`
  - Enforces float-surface policy with CORE-LEAK STOP paths including `crates/replay-core/src/` and `crates/replay-embed/src/`.

- `Dependency DAG Oracle (no replay backedges)`
  - `python3 scripts/check_no_replay_backedges.py`
  - Blocks direct/transitive dependency paths from `{dpw4, geom-signal, geom-spatial}` to `{replay-core, replay-embed, replay-host, replay-cli}`.

- `Replay no_std Oracle (thumbv7em-none-eabihf)`
  - `cargo check -p replay-core --no-default-features --target thumbv7em-none-eabihf --locked`
  - `cargo check -p replay-embed --no-default-features --target thumbv7em-none-eabihf --locked`
  - Ensures replay core/embed no_std compatibility on the embedded target.

- `Replay Baseline Verify (phase8)`
  - `python3 scripts/artifact_tool.py verify artifacts/baseline.bin --signal-model phase8`
  - `python3 scripts/artifact_tool.py hash artifacts/baseline.bin --expect <pinned-sha256>`
  - Intentional behavior split:
    - `verify` is strict and rejects trailing bytes.
    - `hash` computes identity over the canonical artifact prefix and may ignore trailing bytes.
  - Verifies structural/model validity and detects accidental baseline mutation.
  - Active STM32 capture contract is [docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md](INTERVAL_CAPTURE_CONTRACT_v1.md). Historical RPL0 capture notes remain retained for legacy inspection only.

- `Replay Artifact Parser Adversarial Tests`
  - `python3 scripts/test_artifact_parser_adversarial.py`
  - Exercises malformed artifact rejection paths (header/schema/frame truncation and length corruption).

- `Replay Artifact Parser Valid v1 Coverage`
  - `python3 scripts/test_artifact_parser_valid_v1.py`
  - Exercises deterministic acceptance-space coverage for valid v1 artifacts (including zero-frame and extended-header acceptance).

- `Replay Artifact Parser Mutation Corpus`
  - `python3 scripts/test_artifact_parser_mutation_corpus.py`
  - Exercises near-valid structural mutation boundaries (reject and accept expected classes).

- `Replay Artifact Tool Verify CLI Regression`
  - `python3 scripts/test_artifact_tool_verify.py`
  - Verifies `artifact_tool.py verify` exit behavior for valid, trailing-byte, and malformed inputs.

- `Replay Artifact Tool Hash CLI Regression`
  - `python3 scripts/test_artifact_tool_hash.py`
  - Verifies `artifact_tool.py hash` canonical behavior including trailing-byte tolerance and `--expect` mismatch handling.

- `Replay Artifact Compare CLI Regression`
  - `python3 scripts/test_compare_artifact.py`
  - Verifies `compare_artifact.py` deterministic result classes (identical/header mismatch/frame divergence/parse-fail cases).

- `Replay Baseline Metadata Consistency`
  - Verifies `baseline.json`, `baseline.sha256`, and `baseline.bin` agree.
  - Enforces: `artifact_file == "baseline.bin"`, `signal_model == "phase8"`,
    SHA-256 equality across all three sources.
