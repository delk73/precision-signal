# RPL0 Model Live Smoke

- timestamp: `20260515-081635`
- serial: `/dev/ttyACM0`

| Mode | Feature | Artifact | Verify | Replay self-diff | Independent diff | Hash | SHA256 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `phase8` | `default` | `docs/verification/artifacts/model_validation/2026-05-15-rpl0-multi-model/phase8.bin` | PASS | PASS | SKIPPED | PASS | `f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765` |
| `burst8` | `signal-model-burst8` | `docs/verification/artifacts/model_validation/2026-05-15-rpl0-multi-model/burst8.bin` | PASS | PASS | SKIPPED | PASS | `45aadedfddc080b37786d767da139f53f11966d64246ec0283fa6bfe9e765dc2` |
| `seeded_lfsr8` | `signal-model-seeded-lfsr8` | `docs/verification/artifacts/model_validation/2026-05-15-rpl0-multi-model/seeded_lfsr8.bin` | PASS | PASS | SKIPPED | PASS | `bb35e5647b5b982b7a58884708286b7b681371c86be4f9bba14fe025cdf0b2ee` |

## Command Results

| Mode | Build | Flash | Flash compare | Capture | Verify | Replay self-diff | Independent diff | Hash |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `phase8` | PASS | PASS | PASS | PASS | PASS | PASS | SKIPPED | PASS |
| `burst8` | PASS | PASS | PASS | PASS | PASS | PASS | SKIPPED | PASS |
| `seeded_lfsr8` | PASS | PASS | PASS | PASS | PASS | PASS | SKIPPED | PASS |
