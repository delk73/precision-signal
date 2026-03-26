# Reference Hardware: Analog Reconstruction

**Scope:** This document describes adapter-level analog observation hardware.

**Note:** This hardware is **OPTIONAL** for standard logic verification. It is **REQUIRED** only if the operator wishes to visualize the reconstructed analog waveform (the actual shape of the curve) on an oscilloscope. This is NOT part of the DP32 protocol or reference signal definition; it is a guide for implementation-specific observation.

## 1. Context: Digital vs. Analog Adapters

Observation levels describe measurement layers, not protocol layers. To validate the `precision-signal` pipeline on specific hardware, we distinguish between two observation domains:

* **Observation Level A — Digital Adapter (GPIO):**
* **Interface:** Direct probe on GPIO pin.
* **Observation:** GPIO DAC adapter output (PWM or PDM, implementation-specific). PWM is an adapter-level choice on Raspberry Pi hardware and is not part of the `precision-signal` reference protocol.
* **Verification:** Confirms the `rppal` driver is receiving the correct 32-bit timing intervals from the core (determinism and timing).


* **Observation Level B — Analog Reconstruction (RC Filter):**
* **Interface:** Probe via **Reference RC Filter**.
* **Observation:** Reconstructed Analog Signal (Sawtooth, Sine, Triangle).
* **Verification:** Reveals whether the digitally band-limited curve (processed by DPW4) survives physical reconstruction.


## 2. Reference Circuit Specification

To reconstruct the GPIO DAC adapter output into a continuous analog voltage, a **Reference RC Filter** (1st-Order Low-Pass) is used. This process transforms discrete voltage steps into the smooth curves defined by the 4th-order polynomial kernels.

### Component List

* **R1 (Protection):** 220 Ω (Current limiting / GPIO Protection).
* **R2 (Filter Integration):** 1k Ω (±1% tolerance recommended).
* **C1 (Filter Integration):** 10nF (High-stability Film or C0G/NP0 Ceramic recommended).

### Implementation Diagram

```text
Digital Out (GPIO) ----[ 220 R ]----[ 1k R ]----+---- Analog Out (Scope Probe)
                                                |
                                             [ 10nF ]
                                                |
                                               GND

```

### Transfer Function

This circuit is tuned for the Nominal Verification Rate of 48kHz.

The cutoff frequency ($f_c$) is calculated based on the filter resistor ($R_2$) and the capacitor ($C_1$). It is tuned for the 48kHz verification rate:

Formula: $f_c = 1 / (2 \cdot \pi \cdot R_2 \cdot C_1)$

Calculation: $f_c \approx 15.9$ kHz

### Functional Role

1. **Carrier Suppression:** Effectively removes the PWM/PDM carrier frequency (implementation-dependent; typically well above the audio band) which otherwise causes aliasing on physical oscilloscopes and audio equipment.
2. **Harmonic Preservation:** Passes the core audio band (with modest high-frequency attenuation near 20kHz) while providing the natural integration necessary for the 4th-order DPW logic to achieve its theoretical noise floor.
3. **Validation Logic:** The RC filter is intentionally shallow (first-order, 6dB/octave) and insufficient to hide in-band aliasing. Therefore, a clean analog trace free of low-frequency "ghost ripples" confirms that the *source data* was successfully band-limited by the DPW4 algorithm before transmission.

---
## 3. Verification Commands

### Observation Level A: Digital Adapter (GPIO)
**Goal:** Verify CPU performance and Phase Accumulator gearing.
**Probe:** GPIO 18 (Direct)
```bash
cargo run --example rpi_verify_logic
# Expect: Square Wave (PWM/PDM) | Frequency scales with CPU load

```

### Observation Level B: Analog Reconstruction (RC Filter)

**Goal:** Verify Band-Limiting and Curve Shape.
**Probe:** Analog Out (After RC Filter)

```bash
cargo run --example rpi_verify_analog
# Expect: Smooth Sawtooth Wave | Locked 440Hz

```
