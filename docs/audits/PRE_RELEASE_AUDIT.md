## AUDIT — Canonical P5

Purpose: define authoritative pre-release audit policy for Artifact Boundary enforcement.

Scope statement: this document is canonical for **Phase 5** only. Other phases are defined in their respective audit/spec documents and may be consolidated here later.

## P5A — Artifact Boundary (Authoritative STOP gate)

PASS requires both:

```bash
cargo check -p dpw4 --no-default-features --target thumbv7em-none-eabihf --locked
cargo check --workspace --no-default-features --locked
```

If either fails → STOP.

**Rule:** compile-surface is authoritative; token scans are not.

## P5B — Artifact Boundary Hygiene (AST scan; WARN-only unless core-leak)

Run:

```bash
cargo run --locked -p audit-float-boundary -- --mode phase5b
```

Interpretation:

* `examples/**` excluded (non-normative).
* Detection scope is AST-based float surface:
  * type usage: `f32`, `f64`
  * path usage: `core::f64::...`, `std::f64::...`, `f64::...`
  * literal suffix usage: `...f32`, `...f64`
* Remaining hits are **ALLOWED** only in:

  * `#[cfg(test)]`, `tests/`, `benches/`
  * `src/bin/**`
  * `#[cfg(feature="float-ingest")]` gated code
  * file-level gates: `#![cfg(test)]` or `#![cfg(feature = "float-ingest")]`

Policy A (structural): parent-module gating is not sufficient. Hits must be in an allowlisted path, a file-level gate, or an item/block directly gated by `#[cfg(test)]` or `#[cfg(feature = "float-ingest")]`.

Classification:

* Hits outside allowlist → **WARN** and open tracking issue.
* OPTIONAL escalation rule (enable now): if hit is in `crates/dpw4/src/**` excluding `src/bin/**` and not `cfg(test)` or `float-ingest` → classify as **CORE-LEAK** (treat as STOP for release).

Record output as evidence.
