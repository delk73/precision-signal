# Firmware Release Evidence (1.2.2)

This directory is the canonical retained release-evidence location for the
`1.2.2` release record.

This `1.2.2` retained bundle promotes the previously retained hardware-backed
capture evidence from the `1.2.1` release record. No new hardware rerun is
claimed here; the raw retained evidence files in this directory are copied
forward unchanged from `docs/verification/releases/1.2.1/`.

Use `VERIFICATION_GUIDE.md` for the release contract, `make gate` for the
canonical operator-facing release gate, and this directory for retained
release evidence collected for the release decision.

RUN_ID=run_20260320T160831Z
TIMESTAMP_UTC=2026-03-20T16:08:31Z
RUN_DIR=artifacts/replay_runs/run_20260320T160831Z
COMMAND_PATH=docs/replay/FW_F446_CAPTURE_v1.md

The retained hashes in this record derive from the 2026-03-20
`run_20260320T160831Z` recorded session identified above.

## sha256 summary
baseline_sha256 f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765 baseline
f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765 run_01.bin
f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765 run_02.bin
f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765 run_03.bin

## manifest
contract_version=rpl0_capture_v1
artifact_version=1
schema_hash=8c6e82b4f9c80de029775d26da900a655686fd93038013ca759155ff02a68721
signal_model=phase8
baseline_path=artifacts/baseline.bin
baseline_sha256=f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765
requested_runs=3
completed_runs=3
final_status=PASS
failure_class=-
baseline_hash_match=true
timestamp_utc=2026-03-20T16:08:31Z
run_dir=/home/dce/src/precision-dpw/artifacts/replay_runs/run_20260320T160831Z
COMMAND_PATH=docs/replay/FW_F446_CAPTURE_v1.md

## explicit hash check
f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765  artifacts/baseline.bin
f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765  artifacts/run.bin
f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765  artifacts/replay_runs/run_20260320T160831Z/run_01.bin
f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765  artifacts/replay_runs/run_20260320T160831Z/run_02.bin
f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765  artifacts/replay_runs/run_20260320T160831Z/run_03.bin
