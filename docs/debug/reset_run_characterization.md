# Reset / Run Characterization (STM32F446)

## Scope
This runbook defines the known-good reset/attach model for replay capture on STM32F446.
It is operational guidance for board bring-up, flash/debug/capture sequencing, and recovery from bad states.

## 1. Reset Classes

### A. Hardware reset button
- What it does: reboots the MCU and starts firmware from reset vector without changing USB cabling.
- When to use: validated UART capture runs for the self-stimulus flow.
- Expected behavior: one `STATE,...` line is emitted first; successful runs emit `STATE,CAPTURE_DONE,138` followed by CSV.

### B. ST-LINK / `st-flash --reset`
- What it does: programs flash and requests target reset at the end of write.
- When to use: firmware programming path (`flash-ur`) and readback compare path (`flash-compare-ur`).
- Known caveats:
  - If another process holds ST-LINK, flash/readback fails.
  - If attach state is bad, `connect-under-reset` may still require the observed recovery sequence below.
  - `--reset-mode stlink` is present in tooling, but not validated as reliable for UART capture in this session.

### C. `connect-under-reset` / `st-util`
- What it does: attaches debugger while reset is asserted to recover from unstable target states.
- When to use: GDB sessions and low-level register inspection.
- Known caveats:
  - stale `st-util` instances can block new sessions.
  - session can become unreliable if transport is interrupted (for example aggressive Ctrl-C use in unstable sessions).

### D. Full power-cycle
- When it is necessary: board is in persistent bad attach/reset state after normal reset and reflash attempts.
- Symptoms that require it:
  - repeated ST-LINK attach/read errors after process cleanup
  - repeated nonsense memory/register reads
  - UART listener never receives a new stream despite proper operator sequence

## 2. Known-Good Procedures

### A. Validated flash and verify path
Validated operator path:

```bash
make flash-ur
make flash-compare-ur
```

Expected outcome:
- flash write succeeds
- flash readback compare succeeds (device image equals local bin)

### B. Validated self-stimulus capture run
Operator sequence:
1. Run `make flash-ur`.
2. Run `make flash-compare-ur`.
3. Start the listener:
   `python3 scripts/csv_capture.py --serial /dev/ttyACM0 --out observed.csv --reset-mode manual`
4. Press hardware reset button once.
5. Read the first emitted line.
6. On success, wait for the CSV to complete.

Operational notes:
- one reset press per capture run
- do not keep active debug server attached during UART capture
- manual reset is canonical for UART capture on this flow
- success is indicated first by `STATE,CAPTURE_DONE,138`, followed by CSV with header `index,interval_us` and 138 rows
- `STATE,CAPTURE_INCOMPLETE,<N>` is an explicit failure diagnostic; treat it as capture not fully completed
- In this characterization, `--reset-mode stlink` did not produce a `STATE,...` line; treat it as not validated for UART capture on this flow.

### C. GDB debug session
Operator sequence:
1. Kill stale debug servers: `killall -q st-util || pkill -x st-util || true`.
2. Start debug session (`make debug-session` or equivalent st-util/GDB workflow).
3. Set breakpoint at the intended executable line (post-init location for TIM2 checks).
4. Avoid unnecessary Ctrl-C in known unstable transport states.

## 3. Failure Signatures

### A. USB claim failure
Symptom:
- `Stlink usb device found, but unable to claim`

Meaning:
- another process already has the ST-LINK device open.

### B. Invalid GDB memory reads / open-bus style session
Symptoms:
- repeated nonsensical register values
- memory reads not matching expected runtime state

Meaning:
- attach/reset state is invalid; debugger view is not trustworthy.

### C. Wrong breakpoint line in script
Symptoms:
- dump captured before `init_tim2_1khz()`
- TIM2 registers all zero or `ARR=0xffffffff`

Meaning:
- breakpoint hit too early; peripheral init has not occurred yet.

### D. UART script open/read failure
Symptoms:
- `device reports readiness to read but returned no data`
- `Failed: serial open/read error: ...`

Meaning:
- serial session is unhealthy or device/port state is bad.

### E. Listener receives no `STATE,...` line
Symptom:
- listener prints no `STATE,...` line for the run.

Meaning:
- board was not reset for a new run, firmware did not begin a new stream, or the selected reset mode is not producing a validated capture start.

### F. Explicit incomplete capture state
Symptoms:
- first line is `STATE,CAPTURE_INCOMPLETE,0`
- first line is `STATE,CAPTURE_INCOMPLETE,<N>` where `0 < N < 138`

Meaning:
- `0`: A0 capture did not start
- `0 < N < 138`: capture started but did not complete

### G. ST-LINK reset mode produces no `STATE,...` line
Symptom:
- `python3 scripts/csv_capture.py ... --reset-mode stlink` does not print a `STATE,...` line in this board/session

Meaning:
- ST-LINK auto-reset is not validated as reliable for UART capture on this flow
- use the validated manual reset path instead

## 4. Recovery Procedures

### A. ST-LINK held by stale process
1. `killall -q st-util || pkill -x st-util || true`
2. Retry:
```bash
make flash-ur
make flash-compare-ur
```

### B. Bad UART session
1. Stop the listener.
2. Restart listener command.
3. Press hardware reset once.
4. If still bad, unplug/replug USB CDC connection and retry.

### C. Board in bad attach/reset state
Observed recovery sequence for bad attach/reset state:
1. Kill stale `st-util`:
   `killall -q st-util || pkill -x st-util || true`
2. Run `make flash-ur` while holding RESET and allow it to fail.
3. Release RESET.
4. Rerun `make flash-ur`.
5. Run `make flash-compare-ur`.

This is a recovery procedure for bad attach/reset state, not the normal happy-path flash flow.

If the recovery sequence does not restore attach/reset behavior:
1. Full power-cycle board.
2. Re-run:
```bash
make flash-ur
make flash-compare-ur
```
3. Retry UART run or debug session.

### D. Wrong breakpoint line / early capture
1. Move breakpoint to post-`init_tim2_1khz()` executable line.
2. Restart debug session cleanly.
3. Re-check register dump.

## 5. Preferred Operational Rules (Operator Contract)
- Use hardware reset button for UART capture runs on this self-stimulus flow.
- Use `flash-ur` and `flash-compare-ur` for programming/verification.
- Treat manual reset as the validated operator path.
- Treat `STATE,CAPTURE_DONE,138` as capture complete.
- Treat `STATE,CAPTURE_INCOMPLETE,<N>` as explicit capture failure.
- Treat `--reset-mode stlink` as convenience tooling, not a validated UART capture path in this session.
- Use `tim2-smoke` only for debug-focused sessions.
- Do not mix active `st-util` debugging with UART capture workflow.
- Kill stale `st-util` before new debug/flash attempts.
- Avoid Ctrl-C in unstable GDB sessions unless required.
- If recovery loops repeat, power-cycle first, then reflash under reset.
- `debug-irq-count` is diagnostic-only and not part of the canonical release/capture path.

## Quick Reference

### Program + verify
```bash
make flash-ur
make flash-compare-ur
```

### Capture one validated self-stimulus run
```bash
python3 scripts/csv_capture.py --serial /dev/ttyACM0 --out observed.csv --reset-mode manual
```
Then press reset once.

Expected first line:
- `STATE,CAPTURE_DONE,138` on success
- `STATE,CAPTURE_INCOMPLETE,<N>` on explicit capture failure

### Diagnostic-only flash path
```bash
make FW_FEATURES=debug-irq-count flash-ur
make FW_FEATURES=debug-irq-count flash-compare-ur
```
