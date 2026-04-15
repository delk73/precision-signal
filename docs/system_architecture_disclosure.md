# System Architecture Disclosure

## Overview

This document records the architecture implemented in the
`precision-signal` repository.

Use [docs/RELEASE_SURFACE.md](RELEASE_SURFACE.md) for release classification,
[VERIFICATION_GUIDE.md](../VERIFICATION_GUIDE.md) for verification authority, and
[docs/replay/tooling.md](replay/tooling.md) for replay-tooling boundaries.

The repository implements deterministic execution analysis
infrastructure designed to capture execution artifacts,
analyze divergence between independent executions, and
provide a replay-tooling layer that includes historically
released Python operator tooling retained as
support/reference material for `1.6.0`.

## Core Concepts

Artifact Capture

Execution artifacts are recorded from running systems
using a canonical binary format.

Deterministic Replay

The Python tooling layer (`scripts/artifact_tool.py`
and `scripts/artifact_diff.py`) remains historically
released replay-facing operator tooling, but for the
retained `1.6.0` release it is support/reference tooling
and is not part of the canonical `1.6.0` operator
surface. Rust replay
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
divergence localization, but for `1.6.0` it is retained as
support/reference tooling rather than part of the
canonical `1.6.0` operator surface. Experimental Rust
replay remains limited to the legacy-frame scope documented in
[docs/replay/tooling.md](replay/tooling.md).

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
- [VERIFICATION_GUIDE.md](../VERIFICATION_GUIDE.md)
