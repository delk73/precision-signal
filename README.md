# precision-signal

A deterministic execution validation system that captures runtime artifacts and verifies replay equivalence across contexts.

## Orientation Contract

`precision-signal` is deterministic execution analysis infrastructure for
capturing runtime artifacts and verifying replay equivalence across pinned
contexts.

### Stable Surface

| Path / Command | Role |
| --- | --- |
| `make gate` | canonical first verification path and canonical release gate |
| `make release-1.6.0` | canonical 1.6.0 retained-release orchestration after manual Kani preflight |
| [VERIFICATION_GUIDE.md](VERIFICATION_GUIDE.md) | release authority and verification contract |
| [docs/RELEASE_SURFACE.md](docs/RELEASE_SURFACE.md) | stable release-surface classification and routing |
| [docs/verification/releases/index.md](docs/verification/releases/index.md) | retained release evidence index |

Historical CLI reference material is retained under [docs/archive/cli/](docs/archive/cli/) for archival context only and is not part of the active CLI reading path.

### Experimental / Reference Surface

| Path | Role |
| --- | --- |
| [docs/README.md](docs/README.md) | descriptive documentation index, not the release contract |
| [docs/replay/tooling.md](docs/replay/tooling.md) | deeper replay-tooling boundary and operator-tool routing |
| [docs/wip/README.md](docs/wip/README.md) | exploratory notes and non-normative work in progress |

### Run This First

```bash
make gate
```

This is the canonical first-run verification path. Lower-level tooling commands,
including direct Python tool invocations, are deeper operator paths and should
not be mistaken for the top-level entry surface.

### Contribution Contract

- keep pull requests narrow and reviewable
- back claims with retained evidence
- do not widen the release surface casually
- treat workflow and release-path changes as governed changes

### Canonical References

- [VERIFICATION_GUIDE.md](VERIFICATION_GUIDE.md)
- [docs/RELEASE_SURFACE.md](docs/RELEASE_SURFACE.md)
- [docs/verification/releases/index.md](docs/verification/releases/index.md)

## First 5 Minutes

```bash
rustup toolchain install 1.91.1
git clone https://github.com/delk73/precision-signal
cd precision-signal
make gate
```

Expected:

```text
VERIFICATION PASSED
```

If this fails, see [VERIFICATION_GUIDE.md](VERIFICATION_GUIDE.md) → Failure Modes

## Status

- Proven replay pipeline evidence route: [docs/demos/demo_evidence_packaging.md](docs/demos/demo_evidence_packaging.md)
- Documentation index: [docs/README.md](docs/README.md)
- Release surface: [docs/RELEASE_SURFACE.md](docs/RELEASE_SURFACE.md)
- Replay tooling boundary: [docs/replay/tooling.md](docs/replay/tooling.md)
- Exploratory WIP notes: [docs/wip/README.md](docs/wip/README.md)

Workflow authority

- `make gate`: canonical release verification
- `make release-1.6.0`: canonical 1.6.0 release-record orchestration after `bash verify_kani.sh`
- `make ci-local`: aggregate developer pre-merge check

These serve different purposes and are not interchangeable.

`make ci-local` is a convenience aggregate for local confidence before merge.
As implemented, it includes documentation checks, firmware build validation,
workspace tests, replay tooling tests, canonical `make gate`, and fixture drift
checks. It does not require attached hardware, so it is CI-safe in a provisioned
toolchain environment, but it is not release authority and passing it does not
by itself imply release readiness.

Completed Phase 1 through Phase 5 replay evidence is packaged through one proof
path: `make demo-evidence-package`, with the retained proof bundle under
`artifacts/demo_evidence/retained/`.

Operator-facing released tooling remains the broader Python replay toolchain
plus the `precision` validation CLI surface classified in
[docs/RELEASE_SURFACE.md](docs/RELEASE_SURFACE.md). Release `1.6.0` retains one
bounded Rust replay exception: `replay-host diff` for the retained
`artifacts/rpl0/` proof corpus only, with retained transcripts under
[docs/verification/releases/1.6.0/](docs/verification/releases/1.6.0/).
Broader Rust replay, including the remaining `replay-host` commands and the
STM32 replay firmware path, remains experimental and is not promoted by that
proof bundle.

## Release 1.6.0 Procedure

- `cargo clean`
- `bash verify_kani.sh`
- `make release-1.6.0`
- inspect [docs/verification/releases/1.6.0/](docs/verification/releases/1.6.0/)
- `git add . && git commit -m "release(1.6.0): Frozen verification record"`
- final claim/evidence sweep: [README.md](README.md), [docs/RELEASE_SURFACE.md](docs/RELEASE_SURFACE.md), [CHANGELOG.md](CHANGELOG.md), [docs/demos/demo_evidence_packaging.md](docs/demos/demo_evidence_packaging.md)

## License

MIT. See `LICENSE`.
