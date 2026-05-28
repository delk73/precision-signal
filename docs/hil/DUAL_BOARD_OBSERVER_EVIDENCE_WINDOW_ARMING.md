# Dual-Board Observer Evidence-Window Arming Design

Status: implemented for dual-board observer timing diagnostics. The design keeps
raw full-run `result` separate from `evidence_window_result`.

## Current Evidence

Retained and scratch dual-board observer runs show stable paired timing:

```text
max_delta_ticks=6 <= threshold_ticks=9
max_delta_ns=66
```

The current failure mode is event-stream cleanliness, not paired latency.
Run `0005` located the repeatable in-window unexpected acknowledgment at:

```text
pre_first_trigger_ack_count=1
in_window_unexpected_ack_count=1
first_in_window_unexpected_ack_trigger_count=5
last_in_window_unexpected_ack_trigger_count=5
post_final_trigger_ack_count=0
```

Interpretation: the unexpected acknowledgment behavior is early-run and
startup/arming-adjacent. Current evidence does not implicate steady-state
midstream trigger-to-ack latency or post-final shutdown/finalization behavior.

## Decision Record

- Raw full-run result policy: keep `result` as the raw full-run result. Any
  nonzero `unexpected_ack_count` keeps raw `result=FAIL`.
- Evidence-window result policy: add a separate `evidence_window_result` for
  evidence-window-only PASS/FAIL. Do not silently reinterpret existing `result`.
- Arming boundary definition: use a named trigger-count boundary,
  `SYNC_TIMING_EVIDENCE_WINDOW_START_TRIGGER = 8`. The startup window is
  classified as accepted `trigger_count < 8`; the evidence window starts at accepted
  `trigger_count >= 8`.
- Evidence-window size: require exactly 10,000 evidence-window trigger/ack
  pairs for evidence-window PASS. Because startup triggers are excluded from
  evidence, implementation must either have the actor emit startup plus
  evidence triggers or have the observer continue until it captures 10,000
  evidence-window triggers.
- Evidence scope: this remains external-observer evidence only. It does not
  claim actor internal silicon latency, software EXTI timing, platform proof,
  RPL0/replay authority, or release evidence.

## Report Model

Existing raw-observation counters remain visible and continue to describe the
whole observed run:

```text
ack_count
unexpected_ack_count
pre_first_trigger_ack_count
in_window_unexpected_ack_count
first_in_window_unexpected_ack_trigger_count
last_in_window_unexpected_ack_trigger_count
post_final_trigger_ack_count
```

Implemented evidence-window fields:

```text
evidence_window_start_trigger_count=
evidence_window_trigger_count=
evidence_window_ack_count=
evidence_window_unexpected_ack_count=
evidence_window_missed_ack_count=
evidence_window_capture_error_count=
evidence_window_max_delta_ticks=
evidence_window_max_delta_ns=
evidence_window_result=
```

Optional startup fields were considered but are not part of the implemented
report:

```text
startup_ack_count=
startup_trigger_count=
startup_observation_end_trigger_count=
startup_classification=
```

## PASS/FAIL Semantics

Full-run `result` remains the raw full-run result. Startup observations can make
the raw result fail, even if the evidence window passes.

Strict evidence-window PASS requires:

```text
evidence_window_trigger_count=10000
evidence_window_ack_count=10000
evidence_window_missed_ack_count=0
evidence_window_unexpected_ack_count=0
evidence_window_capture_error_count=0
evidence_window_max_delta_ticks<=threshold_ticks
evidence_window_result=PASS
```

A PASS evidence-window result means the observer measured a clean trigger/ack
stream inside the declared evidence window. It does not mean no startup
transient occurred before the evidence window.

## Forbidden Behavior

Do not:

```text
drop early ack edges silently
convert unexpected_ack_count into PASS
hide startup behavior from reports
weaken the generated artifact claim boundary
claim replay authority, release evidence, or platform proof
```

## Implementation Scope

The implementation is limited to:

- Firmware accounting for startup and evidence-window counters.
- Host parser and metadata retention for the new fields.
- Tests for raw FAIL with evidence-window PASS, raw unexpected ack inside the
  evidence window, threshold failure, evidence-window trigger-count failure, and
  missing evidence-window fields.
- Documentation updates explaining that raw full-run counters show everything
  observed, startup counters classify pre-evidence behavior, and evidence-window
  counters control only `evidence_window_result`.

The implementation does not change runner orchestration, generated-file policy,
board alias validation, timing threshold, or external-observer claim boundary.
