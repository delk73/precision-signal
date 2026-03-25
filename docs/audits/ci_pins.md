# CI Pin Registry

This file records immutable CI runner/action pins used by workflow policy checks.
The registry is descriptive; `scripts/check_ci_pins.sh` is the enforcement oracle.
Pin updates must update both this file and the script expectations in one change.

## Pinned Values

- Runner image: `ubuntu-24.04`
- `actions/checkout`: `692973e3d937129bcbf40652eb9f2f61becf3332`
- `dtolnay/rust-toolchain`: `0f44b27771c32bda9f458f75a1e241b09791b331`
- `Swatinem/rust-cache`: `779680da715d629ac1d338a641029a2f4372abb5`

## Machine Check

Run:

```bash
bash scripts/check_ci_pins.sh
```

Expected output:

```text
CI pin check OK
```
