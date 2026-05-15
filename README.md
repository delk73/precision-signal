# precision-signal

Precision Signal is a deterministic execution validation system centered on
replay, operated through the `precision` CLI against an attached STM32 target
over UART.

## Documentation

Start with the canonical documentation router:

- [docs/START_HERE.md](docs/START_HERE.md)

That router points to the active reading path and separates authority documents,
supporting documentation, retained evidence, and transient artifacts.

## Local Verification

```bash
rustup toolchain install 1.91.1
git clone https://github.com/delk73/precision-signal
cd precision-signal
make gate
```

## License

MIT. See `LICENSE`.
