# Witness Model Direction (Post v1.2.2)

Status: Research / Directional
Scope: Non-normative
Affects: Future architecture only

## Problem

The current system is described primarily in terms of artifact capture, replay,
and divergence analysis. That framing is valid for the present release surface
and implementation.

This document records an alternative first-principles framing for future
architecture discussion. It does not redefine the current system, current
specifications, or current release claims.

## Core Idea

A future architecture direction may treat the system primarily as an execution
witness model rather than as artifact plus replay plus analysis.

Under that framing:

- capture produces a proof-carrying record of execution
- comparison establishes sameness or divergence on a defined surface
- explanation is optional and bounded by explicit rules
- replay is an interpreter over the record, not the foundation of the system

This direction is intended only as a possible architectural simplification for
future work. It is not a statement about current scope.

## Directional Axes

### Terminology shift (artifact -> witness)

A future architecture exploration may use `witness` as an internal design term
for a deterministic execution record whose primary purpose is to support
verification and comparison. This would be a framing change for future design
work, not a required rename of the current release surface.

### Hard layer separation

A future design could separate record format, record comparison, explanatory
classification, and replay interpretation more explicitly. The intent would be
to keep capture truth, comparison truth, and explanation truth distinct. This
is a possible architectural refinement, not a present implementation claim.

### Explicit divergence states

A future model may represent divergence with a small number of explicit states,
such as identical, classified divergence, and unclassified divergence. The goal
would be to make unsupported differences first-class outcomes rather than rely
only on tool-level failure signaling. This is directional only and does not
change current behavior.

### State commitments

A future design may evaluate compact state commitments as part of the execution
record so that broader execution sameness can be checked without expanding the
semantic interpretation surface. This is a research direction only and does not
imply any current format or implementation change.

### Perturbation as first-class

A future architecture may treat controlled perturbation workflows as a primary
tool for understanding divergence propagation rather than only as supporting
demonstration material. This would be an optional future extension to the
architectural model and not a statement about current scope.

### Reduced semantic coupling

A future design may reduce direct coupling between field names and explanatory
meaning so that the minimal comparison surface remains stable even as
higher-level interpretation evolves. This is a possible future simplification,
not a proposed change to current released terminology or tooling.

## Relationship to Current System

The current system already satisfies many properties that align with this
direction:

- the deterministic artifact contract defines a strict recorded surface
- `artifact_diff` already acts as a comparator over that surface
- Demo V3 through Demo V5 already provide bounded explanation layers
- unsupported differences are rejected rather than explained speculatively

This document therefore describes a refinement in framing for future
architecture discussion. It is not a claim that the current system is
incomplete, incorrect, or misdesigned.

## Non-goals

- no immediate rename of the current release surface
- no mandatory refactor
- no current artifact-format change
- no current validation-semantic change

## Adoption Criteria

Any future adoption of this direction should:

- preserve the deterministic artifact contract
- preserve bounded explanation behavior
- preserve reproducible validation gates
- proceed incrementally rather than through rewrite-driven replacement

## Next Steps (Post-Release)

Evaluate this direction only within a dedicated future architecture sprint.
