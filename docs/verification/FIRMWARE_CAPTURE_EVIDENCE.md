# Firmware Capture Evidence

## Context

- Audit date: 2026-03-18
- Hardware context: committed STM32F446 capture artifacts in `artifacts/demo_captured/`
- Historical local execution context from the retained audit log: `/home/dce/src/precision-dpw`

## Commands

```bash
make fw
python3 scripts/artifact_tool.py verify artifacts/demo_captured/run_A.rpl --signal-model phase8
python3 scripts/artifact_tool.py hash artifacts/demo_captured/run_A.rpl
python3 scripts/artifact_tool.py verify artifacts/demo_captured/run_B.rpl --signal-model none
python3 scripts/artifact_tool.py hash artifacts/demo_captured/run_B.rpl
python3 scripts/artifact_diff.py artifacts/demo_captured/run_A.rpl artifacts/demo_captured/run_B.rpl
```

## Artifact Hashes

- `artifacts/demo_captured/run_A.rpl`: `b246aa88e5ffdfff32ff32bb39c6aa517601888772e278a285b1b42674430bf3`
- `artifacts/demo_captured/run_B.rpl`: `5957b539722fdd0021b56882c7eb04b9c68ef16484c18b6293cb4ff0d80a5d6d`

## Pass / Fail Criteria

- `make fw` must exit `0` and produce `target/thumbv7em-none-eabihf/debug/replay-fw-f446`
- `artifact_tool.py verify` must report `PASS: artifact structure is valid`
- `artifact_tool.py hash` must report the hashes above
- `artifact_diff.py` must report divergence at frame `4096`

## Result

- Firmware build passed locally.
- Artifact verification and hashing passed for both committed capture artifacts.
- Divergence analysis passed on the committed capture pair.
- No direct operator-path capture was exercised in this audit.
