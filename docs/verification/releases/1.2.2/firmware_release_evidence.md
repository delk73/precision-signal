# Firmware Release Evidence (1.2.2)

This directory is the canonical retained release-evidence location for the
`1.2.2` release record.

Hardware capture and verification were re-executed for release `1.2.2`.
This retained bundle records the direct hardware-backed run for:

- board: `NUCLEO-F446RE`
- capture boundary: `TIM2 ISR (1 kHz)`
- frame count: `10000`
- canonical SHA256: `f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765`
- run directory: `artifacts/replay_runs/run_20260325T184038Z`

Use `VERIFICATION_GUIDE.md` for the release contract, `make gate` for the
canonical operator-facing release gate, and this directory for retained
release evidence collected for the release decision.

RUN_ID=run_20260325T184038Z
TIMESTAMP_UTC=2026-03-25T18:40:38Z
RUN_DIR=artifacts/replay_runs/run_20260325T184038Z
COMMAND_PATH=docs/replay/FW_F446_CAPTURE_v1.md

The retained hashes in this record derive from the 2026-03-25
`run_20260325T184038Z` recorded session identified above.

Post-release integrity repair note: this retained bundle originally carried a
stale `hash_check.txt` path set from an earlier capture export. The retained
hash inventory was corrected to the archived `run_20260325T184038Z` files
listed in this directory. No retained hashes or release claims changed.

## commands

- `make flash-ur`
- `make flash-verify-ur`
- `make flash-compare-ur`
- `make replay-check`
- `make replay-repeat-auto`

## results

- flash write verified
- MSP / ResetVec valid
- flash compare matched built binary
- artifact extraction complete
- `artifact_tool verify: PASS`
- baseline comparison: identical
- deterministic repeatability: PASS (5/5 runs identical)

## sha256 summary
baseline_sha256 f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765 baseline
f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765 run_01.bin
f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765 run_02.bin
f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765 run_03.bin
f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765 run_04.bin
f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765 run_05.bin

## manifest
contract_version=rpl0_capture_v1
artifact_version=1
schema_hash=8c6e82b4f9c80de029775d26da900a655686fd93038013ca759155ff02a68721
signal_model=phase8
baseline_path=artifacts/baseline.bin
baseline_sha256=f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765
requested_runs=5
completed_runs=5
final_status=PASS
failure_class=-
baseline_hash_match=true
timestamp_utc=2026-03-25T18:40:38Z
run_dir=/home/dce/src/precision-signal/artifacts/replay_runs/run_20260325T184038Z

## explicit hash check
f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765  artifacts/baseline.bin
f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765  artifacts/run.bin
f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765  artifacts/replay_runs/run_20260325T184038Z/run_01.bin
f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765  artifacts/replay_runs/run_20260325T184038Z/run_02.bin
f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765  artifacts/replay_runs/run_20260325T184038Z/run_03.bin
f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765  artifacts/replay_runs/run_20260325T184038Z/run_04.bin
f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765  artifacts/replay_runs/run_20260325T184038Z/run_05.bin
