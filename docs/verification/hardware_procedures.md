# Hardware Procedures

This document collects manual hardware-oriented support procedures.

It is non-authoritative supporting material only. It does not define the
release surface, canonical verification policy, or release-scoped correctness
claims.

## Historical Manual Firmware Procedure

Retained from the prior ad hoc note labeled "One block phased":

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

# 3 run Phase D repeat capture
mkdir -p artifacts/phase_d
SERIAL=/dev/ttyACM0 python3 scripts/repeat_capture.py \
  --runs 3 \
  --out artifacts/phase_d
```
