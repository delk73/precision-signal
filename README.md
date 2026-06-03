# precision-signal

Precision Signal is a deterministic execution validation system centered on
replay, operated through the `precision` CLI against an attached STM32 target
over UART.

## Current Surface

| Surface | Scope | Entry point |
| --- | --- | --- |
| Software validation | Host / software-only | `make gate` |
| Authoritative CLI tests | Host / software-only | `make authoritative-replay-cli-tests` |
| Retained release evidence | `1.9.1` | [docs/verification/releases/1.9.1/index.md](docs/verification/releases/1.9.1/index.md) |
| Hardware-backed validation | STM32F446 / ST-LINK / UART | `make bench-check`, `make fw-gate` |

## Evaluate 1.9.1

```bash
make gate
make authoritative-replay-cli-tests
make release-bundle-check VERSION=1.9.1
```

Hardware-backed validation is release authority only through the documented
[STM32F446/ST-LINK/UART firmware capture contract](docs/replay/FW_F446_CAPTURE_v1.md).
Other hardware, HIL, and observer artifacts are supplemental unless explicitly
named as release authority.

## Local Setup

```bash
rustup toolchain install 1.91.1
git clone https://github.com/delk73/precision-signal
cd precision-signal
make gate
```

## Reference Map

1. [docs/VERIFICATION_GUIDE.md](docs/VERIFICATION_GUIDE.md) — local validation,
   STM32 bench preflight, firmware gate, proof boundary, and release evidence
   routing.
2. [docs/RELEASE_SURFACE.md](docs/RELEASE_SURFACE.md) — released command
   surface and support/experimental boundaries.
3. [docs/authority/cli_contract.md](docs/authority/cli_contract.md) — command
   line interface definitions and grammars.

Core contracts and references:

- [docs/replay/FW_F446_CAPTURE_v1.md](docs/replay/FW_F446_CAPTURE_v1.md) —
  active firmware telemetry contract
- [docs/spec/rpl0_format_contract.md](docs/spec/rpl0_format_contract.md) —
  RPL0 serialization format
- [docs/MATH_CONTRACT.md](docs/MATH_CONTRACT.md) —
  fixed-point math and proof-boundary contract
- [docs/replay/DIVERGENCE_SEMANTICS.md](docs/replay/DIVERGENCE_SEMANTICS.md) —
  replay divergence model
- [docs/verification/releases/index.md](docs/verification/releases/index.md) —
  retained release evidence and release mechanics
- [docs/physical_characterization/PHYSICAL_CHARACTERIZATION.md](docs/physical_characterization/PHYSICAL_CHARACTERIZATION.md) —
  bench power, timing, and stability observations
- [docs/architecture/repository_mapping.md](docs/architecture/repository_mapping.md) —
  repository structure and implementation map

## License

MIT. See `LICENSE`.
