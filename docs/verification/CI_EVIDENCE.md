# CI Evidence

This file is historical CI evidence only.
It is not the release contract, it is not the canonical release gate
definition, and it is not the canonical retained release record.
It remains retained historical verification material behind
[docs/verification/releases/index.md](releases/index.md).
Historical repository name/path references are preserved from the
pre-ejection repository identity (`precision-dpw`).

Use:

- [VERIFICATION_GUIDE.md](../VERIFICATION_GUIDE.md) for the release contract
- `make gate` for the canonical operator-facing release gate
- `docs/verification/releases/<version>/` for retained release evidence
- [docs/verification/releases/1.2.2/kani_evidence.md](releases/1.2.2/kani_evidence.md) for fresh local Kani evidence at `1.2.2`
- [docs/verification/releases/1.2.2/gate_full_evidence.md](releases/1.2.2/gate_full_evidence.md) for supplementary `--mode full` evidence at `1.2.2`

- Baseline: v1.2.0-rc1
- Workspace version: 1.2.0-rc1
- Lock version: 1.2.0-rc1

- Commit: 8c59b55aac18cf6396d300eca97b2490cc0540c8
- Date (UTC): 2026-03-18T05:09:22Z

- Workflow: CI
- Run ID: 23230032284
- Run URL: https://github.com/delk73/precision-dpw/actions/runs/23230032284
- Conclusion: success

- rustc: 1.91.1
- Targets: thumbv7em-none-eabihf

- Commands asserted by CI:
  - `make check-workspace`
  - `make test`
  - `make gate`

- precision validate --mode quick: PASSED (no drift)
