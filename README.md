# precision-signal

A deterministic execution validation system that captures runtime artifacts and verifies replay equivalence across contexts.

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

Completed Phase 1 through Phase 5 replay evidence is packaged through one proof
path: `make demo-evidence-package`, with the retained proof bundle under
`artifacts/demo_evidence/retained/`.

Operator-facing released tooling remains the Python replay toolchain plus the
`precision` validation CLI surface classified in
[docs/RELEASE_SURFACE.md](docs/RELEASE_SURFACE.md). Experimental components,
including `replay-host` and the STM32 replay firmware path, are not promoted by
that proof bundle.

## License

MIT. See `LICENSE`.
