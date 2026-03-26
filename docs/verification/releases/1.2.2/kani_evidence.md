# Kani Verification Evidence (1.2.2)

This file is the retained formal-verification record for release `1.2.2`.

- `EXECUTED_AT_UTC=2026-03-26T00:19:09Z`
- `COMMAND=bash verify_kani.sh`
- `WORKSPACE_VERSION=1.2.2`
- `HEAD=90bd390`
- `rustc=1.91.1 (ed61e7d7e 2025-11-07)`
- `cargo=1.91.1 (ea2d97820 2025-10-10)`
- `cargo_kani=0.67.0`
- `RUN_HEAVY=0`
- `KEEP_LOGS=0`
- `RESULT=PASS`

## Scope

- Tier-1 harnesses executed: `23`
- Tier-2 harnesses skipped by default runner mode: `5`
- Total manifest-defined harnesses: `28`
- Expected Tier-1 harness set ran to completion: `yes`

## Result Summary

- `bash verify_kani.sh` completed successfully in Tier-1 mode.
- The runner reported `Kani verification complete (470s total)`.
- No Tier-1 proof failures were reported.
- Some harnesses emitted Kani unsupported-construct warnings during setup; verification still completed successfully and no reachable unsupported construct caused proof failure.

## Proof-Scope Note

This retained evidence covers the canonical Tier-1 runner surface only. Heavy Tier-2 harnesses remain available via `RUN_HEAVY=1 bash verify_kani.sh` and were not required for the canonical `1.2.2` release gate refresh.
