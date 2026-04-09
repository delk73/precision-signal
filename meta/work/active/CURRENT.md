# Packet
- 1.6.0 retained release-bundle preparation

## Objective
- create/populate docs/verification/releases/1.6.0 so release-bundle-check passes for the narrowed 1.6.0 claim

## Entry criteria
- release-readiness packet completed
- narrowed primary precision claim recorded
- formal blocker identified

## Exit criteria
- docs/verification/releases/1.6.0 exists
- make release-bundle-check VERSION=1.6.0 passes
- formal release decision can be reevaluated

## In scope
- inspect prior retained release bundle pattern
- determine minimum required 1.6.0 retained artifacts
- create/populate docs/verification/releases/1.6.0
- rerun release-bundle-check VERSION=1.6.0

## Out of scope
- replay-host scope expansion
- new primary-surface review
- broad architecture changes
- widening the 1.6.0 claim beyond today’s narrowed boundary

## Current blocker
- docs/verification/releases/1.6.0 does not exist; make release-bundle-check VERSION=1.6.0 fails
