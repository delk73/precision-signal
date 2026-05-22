# Verification Guide

This guide defines the active verification flow for repository changes, STM32
bench validation, formal proof evidence, and retained release records.

## Primary Path

For ordinary local changes, run the release-facing software gate:

```text
make gate
```

For hardware-backed validation, run the bench preflight before the firmware
gate:

```text
make bench-check
make fw-gate
```

Release-surface classification remains in [RELEASE_SURFACE.md](RELEASE_SURFACE.md).
Retained release records and release-package mechanics remain under
[verification/releases/](verification/releases/).

---

## Local Validation

```text
make gate

```

Runs the basic software test suite to ensure local changes have not broken core host or parsing logic.

---

## STM32 Bench Validation

```text
make bench-check
make fw-gate

```

* **Automation Boundary:** The active board-bringup path uses ST-LINK reset logic by default. It must not require the operator to press a physical reset button manually.
* **Isolation:** Kani symbolic proofs are not executed during physical target validation.

---

## Formal Proof Boundary (Kani)

Mathematical proofs provide compile-time verification for state engine semantics and cross-platform fixed-point math. These assets define your proof-boundary evidence and are decoupled from quick local gates:

* For math kernel specs, review the verification paths indexed in [docs/MATH_CONTRACT.md](MATH_CONTRACT.md).
* Do not mix formal proof execution into quick local or physical bench gates
  unless a release procedure explicitly asks for retained proof evidence.

---

## Release Navigation

Existing release evidence lives in versioned records under
[docs/verification/releases/](verification/releases/).

To construct or evaluate release-bearing evidence packages, follow the
mechanics specified in
[docs/verification/releases/index.md](verification/releases/index.md).

---

## Authority Links

* **Release Surface Classification:** [docs/RELEASE_SURFACE.md](RELEASE_SURFACE.md)
* **Release Mechanics Index:** [docs/verification/releases/index.md](verification/releases/index.md)
* **STM32 Firmware Contract:** [docs/replay/FW_F446_CAPTURE_v1.md](replay/FW_F446_CAPTURE_v1.md)
* **RPL0 Format Contract:** [docs/spec/rpl0_format_contract.md](spec/rpl0_format_contract.md)
