## Axis A — Phase Ordering (`P0..P9`)

Mechanical release gates only.

```
P0 Clean Tree / Provenance
P1 Oracle Integrity (tests + validate + repro)
P2 no_std Oracle
P3 Compile-Surface Quarantine (no-default-features)
P4 Contract Consistency
P5 Artifact Boundary
  P5A Compile-Surface Boundary (authoritative)
  P5B AST Hygiene (WARN; CORE-LEAK=STOP)
P6 Panic / UB / Unsafe / Float Surface
P7 Version / Tag Readiness
```

Letters are subchecks only.

## Axis B — Backlog Items (`Δ`, `DEBT`)

Not phases.

* Δ-### → contract/math/audit issue
* DEBT-### → cleanup/refactor

Must never appear as phase identifiers.

## Axis C — State Tokens

Explicit only:

```
PASS
FAIL
WARN
STOP
```

STOP = do not tag.
