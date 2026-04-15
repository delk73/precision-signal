# Replay Capture Acceptance Contract (Historical v0)

> **Status: LEGACY / HISTORICAL**
> v0 parsing remains available for historical inspection only. The active
> firmware hardware gate is the STM32 interval CSV path, not UART `RPL0`
> capture.

Status: historical v0 boundary retained for audit traceability.

## Contract Constants
- Magic: `RPL0`
- Version: `0`
- Frame count: `10000`
- Frame size: `16`
- Artifact size: `160016`
- Endianness: little-endian
- Transport: UART `115200 8N1`

Canonical baseline SHA-256:

`b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3`

## Contract Boundary
- Wire contract is defined by explicit encoder outputs and fixed v0 size constants.
- Struct representation (`#[repr(C)]`) is implementation discipline, not the normative artifact boundary.

## Acceptance Gates

### Gate 0: Formal wire determinism proof (Kani)
- Encoder output sizes match v0 constants.
- Encoded field byte positions match v0 wire contract.
- Multibyte fields are little-endian.
- Serialization emits bytes only from explicit field encodes; raw struct memory is never serialized.

### Gate 1: Single capture acceptance (`make rpl0-replay-check`)
- Human-in-the-loop.
- Listener waits for replay header.
- Operator presses reset once after listener starts.
- Capture command is transport-only.
- Verification enforces signal model.
- Candidate artifact must be byte-identical to baseline.

### Gate 2: Repeatability acceptance (`make rpl0-replay-repeat-check`)
- Wrapper creates run directory under `artifacts/replay_runs/`.
- Baseline artifacts are read-only.
- Required outputs: `replay_manifest_v0.txt`, `sha256_summary.txt`.
- Success requires all run SHA-256 values equal each other and equal baseline SHA-256.
- Human-in-the-loop: each run waits for listener readiness, then operator presses reset once.

Required `replay_manifest_v0.txt` fields:
- `contract_version`
- `signal_model`
- `baseline_path`
- `baseline_sha256`
- `requested_runs`
- `completed_runs`
- `final_status`
- `failure_class` (if failed)
- `baseline_hash_match` (`true|false`)
- `timestamp_utc` (UTC ISO-8601, `YYYY-MM-DDTHH:MM:SSZ`)
- `run_dir`

### Gate 3: Cross-MCU portability proof
- Run only after Gate 0/1/2 are stable and F446 10-run evidence is archived.
- F411 success requires SHA-256 equal to canonical baseline hash.

## Reference Commands
```bash
make rpl0-replay-check
make rpl0-replay-repeat-check
REPLAY_REPEAT_RUNS=10 make rpl0-replay-repeat-check
```

## Scope Lock (Stabilization Horizon)
- No new signal models.
- No artifact format or frame layout changes.
- No DSP feature work.
- No new auxiliary subsystems.
- No baseline promotion occurs in stabilization gates.
