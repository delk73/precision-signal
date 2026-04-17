# precision-signal

Precision Signal is a deterministic execution validation system centered on replay, operated through the `precision` CLI against an attached STM32 target over UART.

**Naming**
- Product name: **Precision Signal**
- Authoritative CLI command: `precision`
- Canonical hardware path: attached STM32 target over UART

## Quick Start

```bash
rustup toolchain install 1.91.1
git clone https://github.com/delk73/precision-signal
cd precision-signal
make gate
```

Runs the canonical release verification gate.

## Active Authority Path

Use these documents in this order:

| Path | Role |
| --- | --- |
| [VERIFICATION_GUIDE.md](VERIFICATION_GUIDE.md) | release-readiness authority and verification contract |
| [docs/RELEASE_SURFACE.md](docs/RELEASE_SURFACE.md) | release-surface classification and routing |
| [docs/authority/cli_contract.md](docs/authority/cli_contract.md) | sole active CLI contract authority |
| [docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md](docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md) | sole active STM32 capture contract authority |

## Historical / Release-Record Landing

| Path | Role |
| --- | --- |
| [docs/verification/releases/index.md](docs/verification/releases/index.md) | retained release records and historical verification landing |

## Descriptive Index

For descriptive and supporting documentation behind the active authority path, use [docs/README.md](docs/README.md).
Experimental notes remain under [docs/wip/README.md](docs/wip/README.md) and are non-normative.

## License

MIT. See `LICENSE`.
