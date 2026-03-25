# Firmware Release Evidence (1.2.0)

This directory is the canonical retained release-evidence location for the
`1.2.0` release record.

Use `VERIFICATION_GUIDE.md` for the release contract, `make gate` for the
canonical operator-facing release gate, and this directory for retained
release evidence collected for the release decision.

RUN_DIR=artifacts/replay_runs/run_20260319T044754Z

## sha256 summary
baseline_sha256 b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3 baseline
b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3 run_01.bin
b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3 run_02.bin
b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3 run_03.bin

## manifest
contract_version=replay_capture_v0
signal_model=phase8
baseline_path=artifacts/baseline.bin
baseline_sha256=b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3
requested_runs=3
completed_runs=3
final_status=PASS
failure_class=-
baseline_hash_match=true
timestamp_utc=2026-03-19T04:47:54Z
run_dir=/home/dce/src/precision-dpw/artifacts/replay_runs/run_20260319T044754Z

## explicit hash check
b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3  artifacts/baseline.bin
b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3  artifacts/run.bin
b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3  artifacts/replay_runs/run_20260319T044754Z/run_01.bin
b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3  artifacts/replay_runs/run_20260319T044754Z/run_02.bin
b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3  artifacts/replay_runs/run_20260319T044754Z/run_03.bin
