# STM32 Interval Capture Contract v1

**Document revision:** 1.2.2  
**Status:** NORMATIVE  
**Scope:** STM32F446 self-stimulus interval capture emitted by `replay-fw-f446`,
captured by `scripts/csv_capture.py`, and accepted by `replay-host validate-interval-csv`
and `replay-host import-interval-csv`.

## Purpose

This document freezes the canonical upstream input surface for the validated
STM32 self-stimulus capture path. Phase 2 may consume only CSV captures that
conform to this contract.

## Capture Boundary

The UART stream for a successful run is:

1. One state line: `STATE,CAPTURE_DONE,138`
2. One CSV file body with header `index,interval_us`
3. Exactly 138 data rows

`STATE,CAPTURE_INCOMPLETE,<N>` is an explicit failed capture and is not valid
downstream input.

The canonical retained file surface is the CSV body written by
`scripts/csv_capture.py` after the successful state line is observed.
The `STATE,...` line is transport metadata and is not part of the canonical CSV
file itself.

## Canonical Record Form

Header:

```text
index,interval_us
```

Data row:

```text
<index>,<interval_us>
```

Example first rows from a retained valid capture:

```text
index,interval_us
0,305564
1,304000
2,304000
```

## Canonical Field Order

1. `index`
2. `interval_us`

No extra columns are permitted.

## Units and Representation

- Encoding: UTF-8 text, one record per line
- Delimiter: comma
- Header text: exact literal `index,interval_us`
- `index`: base-10 unsigned integer
- `interval_us`: base-10 unsigned integer representing microseconds
- Row count: exactly 138 interval rows
- Index domain: exact contiguous sequence `0..137`

## Required Invariants

- Successful capture is identified by the preamble `STATE,CAPTURE_DONE,138`
  before any CSV rows are accepted.
- The retained CSV file begins with the exact header `index,interval_us`.
- Every data row has exactly two fields.
- `index` for row `n` equals `n`, with no gaps, duplicates, or reordering.
- `interval_us` parses as `u32`.
- `interval_us > 0` for every retained row.
- The file contains exactly 138 interval rows, matching the firmware
  `INTERVAL_COUNT`.

## Rejection Conditions

`replay-host validate-interval-csv` and `replay-host import-interval-csv`
reject the file if any of the following hold:

- missing or non-exact header
- empty file
- empty row
- row with fewer or more than two columns
- non-numeric `index`
- non-contiguous or out-of-order `index`
- non-numeric `interval_us`
- `interval_us = 0`
- row count other than 138

## Downstream Consumer Boundary

Downstream import is frozen to this rule:

- `replay-host import-interval-csv` may consume only files that first pass
  `replay-host validate-interval-csv`
- `replay-host validate-interval-csv` defines the accepted schema boundary for
  this import path
- downstream consumers must not widen, reinterpret, or silently accept variant
  input outside that validator-defined contract

No shorter CSV, padded CSV, alternate header, alternate column order, or
failure-state capture is part of the canonical boundary.

## Retained Evidence

- Sample files:
  - `artifacts/capture_contract_v1/run_20260331T150000Z.csv`
  - `artifacts/capture_contract_v1/run_20260331T160000Z.csv`
- Hash manifest:
  - `artifacts/capture_contract_v1/manifest.txt`
- Validation log:
  - `artifacts/capture_contract_v1/validation.txt`
