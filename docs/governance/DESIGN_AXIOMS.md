## Axiom: Discretization Follows Finite Observation

**Status:** Normative

### Statement
Continuous dynamics are primary. Discrete states (modes, events, classes) are
secondary artifacts imposed by finite observation, finite representation, and
finite actuation.

Discretization MUST NOT be introduced as a modeling convenience. It MUST only
appear at interfaces where resolution collapses and indistinguishability is
structural.

### Rationale (Informative)
Any sensor, encoder, and actuator has bounded precision and bandwidth. When
differences in a continuous quantity are below the observer’s resolution, they
are indistinguishable and can only be represented as equivalence classes. Those
equivalence classes are what we call “discrete states.”

### Implications (Normative)
1. **Hot paths remain curve-native.** Inner loops MUST operate on continuous
   (or fixed-point continuous) trajectories and preserve invariants over time.
2. **Discreteness is an interface property.** Mode switches, thresholds, and
   scheduling decisions MUST be implemented at boundaries where:
   - measurement precision is bounded,
   - actuation requires commitment, or
   - sampling imposes discrete time.
3. **No premature mode logic.** Discrete policy MUST NOT leak into hot loops
   beyond reading latched scalars/enums.
4. **Quantization is explicit.** All quantization (rounding, truncation,
   saturation, binning) MUST be specified and testable, with stated resolution.
5. **Budgets justify partitions.** Any discrete partitioning (e.g. mode bands,
   threshold windows) MUST be defensible via:
   - measurement noise floor, or
   - timing/compute budgets, or
   - stability/hysteresis constraints.

### Acceptance Criteria (Normative)
- If a discrete state exists, the repository MUST identify the forcing boundary
  (sensor resolution, encoding width, sampling rate, or actuator commitment).
- If no forcing boundary can be identified, the discretization is invalid and
  MUST be removed or reclassified as advisory experimentation.

### Examples (Informative)
- Valid: quantizing a control decision because telemetry is measured at 250 Hz
  with known ADC resolution and noise.
- Valid: saturating magnitude to prevent overflow with explicitly defined
  thresholds.
- Invalid: introducing “modes” in the hot loop solely to simplify reasoning
  when the underlying quantity is continuous and already bandlimited.
