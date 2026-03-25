# Deterministic Replay Divergence Demo

## 1. What this demo proves

This demo proves that deterministic execution artifacts can be compared directly and the exact first divergence frame can be located mechanically.

For deterministic execution under identical conditions, artifacts must be byte-identical.

## 2. Demo files

- `artifacts/demo/run_A.rpl`: baseline artifact.
- `artifacts/demo/run_B.rpl`: same artifact with one deterministic sample perturbation at frame `8321` (`sample += 1`).
- `scripts/mutate_frame.py`: deterministic one-frame mutation utility.
- `scripts/artifact_diff.py`: first-divergence comparator.

## 3. Reproduction commands

```bash
python3 scripts/artifact_tool.py verify artifacts/demo/run_A.rpl --signal-model none
python3 scripts/artifact_tool.py verify artifacts/demo/run_B.rpl --signal-model none
python3 scripts/artifact_diff.py artifacts/demo/run_A.rpl artifacts/demo/run_B.rpl
python3 scripts/artifact_tool.py inspect artifacts/demo/run_A.rpl --frames 8318:8324
python3 scripts/artifact_tool.py inspect artifacts/demo/run_B.rpl --frames 8318:8324
```

Optional: recreate `run_B.rpl` from `run_A.rpl` exactly.

```bash
python3 scripts/mutate_frame.py artifacts/demo/run_A.rpl 8321 +1 --out artifacts/demo/run_B.rpl
```

## 4. Expected output

Comparison reports exact first divergence:

```text
DIVERGENCE DETECTED
First divergence frame: 8321
Sample A: 0x00000081
Sample B: 0x00000082
```

Inspection of `8318:8324` shows identical frames before `8321`, the changed frame at `8321`, then continued suffix frames.

## 5. Why this matters

It turns deterministic execution into a concrete artifact-level debugging primitive: compare two runs and immediately jump to the first exact divergence point.
