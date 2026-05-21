# Verification Guide

Use this guide to select the correct verification path for your task.

## Choose Your Path

| Intent | Do This | Not This |
| :--- | :--- | :--- |
| **Validate Local Repo** | `make gate` | Running heavy proofs or release bundle assembly unnecessarily. |
| **Bring Up STM32 Board** | `make bench-check`<br>`make fw-gate` | Manual target resets or running Kani proofs during physical bring-up. |
| **Verify Math / Invariants** | Run the formal proof suite via the mathematical verification index. | Mixing symbolic execution with quick hardware checks. |
| **Inspect Past Releases** | Check the retained records at:<br>`docs/verification/releases/` | Attempting a new capture sequence unless actively revalidating. |
| **Prepare a New Release** | Follow mechanics at:<br>`docs/verification/releases/index.md` | Utilizing ad-hoc board or debug workflows. |

---

## Local Validation

```text
make gate

```

Runs the basic software test suite to ensure local changes have not broken core host or parsing logic.

---

## STM32 Board Bring-Up

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

---

## Release Navigation

* **Inspection:** To audit existing release evidence, start with the specific version records located at [docs/verification/releases/](verification/releases/).
* **Preparation:** To construct or evaluate release-bearing evidence packages, follow the mechanics specified in [docs/verification/releases/index.md](verification/releases/index.md).

---

## Authority Links

* **Release Surface Classification:** [docs/RELEASE_SURFACE.md](RELEASE_SURFACE.md)
* **Release Mechanics Index:** [docs/verification/releases/index.md](verification/releases/index.md)
* **STM32 Firmware Contract:** [docs/replay/FW_F446_CAPTURE_v1.md](replay/FW_F446_CAPTURE_v1.md)
* **RPL0 Format Contract:** [docs/spec/rpl0_format_contract.md](spec/rpl0_format_contract.md)
