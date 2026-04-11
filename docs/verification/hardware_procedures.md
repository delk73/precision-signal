# Hardware Procedures

This document collects manual hardware-oriented support procedures.

It is non-authoritative supporting material only. It does not define the
release surface, canonical verification policy, or release-scoped correctness
claims.

## Active 1.6.0 Firmware Gate

Current validated `1.6.0` firmware release gating is manual-reset,
human-in-the-loop; automated reset is not the validated release path unless it
is separately proven and recorded.

Authority split for the active path:
- `scripts/csv_capture.py` is transport only
- `replay-host validate-interval-csv` is the CSV validation authority
- `replay-host import-interval-csv` is the artifact import authority

Canonical operator sequence:

```bash
make fw-capture-check SERIAL=/dev/ttyACM0
make fw-repeat-check SERIAL=/dev/ttyACM0 REPLAY_REPEAT_RUNS=3
make firmware-release-check SERIAL=/dev/ttyACM0
make fw-release-archive VERSION=1.6.0 SERIAL=/dev/ttyACM0
```

Expected success markers:
- listener reports `STATE,CAPTURE_DONE,138`
- captured file begins with `index,interval_us`
- `replay-host validate-interval-csv` reports success
- repeated CSV hashes are identical
- repeated imported artifact hashes are identical

Retained evidence written by the archive step:
- `docs/verification/releases/1.6.0/fw_capture/`
- `docs/verification/releases/1.6.0/fw_repeat/`
- `docs/verification/releases/1.6.0/firmware_release_evidence.md`

## Active Single-Run Capture

Validated manual-reset capture example:

```bash
python3 scripts/csv_capture.py --serial /dev/ttyACM0 --out observed.csv --reset-mode manual
cargo run -q -p replay-host -- validate-interval-csv observed.csv
cargo run -q -p replay-host -- import-interval-csv observed.csv observed.rpl
```

Operational notes:
- press hardware reset once after listener readiness
- ensure the physical loopback is installed from `PA6` (`TIM3_CH1`) to `PA0` (`TIM2_CH1`)
- do not mix active debugger attachment with UART capture
- treat missing UART emission, malformed `STATE,...`, or validator/import failure as hard failure
- treat `--reset-mode stlink` as convenience tooling only, not the validated release gate path

## Historical Manual Firmware Procedure

Retained historical `RPL0` capture procedure:

```bash
# 0 record repo state
git status --short
git rev-parse HEAD

# 1 build canonical firmware
cargo build -p replay-fw-f446 --target thumbv7em-none-eabihf --locked
cargo objcopy -p replay-fw-f446 --target thumbv7em-none-eabihf -- -O binary \
  target/thumbv7em-none-eabihf/debug/replay-fw-f446.bin

# 2 flash canonical firmware
st-flash --connect-under-reset --reset write \
  target/thumbv7em-none-eabihf/debug/replay-fw-f446.bin 0x08000000

# 3 run retained historical RPL0 repeat capture
mkdir -p artifacts/phase_d
SERIAL=/dev/ttyACM0 python3 scripts/repeat_capture.py \
  --contract rpl0 \
  --runs 3 \
  --signal-model phase8 \
  --artifacts-dir artifacts/phase_d
```

This retained path is not part of the active default firmware gate.
