# Divergence Classification Demo (V3)

## 1. Problem statement

Demo V2 localizes the first frame where two deterministic replay artifacts diverge.
Demo V3 extends that result by classifying the post-divergence shape so artifact
comparison supports diagnosis, not only detection.

## 2. Classification definitions (implemented rules)

Demo V3 uses `scripts/artifact_diff.py` to locate first mismatch frame `N` and
classify sample trajectory for frames `i >= N` using deterministic rules.

Let `diff[i] = sample_A[i] - sample_B[i]` and `K = 8`.

- `transient`
  - Reconvergence index `r` exists within `N+1..N+K`.
  - `diff[r] == 0`.
  - `diff[j] == 0` for all `j` from `r` through `min(N+K, end)`.
- `persistent_offset`
  - `diff[i]` is constant for all `i >= N`.
- `rate_divergence`
  - Not `transient`.
  - Not `persistent_offset`.
  - `|diff[i+1]| >= |diff[i]|` for all `i >= N`.
  - At least one strict increase exists (`|diff[i+1]| > |diff[i]|`).

If no rule matches, `artifact_diff.py` exits with `FAIL` and does not emit a
classification label.

Allowed output values are exactly:

```text
transient
persistent_offset
rate_divergence
```

## 3. Demo fixtures and provenance

Fixtures are deterministic synthetic artifacts generated from the Demo V2
canonical base artifact (`artifacts/demo_persistent/run_A.rpl`) into
`artifacts/demo_v3/` by:

```bash
python3 scripts/generate_demo_v3_fixtures.py
```

Demo V3 fixtures are expected to remain bit-identical across generator runs.
Repository CI includes a drift guard (`make fixture-drift-check`) that
regenerates fixtures and fails if `artifacts/demo_v3/` differs from committed
content.

Produced pairs:

- `transient_A.rpl`, `transient_B.rpl`
- `offset_A.rpl`, `offset_B.rpl`
- `rate_A.rpl`, `rate_B.rpl`

- transient pair: sample `+1` at frames `4096` and `4097`, reconverges at `4098`.
- persistent offset pair: sample `+1` for all frames `>= 4096`.
- rate pair: sample offset ramps `+1..+64` from frame `4096`, then stays at `+64`.

## 4. Example output

```bash
python3 scripts/artifact_diff.py artifacts/demo_v3/offset_A.rpl artifacts/demo_v3/offset_B.rpl
```

Expected key lines:

```text
First divergence frame: 4096
Classification: persistent_offset
```

## 5. Interpretation

- `transient`: short-lived perturbation with rapid reconvergence.
- `persistent_offset`: stable phase/offset shift that does not self-correct.
- `rate_divergence`: cumulative drift where mismatch shape changes over time.

These classes are deterministic artifact-derived diagnostics and do not alter
RPL0 format, capture path, or Demo V2 fixtures.

Demo V3 does not claim nondeterminism detection. It classifies deterministic
artifact divergence shape under explicit rules.

## 6. Demo V3 workflow targets

```bash
make demo-v3-verify
make demo-v3-audit-pack
make demo-v3-record
```
