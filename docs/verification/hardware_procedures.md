# Hardware Procedures

This document collects manual hardware-oriented support procedures.

It is non-authoritative supporting material only. It does not define the
release surface, canonical verification policy, or release-scoped correctness
claims.

## Manual Firmware Gate

Current validated firmware release gating is manual-reset,
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
make fw-release-archive VERSION=<release> SERIAL=/dev/ttyACM0
```

Expected success markers:
- listener reports `STATE,CAPTURE_DONE,138`
- captured file begins with `index,interval_us`
- `replay-host validate-interval-csv` reports success
- repeated CSV hashes are identical
- repeated imported artifact hashes are identical

Retained evidence written by the archive step:
- retained under the corresponding `docs/verification/releases/<release>/` bundle when the hardware archive workflow is executed

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

## Bounded STM32 Physical Replay Characterization

This documentation prepares bounded physical replay characterization for STM32
power-floor and degradation workflows. It does not expand the authoritative
release surface beyond the exercised `precision` CLI replay path.

Physical characterization evidence is supporting evidence unless explicitly
promoted by a later release packet. It does not by itself widen the
authoritative Precision Signal release surface.

Purpose: define how a future voltage / power-floor characterization run records
discrete observations without claiming a full power envelope. The authoritative
replay path remains `precision record`, `precision replay`, retained replay
artifacts, exact equivalence / first-divergence reporting, and bounded
STM32-over-UART operation.

Discrete sweep-point framing:
- treat each voltage or power condition as a separate observation point
- keep the target setup, firmware/build identity, host/tooling version, and
  replay artifact provenance attached to that point
- record capture and UART/log completeness independently from replay
  equivalence
- separate power provenance from execution provenance
- do not infer continuity between sweep points without later evidence and an
  explicit retained record

A voltage / power-floor characterization run may support bounded claims that:

- under a defined setup, replay behavior remained exact above an observed
  operating floor
- below or near that floor, failures were classified observationally
- failure observations were correlated with voltage / power conditions
- retained artifacts preserved provenance for those observations

Minimum provenance fields for future retained power-floor characterization
evidence:

- target identity
- target firmware/build identity
- host/tooling version
- `precision` version
- replay artifact path
- replay artifact hash
- capture artifact path/hash, if separate
- voltage or power condition
- power observation source:
  - `target_reported`
  - `independent_observer`
  - `operator_entered`
  - `unknown`
- result classification
- equivalence classification
- first divergence, if any
- capture completeness status
- UART/log completeness status
- operator notes, if any

Result classification vocabulary for future characterization runs:

- `exact_replay`
- `replay_divergence`
- `capture_incomplete`
- `uart_incomplete`
- `target_reset`
- `no_state_preamble`
- `non_start`
- `operator_invalid`
- `unknown`

Equivalence classification remains separate from this vocabulary and must use
the authoritative `precision` ResultBlock fields where available, including
`EQUIVALENCE` and `FIRST_DIVERGENCE`. Do not modify the authoritative
ResultBlock schema for characterization runs.

Self-board voltage / power-floor testing is useful for first-slice
characterization and local failure classification, but the target is not fully
authoritative for its own power trajectory near collapse.

Near the failure boundary:
- execution fidelity may degrade
- measurement fidelity may also degrade
- logs may truncate
- UART output may become incomplete
- reset behavior may erase or obscure the most interesting interval

Therefore:
- self-board evidence may classify observed replay / capture behavior
- self-board power readings must be treated as target-reported state
- an independent observer is preferred for authoritative power-trajectory
  evidence
- power provenance and execution provenance must remain separable
- retained evidence should distinguish target-reported state from independently
  observed rail state

This boundary does not block a first self-board characterization run. It blocks
overclaiming self-measured rail behavior as authoritative ground truth.

This work does not claim:
- full compute correctness across voltage
- complete power-envelope mapping
- arbitrary firmware correctness
- arbitrary hardware correctness
- universal signal validation
- firmware release readiness
- replay-host release readiness
- authoritative power trajectory from target self-measurement alone

Retained evidence generation is out of scope for this prep packet. A later
bounded procedure/evidence-capture packet must create any retained evidence and
show the command transcript, artifact hashes, provenance, and classification
records for the actual run.

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
