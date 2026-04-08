# Demo Claim Matrix

This matrix is a reference demo artifact.
Release status is classified in [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md).
Verification authority is defined in [VERIFICATION_GUIDE.md](../../VERIFICATION_GUIDE.md).
Demo pages must not introduce new capability claims beyond what is already
evidenced elsewhere.

## Demo Comparison

| Demo | Perturbation | Divergence | Localized | Persistent | Classified |
| --- | --- | --- | --- | --- | --- |
| V1 | Output | Transient | Yes | No | No |
| V2 | Constructed persistent fixture | Persistent | Yes | Yes | No |
| V3 | Synthetic post-divergence trajectories from V2 canonical base | Transient / Persistent / Rate | Yes | Yes (offset case) | Yes (`transient`, `persistent_offset`, `rate_divergence`) |
| V4 | Synthetic structural-region fixtures from V2 canonical base | Header / Timer / IRQ / Sample / Mixed | Yes | Yes (sample case) | Yes (`transient`, `persistent_offset`, `rate_divergence`) + attributed (`header_schema`, `timer_delta`, `irq_state`, `sample_payload`) |

## Claims and Evidence

| Claim | Status | Validation Type | Evidence |
| --- | --- | --- | --- |
| deterministic signal generation | evidenced | semantic | `cargo run --release -p dpw4 --features cli --bin sig-util -- validate --mode quick` |
| artifact structural validation | evidenced | structural | `python3 scripts/artifact_tool.py verify <artifact> --signal-model <model>` |
| replay artifact parsing correctness | evidenced | structural | parser tests + replay CI gates (`test_artifact_parser_*`, [docs/replay/CI_GATES.md](../replay/CI_GATES.md)) |
| artifact comparison correctness | evidenced | structural | `python3 scripts/artifact_diff.py <artifact_a> <artifact_b>` |
| first divergence localization | evidenced | structural | `python3 scripts/artifact_diff.py <artifact_a> <artifact_b>` |
| artifact capture determinism | evidenced | structural | repeat capture workflow + identical SHA artifacts |
| persistent divergence localization | evidenced (Demo V2 fixture) | structural | `python3 scripts/artifact_diff.py artifacts/demo_persistent/run_A.rpl artifacts/demo_persistent/run_B.rpl` + `inspect` window after frame `4096` |
| divergence-shape classification | evidenced (Demo V3 fixtures) | structural | `python3 scripts/artifact_diff.py` over `artifacts/demo_v3/*` with allowed labels: `transient`, `persistent_offset`, `rate_divergence` |
| divergence-region attribution | evidenced (Demo V4 fixtures) | structural | `python3 scripts/artifact_diff.py` over `artifacts/demo_v4/*` with allowed regions: `header_schema`, `timer_delta`, `irq_state`, `sample_payload` |
| persistent execution-state divergence (captured) | evidenced locally | structural | committed captured artifacts (`artifacts/demo_persistent/run_A_captured.rpl`, `artifacts/demo_persistent/run_B_captured.rpl`) plus local/archive runlogs (runlogs directory is ignored payload, not committed evidence) |
