# precision-signal

A deterministic execution validation system that captures runtime artifacts and verifies replay equivalence across contexts.

## Quick Start

```bash
rustup toolchain install 1.91.1
git clone https://github.com/delk73/precision-signal
cd precision-signal
make gate
```

Runs the canonical release verification gate.

Use these documents as the authoritative entry points:

| Path | Role |
| --- | --- |
| [VERIFICATION_GUIDE.md](VERIFICATION_GUIDE.md) | release-readiness authority and verification contract |
| [docs/RELEASE_SURFACE.md](docs/RELEASE_SURFACE.md) | release-surface classification and routing |
| [docs/verification/releases/index.md](docs/verification/releases/index.md) | retained release evidence index |

Historical CLI reference material is retained under [docs/archive/cli/](docs/archive/cli/) for archival context only and is not part of the active CLI reading path.

For deeper documentation, see [docs/README.md](docs/README.md), [docs/replay/tooling.md](docs/replay/tooling.md), and [docs/wip/README.md](docs/wip/README.md).

## License

MIT. See `LICENSE`.
