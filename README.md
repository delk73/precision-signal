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

- Documentation index: [docs/README.md](docs/README.md)
- Release surface: [docs/RELEASE_SURFACE.md](docs/RELEASE_SURFACE.md)
- Replay tooling boundary: [docs/replay/tooling.md](docs/replay/tooling.md)

## License

MIT. See `LICENSE`.
