# Quantization Probe

Minimal host-only witness for deterministic divergence under explicit precision reduction.

This directory retains WIP-only experimental evidence. It is not part of the repository release surface.

Pipeline per corpus sample:

1. affine transform: `t = sample * 5 + 3`
2. accumulation: `acc += t`
3. bounded stage: `bounded = clamp(acc, -1024, 1024)`
4. threshold stage: `decision = 1 if bounded >= 256 else 0`
5. emitted sample: `bounded + decision`

Path split:

- `baseline`: uses `t` directly
- `quantized`: truncates transform precision each step with `(t >> 3) << 3`

The checked-in corpus keeps all but one transformed value aligned to the quantizer bucket. The outlier at frame_idx=4 (0-based) introduces a stable one-count persistent offset that is intended to produce a stable first divergence observable via `scripts/artifact_diff.py`.

Run examples:

```bash
python3 experiments/quantization_probe/generate_probe_artifact.py --mode baseline --out /tmp/quant_probe_baseline.rpl
python3 experiments/quantization_probe/generate_probe_artifact.py --mode quantized --out /tmp/quant_probe_quantized.rpl
python3 scripts/artifact_diff.py /tmp/quant_probe_baseline.rpl /tmp/quant_probe_quantized.rpl
```

Repeatability verification:

```bash
python3 experiments/quantization_probe/generate_probe_artifact.py --mode baseline --out /tmp/quant_probe_baseline_run1.rpl
python3 experiments/quantization_probe/generate_probe_artifact.py --mode baseline --out /tmp/quant_probe_baseline_run2.rpl
python3 experiments/quantization_probe/generate_probe_artifact.py --mode quantized --out /tmp/quant_probe_quantized_run1.rpl
python3 experiments/quantization_probe/generate_probe_artifact.py --mode quantized --out /tmp/quant_probe_quantized_run2.rpl
cmp -s /tmp/quant_probe_baseline_run1.rpl /tmp/quant_probe_baseline_run2.rpl
cmp -s /tmp/quant_probe_quantized_run1.rpl /tmp/quant_probe_quantized_run2.rpl
sha256sum /tmp/quant_probe_baseline_run1.rpl /tmp/quant_probe_baseline_run2.rpl
sha256sum /tmp/quant_probe_quantized_run1.rpl /tmp/quant_probe_quantized_run2.rpl
python3 scripts/artifact_diff.py /tmp/quant_probe_baseline_run1.rpl /tmp/quant_probe_quantized_run1.rpl
```

Observed BBB host result:

- `make gate`: PASS
- baseline artifact run output: `frame_count: 12`, `first_output: 8`, `last_output: 121`
- quantized artifact run output: `frame_count: 12`, `first_output: 8`, `last_output: 120`
- repeated baseline hash: `67e309b08d7bf8db286869b2b81a23da297b7ccfd2ecd9e322830729e69a9e69`
- repeated quantized hash: `fe992bec716077dc20eb94550d007022439fef871a1bf101a30727b2d18a8abf`
- `artifact_diff.py`: `First divergence frame: 4`
- classification summary: `shape_class: persistent_offset`, `primary_region: sample_payload`, `evolution_class: bounded_persistent`
- first divergent samples: `Sample A: 0x00000041`, `Sample B: 0x00000040`
