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
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode baseline --out /tmp/quant_probe_baseline.rpl
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode quantized --out /tmp/quant_probe_quantized.rpl
PYTHONPATH=. python3 scripts/artifact_diff.py /tmp/quant_probe_baseline.rpl /tmp/quant_probe_quantized.rpl
```

Repeatability verification:

```bash
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode baseline --out /tmp/quant_probe_baseline_run1.rpl
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode baseline --out /tmp/quant_probe_baseline_run2.rpl
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode quantized --out /tmp/quant_probe_quantized_run1.rpl
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode quantized --out /tmp/quant_probe_quantized_run2.rpl
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

WIP-006 canonical parity matrix

The reduced cross-surface parity matrix for WIP-006 uses only corpus `C1` with baseline and quantized shift `3`.

Exact commands:

```bash
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode baseline --corpus C1 --out /tmp/WIP006_C1_Q3_baseline_run1.rpl
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode baseline --corpus C1 --out /tmp/WIP006_C1_Q3_baseline_run2.rpl
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode quantized --corpus C1 --quant-shift 3 --out /tmp/WIP006_C1_Q3_quant_run1.rpl
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode quantized --corpus C1 --quant-shift 3 --out /tmp/WIP006_C1_Q3_quant_run2.rpl
cmp -s /tmp/WIP006_C1_Q3_baseline_run1.rpl /tmp/WIP006_C1_Q3_baseline_run2.rpl
cmp -s /tmp/WIP006_C1_Q3_quant_run1.rpl /tmp/WIP006_C1_Q3_quant_run2.rpl
sha256sum /tmp/WIP006_C1_Q3_baseline_run1.rpl /tmp/WIP006_C1_Q3_baseline_run2.rpl /tmp/WIP006_C1_Q3_quant_run1.rpl /tmp/WIP006_C1_Q3_quant_run2.rpl
PYTHONPATH=. python3 scripts/artifact_diff.py /tmp/WIP006_C1_Q3_baseline_run1.rpl /tmp/WIP006_C1_Q3_quant_run1.rpl
```

Parity target:

- `first_divergence_frame = 4`
- `classification = persistent_offset`
- `baseline_invariant = true`

Observed BBB parity result:

- baseline hash `67e309b08d7bf8db286869b2b81a23da297b7ccfd2ecd9e322830729e69a9e69`
- quantized hash `fe992bec716077dc20eb94550d007022439fef871a1bf101a30727b2d18a8abf`
- `first_divergence_frame = 4`
- `classification = persistent_offset`
- `baseline_invariant = true`

Controlled witness matrix

The matrix below is retained as WIP-only host evidence from the current Ubuntu x86_64 host. It does not replace the retained phase-1 BBB evidence above.

Exact matrix commands:

```bash
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode baseline --corpus C1 --out /tmp/C1_Q2_baseline_run1.rpl
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode baseline --corpus C1 --out /tmp/C1_Q2_baseline_run2.rpl
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode quantized --corpus C1 --quant-shift 2 --out /tmp/C1_Q2_quant_run1.rpl
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode quantized --corpus C1 --quant-shift 2 --out /tmp/C1_Q2_quant_run2.rpl
cmp -s /tmp/C1_Q2_baseline_run1.rpl /tmp/C1_Q2_baseline_run2.rpl
cmp -s /tmp/C1_Q2_quant_run1.rpl /tmp/C1_Q2_quant_run2.rpl
sha256sum /tmp/C1_Q2_baseline_run1.rpl /tmp/C1_Q2_baseline_run2.rpl
sha256sum /tmp/C1_Q2_quant_run1.rpl /tmp/C1_Q2_quant_run2.rpl
python3 scripts/artifact_diff.py /tmp/C1_Q2_baseline_run1.rpl /tmp/C1_Q2_quant_run1.rpl

PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode baseline --corpus C1 --out /tmp/C1_Q3_baseline_run1.rpl
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode baseline --corpus C1 --out /tmp/C1_Q3_baseline_run2.rpl
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode quantized --corpus C1 --quant-shift 3 --out /tmp/C1_Q3_quant_run1.rpl
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode quantized --corpus C1 --quant-shift 3 --out /tmp/C1_Q3_quant_run2.rpl
cmp -s /tmp/C1_Q3_baseline_run1.rpl /tmp/C1_Q3_baseline_run2.rpl
cmp -s /tmp/C1_Q3_quant_run1.rpl /tmp/C1_Q3_quant_run2.rpl
sha256sum /tmp/C1_Q3_baseline_run1.rpl /tmp/C1_Q3_baseline_run2.rpl
sha256sum /tmp/C1_Q3_quant_run1.rpl /tmp/C1_Q3_quant_run2.rpl
python3 scripts/artifact_diff.py /tmp/C1_Q3_baseline_run1.rpl /tmp/C1_Q3_quant_run1.rpl

PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode baseline --corpus C1 --out /tmp/C1_Q4_baseline_run1.rpl
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode baseline --corpus C1 --out /tmp/C1_Q4_baseline_run2.rpl
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode quantized --corpus C1 --quant-shift 4 --out /tmp/C1_Q4_quant_run1.rpl
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode quantized --corpus C1 --quant-shift 4 --out /tmp/C1_Q4_quant_run2.rpl
cmp -s /tmp/C1_Q4_baseline_run1.rpl /tmp/C1_Q4_baseline_run2.rpl
cmp -s /tmp/C1_Q4_quant_run1.rpl /tmp/C1_Q4_quant_run2.rpl
sha256sum /tmp/C1_Q4_baseline_run1.rpl /tmp/C1_Q4_baseline_run2.rpl
sha256sum /tmp/C1_Q4_quant_run1.rpl /tmp/C1_Q4_quant_run2.rpl
python3 scripts/artifact_diff.py /tmp/C1_Q4_baseline_run1.rpl /tmp/C1_Q4_quant_run1.rpl

PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode baseline --corpus C2 --out /tmp/C2_Q3_baseline_run1.rpl
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode baseline --corpus C2 --out /tmp/C2_Q3_baseline_run2.rpl
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode quantized --corpus C2 --quant-shift 3 --out /tmp/C2_Q3_quant_run1.rpl
PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode quantized --corpus C2 --quant-shift 3 --out /tmp/C2_Q3_quant_run2.rpl
cmp -s /tmp/C2_Q3_baseline_run1.rpl /tmp/C2_Q3_baseline_run2.rpl
cmp -s /tmp/C2_Q3_quant_run1.rpl /tmp/C2_Q3_quant_run2.rpl
sha256sum /tmp/C2_Q3_baseline_run1.rpl /tmp/C2_Q3_baseline_run2.rpl
sha256sum /tmp/C2_Q3_quant_run1.rpl /tmp/C2_Q3_quant_run2.rpl
python3 scripts/artifact_diff.py /tmp/C2_Q3_baseline_run1.rpl /tmp/C2_Q3_quant_run1.rpl
```

Observed matrix result:

- Baseline artifacts are identical across all matrix cases (expected; baseline path is unchanged).
- `C1-Q2`: baseline repeat `PASS`, baseline hash `67e309b08d7bf8db286869b2b81a23da297b7ccfd2ecd9e322830729e69a9e69`, quantized repeat `PASS`, quantized hash `fe992bec716077dc20eb94550d007022439fef871a1bf101a30727b2d18a8abf`, first divergence `frame_idx=4`, `shape_class=persistent_offset`, `primary_region=sample_payload`, `evolution_class=bounded_persistent`
- `C1-Q3`: baseline repeat `PASS`, baseline hash `67e309b08d7bf8db286869b2b81a23da297b7ccfd2ecd9e322830729e69a9e69`, quantized repeat `PASS`, quantized hash `fe992bec716077dc20eb94550d007022439fef871a1bf101a30727b2d18a8abf`, first divergence `frame_idx=4`, `shape_class=persistent_offset`, `primary_region=sample_payload`, `evolution_class=bounded_persistent`
- `C1-Q4`: baseline repeat `PASS`, baseline hash `67e309b08d7bf8db286869b2b81a23da297b7ccfd2ecd9e322830729e69a9e69`, quantized repeat `PASS`, quantized hash `a1898d79ef3b55f8f60cdc4cb24467b25665f630a7fe0dc4a7a39318af228d83`, first divergence `frame_idx=0`, `shape_class=rate_divergence`, `primary_region=sample_payload`, `evolution_class=monotonic_growth`
- `C2-Q3`: baseline repeat `PASS`, baseline hash `d6009946947b0e1b1ead89dac01112cda52bf116b711ec0728722e384f7e17d1`, quantized repeat `PASS`, quantized hash `a0fa69c57d1f9356e2fa3549d8c3233e8ee777730acecaba8a090ed0a2fe5724`, first divergence `frame_idx=7`, `shape_class=persistent_offset`, `primary_region=sample_payload`, `evolution_class=bounded_persistent`

Note: `C1-Q2` and `C1-Q3` produce identical hashes and divergence behavior for this corpus (quantization collapse under current input distribution).
Observation: Increasing quantization strength (`Q4`) changes classification from `persistent_offset` to `rate_divergence`.
