# Controlled Capture Perturbation Demo

## 1. Demo Claim

This demo demonstrates deterministic replay artifact comparison and exact
first-difference localization.

A controlled perturbation is injected during capture at frame `4096`.
The phase accumulator state continues canonical evolution.
Divergence occurs at frame `4096`; the canonical sequence resumes at frame `4097`.

Artifact directory `artifacts/demo_natural/` is retained to avoid fixture churn.
The demo name was corrected in documentation only.

## 2. How perturbation is induced

- Baseline run (`run_A.rpl`) is captured from default firmware.
- Perturbed run (`run_B.rpl`) is captured from firmware built with feature
  `demo-divergence`.
- `demo-divergence` perturbs exactly one captured frame during ISR execution:
  - target frame: `4096`
  - field: `input_sample`
  - operation: `+1`

## 3. Reproduction commands

```bash
# A) Baseline capture
make FW_FEATURES= flash-ur
SERIAL=/dev/ttyACM0 python3 scripts/repeat_capture.py \
  --contract rpl0 --runs 1 --signal-model phase8 --reset-mode stlink \
  --artifacts-dir artifacts/demo_natural/a_capture
cp artifacts/demo_natural/a_capture/run_01.bin artifacts/demo_natural/run_A.rpl

# B) Perturbed capture (capture-time perturbation)
make FW_FEATURES=demo-divergence flash-ur
SERIAL=/dev/ttyACM0 python3 scripts/repeat_capture.py \
  --contract rpl0 --runs 1 --signal-model phase8 --reset-mode stlink \
  --artifacts-dir artifacts/demo_natural/b_capture
cp artifacts/demo_natural/b_capture/run_01.bin artifacts/demo_natural/run_B.rpl

# C) Validate and compare
python3 scripts/artifact_tool.py verify artifacts/demo_natural/run_A.rpl --signal-model phase8
python3 scripts/artifact_tool.py verify artifacts/demo_natural/run_B.rpl --signal-model phase8
python3 scripts/artifact_tool.py verify artifacts/demo_natural/run_B.rpl --signal-model none
python3 scripts/artifact_diff.py artifacts/demo_natural/run_A.rpl artifacts/demo_natural/run_B.rpl
python3 scripts/artifact_tool.py inspect artifacts/demo_natural/run_A.rpl --frames 4093:4099
python3 scripts/artifact_tool.py inspect artifacts/demo_natural/run_B.rpl --frames 4093:4099
```

## 4. Expected validation behavior

```text
run_A --signal-model phase8  -> PASS
run_B --signal-model phase8  -> FAIL (expected)
run_B --signal-model none    -> PASS
```

Expected semantic failure snippet (`run_B --signal-model phase8`):

```text
Frame 4096 mismatch
expected (phase8): 0x00000000
observed:          0x00000001
```

Expected comparison output (`artifact_diff.py`):

```text
DIVERGENCE DETECTED
First divergence frame: 4096
Sample A: 0x00000000
Sample B: 0x00000001
Total frames: 10000
```

`artifact_diff.py` is diagnostic and not a CI gating command.

## 5. Why this matters

The demo proves that replay tooling can localize an exact first difference in a
controlled capture perturbation without claiming persistent execution divergence.
