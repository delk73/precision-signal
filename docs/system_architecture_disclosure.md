# System Architecture Disclosure

## Overview

This document records the architecture implemented in the
`precision-signal` repository.

Use [docs/RELEASE_SURFACE.md](RELEASE_SURFACE.md) for release classification,
[VERIFICATION_GUIDE.md](./VERIFICATION_GUIDE.md) for verification authority, and
[docs/replay/tooling.md](replay/tooling.md) for replay-tooling boundaries.

Precision Signal is a deterministic execution validation system centered on replay, operated through the `precision` CLI against an attached STM32 target over UART.

The system captures execution artifacts, analyzes divergence between independent executions, and provides a replay-tooling layer that includes historically released Python support tooling retained as support/reference material.

## Core Concepts

Artifact Capture

Execution artifacts are recorded from running systems
using a canonical binary format.

Deterministic Replay

The Python tooling layer (`scripts/artifact_tool.py`
and `scripts/artifact_diff.py`) remains historically
released replay-facing support tooling and is not part of the canonical
surface or active operator authority. Rust replay
(`crates/replay-host`) is experimental and limited to
legacy-frame replay boundaries documented in
[docs/replay/tooling.md](replay/tooling.md).

Divergence Localization

Two executions can be compared by replaying their artifacts
and identifying the first divergence point.

Artifact Identity Hashing

Artifacts produce deterministic hash streams that enable
comparison and reproducibility checks.

Verification Infrastructure

The repository includes verification gates combining
hash-locked outputs, formal verification harnesses,
and deterministic build enforcement.

## Architecture Flow

Execution
→ Artifact Encoding
→ Replay
→ Divergence Analysis

Artifacts encode execution events in a canonical format.
Historically released Python tooling provides artifact
inspection, verification, hashing, compare workflows, and
divergence localization, but it is retained as
support/reference tooling rather than part of the
canonical operator surface or active operator
authority. Experimental Rust
replay remains limited to the legacy-frame scope documented in
[docs/replay/tooling.md](replay/tooling.md).
The authoritative operator entrypoint is the `precision` CLI, and the
canonical attached-hardware route is an STM32 target over UART.

Replay-tooling boundaries, including the historically
released Python layer and experimental Rust replay,
are routed through [docs/replay/tooling.md](replay/tooling.md).

## Applications

Possible applications include:

- embedded debugging
- deterministic execution analysis
- reproducible verification
- safety-critical system auditing
- instrumentation of embedded systems

## Implementation Status

The repository provides a reference implementation of the
system architecture. That statement is descriptive only and does
not classify release status.

See:
- [docs/RELEASE_SURFACE.md](RELEASE_SURFACE.md)
- [docs/replay/tooling.md](replay/tooling.md)
- [VERIFICATION_GUIDE.md](./VERIFICATION_GUIDE.md)
