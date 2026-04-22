# Replay Stability Envelope — STM32

## Objective

* Create and maintain a live working document that records the replay stability envelope experiment across 5 cycles, without altering prior validated baseline docs.

## Grounding

* Project: Precision Signal
* Core capability: deterministic execution validation via replay
* Authoritative interface: `precision` CLI
* Canonical path: STM32 over UART with PA6 -> PA0 loopback
* Current phase: stability measurement of an already validated replay path
* Prior validation doc is complete and must not be mutated

## Execution Protocol

* Scope:

  * Same STM32 target
  * Same validated wiring and loopback
  * Same flash/reset procedure
  * Repeated capture -> record -> replay cycles
  * Evidence collection only
* Constraints:

  * No code changes
  * No contract changes
  * No refactors
  * No hardware changes once the run set begins
  * Write observations only from recorded outputs
  * Do not summarize or compress ResultBlocks
  * Document must be append-only during run
* Cycle procedure:

  1. Capture
  2. Record
  3. Replay
  4. Log results exactly as observed
* Stop condition:

  * Stop immediately on a hard failure
  * Record the exact failure signature

## Capture Invariants (all cycles)

* status: `STATE,CAPTURE_DONE,138`
* sha256: `b6243038335e15871ff9750b0b1bfa0cc75cc03639fe25d0cd22644abeff43a1`

## Cycle Log

### Cycle 01

* Record ResultBlock:

```text
RESULT: PASS
COMMAND: record
TARGET: artifacts/stability_envelope/20260420T224801Z/cycle_01/run_01.csv
MODE: runtime_mode
EQUIVALENCE: exact
FIRST_DIVERGENCE: none
ARTIFACT: artifacts/20260420T224805Z-4196125988a4c8fd
```

* Replay ResultBlock:

```text
RESULT: PASS
COMMAND: replay
TARGET: artifacts/20260420T224805Z-4196125988a4c8fd
MODE: runtime_mode
EQUIVALENCE: exact
FIRST_DIVERGENCE: none
ARTIFACT: artifacts/20260420T224806Z-a72394bd64079308
```

### Cycle 02

* Record ResultBlock:

```text
RESULT: PASS
COMMAND: record
TARGET: artifacts/stability_envelope/20260420T224801Z/cycle_02/run_01.csv
MODE: runtime_mode
EQUIVALENCE: exact
FIRST_DIVERGENCE: none
ARTIFACT: artifacts/20260420T224810Z-6d6c2fb955185ac5
```

* Replay ResultBlock:

```text
RESULT: PASS
COMMAND: replay
TARGET: artifacts/20260420T224810Z-6d6c2fb955185ac5
MODE: runtime_mode
EQUIVALENCE: exact
FIRST_DIVERGENCE: none
ARTIFACT: artifacts/20260420T224810Z-9ba9c695f2ffaa54
```

### Cycle 03

* Record ResultBlock:

```text
RESULT: PASS
COMMAND: record
TARGET: artifacts/stability_envelope/20260420T224801Z/cycle_03/run_01.csv
MODE: runtime_mode
EQUIVALENCE: exact
FIRST_DIVERGENCE: none
ARTIFACT: artifacts/20260420T224814Z-3c39b479c82a2f8d
```

* Replay ResultBlock:

```text
RESULT: PASS
COMMAND: replay
TARGET: artifacts/20260420T224814Z-3c39b479c82a2f8d
MODE: runtime_mode
EQUIVALENCE: exact
FIRST_DIVERGENCE: none
ARTIFACT: artifacts/20260420T224815Z-08bc1edcd592cc1b
```

### Cycle 04

* Record ResultBlock:

```text
RESULT: PASS
COMMAND: record
TARGET: artifacts/stability_envelope/20260420T224801Z/cycle_04/run_01.csv
MODE: runtime_mode
EQUIVALENCE: exact
FIRST_DIVERGENCE: none
ARTIFACT: artifacts/20260420T224819Z-25e02cca1f23f693
```

* Replay ResultBlock:

```text
RESULT: PASS
COMMAND: replay
TARGET: artifacts/20260420T224819Z-25e02cca1f23f693
MODE: runtime_mode
EQUIVALENCE: exact
FIRST_DIVERGENCE: none
ARTIFACT: artifacts/20260420T224819Z-7b8c91140b0e9d89
```

### Cycle 05

* Record ResultBlock:

```text
RESULT: PASS
COMMAND: record
TARGET: artifacts/stability_envelope/20260420T224801Z/cycle_05/run_01.csv
MODE: runtime_mode
EQUIVALENCE: exact
FIRST_DIVERGENCE: none
ARTIFACT: artifacts/20260420T224823Z-80e8a72ca1b322e3
```

* Replay ResultBlock:

```text
RESULT: PASS
COMMAND: replay
TARGET: artifacts/20260420T224823Z-80e8a72ca1b322e3
MODE: runtime_mode
EQUIVALENCE: exact
FIRST_DIVERGENCE: none
ARTIFACT: artifacts/20260420T224824Z-f8499d4b91209677
```

## Observations

* All 5 cycles completed.
* Capture invariants held across all 5 cycles (status and sha256 unchanged).
* All `record` ResultBlocks report:

  * `RESULT: PASS`
  * `MODE: runtime_mode`
  * `EQUIVALENCE: exact`
  * `FIRST_DIVERGENCE: none`
* All `replay` ResultBlocks report:

  * `RESULT: PASS`
  * `MODE: runtime_mode`
  * `EQUIVALENCE: exact`
  * `FIRST_DIVERGENCE: none`
* No divergence was observed in any completed cycle.
* Session manifest reports `PASS` for cycles 01 through 05 with failure class `-`.

## Session Summary

* session_id: `20260420T224801Z`
* session_root: `artifacts/stability_envelope/20260420T224801Z`
* status: `PASS`
* cycles_requested: `5`
* cycles_completed: `5`
* failure_cycle: `-`
* failure_class: `-`

## Validation Criteria

* Document reflects all completed cycles 01 through 05.
* ResultBlocks are preserved exactly as recorded.
* Evidence is derived only from recorded outputs under `artifacts/stability_envelope/20260420T224801Z`.
* No prior baseline validation doc was mutated.

## Final Classification

* Replay stability envelope session `20260420T224801Z` completed with `PASS`.
* Under the tested path, replay remained exact across 5/5 capture -> record -> replay cycles with no observed divergence.

## Notes

* This document records the completed stability envelope experiment only.
* Scope is limited to the tested configuration:

  * STM32 over UART
  * PA6 -> PA0 loopback
  * ST-LINK reset
  * single CSV capture per cycle
  * 5 completed cycles
* Per-cycle raw capture and stdout artifacts are not retained in-repo; evidence is represented via `manifest.tsv` and ResultBlocks.
* This evidence should not be read as a broader claim beyond the exercised path and recorded artifacts.
