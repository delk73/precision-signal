# Demo Landing

Demo material explains and exercises replay divergence behavior. It is
support/reference material only. Active authority remains with
[docs/START_HERE.md](../START_HERE.md), especially
[docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md),
[docs/replay/FW_F446_CAPTURE_v1.md](../replay/FW_F446_CAPTURE_v1.md), and
[docs/replay/DIVERGENCE_SEMANTICS.md](../replay/DIVERGENCE_SEMANTICS.md).

## Demo Routes

| Route | Class | Scope |
| --- | --- | --- |
| [demo_capture_perturbation.md](demo_capture_perturbation.md) | historical/demo | One-frame deterministic perturbation walkthrough. |
| [demo_persistent_divergence.md](demo_persistent_divergence.md) | historical/demo | Persistent divergence demonstration. |
| [demo_v3_divergence_classification.md](demo_v3_divergence_classification.md) | historical/demo | Shape classification fixture route. |
| [demo_v4_region_attribution.md](demo_v4_region_attribution.md) | historical/demo | Region attribution fixture route. |
| [demo_v5_evolution.md](demo_v5_evolution.md) | historical/demo | Evolution classification fixture route. |
| [demo_captured_divergence.md](demo_captured_divergence.md) | historical/demo | Captured divergence pair support material. |
| [demo_evidence_packaging.md](demo_evidence_packaging.md) | active support | Packaged replay proof/support route, not release authority. |
| [demo_claim_matrix.md](demo_claim_matrix.md) | active support | Demo claim/evidence boundary. |
| [demo_visual.html](demo_visual.html) | historical/demo | Visual support page. |

## Minimal Reproduction

The original one-frame perturbation walkthrough remains reproducible from the
repository root:

```bash
python3 scripts/artifact_tool.py verify artifacts/demo/run_A.rpl --signal-model none
python3 scripts/artifact_tool.py verify artifacts/demo/run_B.rpl --signal-model none
python3 scripts/artifact_diff.py artifacts/demo/run_A.rpl artifacts/demo/run_B.rpl
python3 scripts/artifact_tool.py inspect artifacts/demo/run_A.rpl --frames 8318:8324
python3 scripts/artifact_tool.py inspect artifacts/demo/run_B.rpl --frames 8318:8324
```

Optional recreation of `run_B.rpl` from `run_A.rpl`:

```bash
python3 scripts/mutate_frame.py artifacts/demo/run_A.rpl 8321 +1 --out artifacts/demo/run_B.rpl
```

## Expected Output

The comparison reports exact first divergence:

```text
DIVERGENCE DETECTED
First divergence frame: 8321
Sample A: 0x00000081
Sample B: 0x00000082
```

Inspection of `8318:8324` shows identical frames before `8321`, the changed frame at `8321`, then continued suffix frames.
