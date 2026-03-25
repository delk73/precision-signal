# CI Evidence

This file is historical CI evidence only.
It is not the release contract, it is not the canonical release gate
definition, and it is not the canonical retained release record.
Historical repository name/path references are preserved from the
pre-ejection repository identity (`precision-dpw`).

Use:

- [VERIFICATION_GUIDE.md](../../VERIFICATION_GUIDE.md) for the release contract
- `make gate` for the canonical operator-facing release gate
- `docs/verification/releases/<version>/` for retained release evidence

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
