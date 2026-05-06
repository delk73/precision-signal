Title: Power Curve → Execution Mapping (Design Note)

Context:
- replay system validates execution equivalence
- voltage sweep shows deterministic behavior until collapse
- hysteresis observed between run-sustain and recovery

Concept:
- map input power profiles (curves) to execution outcomes
- outcomes: PASS / EDGE / FAIL / DIVERGENCE

Non-goal:
- do not expand CLI contract yet

Implication:
- future characterization layer above record/replay/diff
- requires:
  - programmable power control
  - synchronized capture
  - repeatable profiles

Status:
- exploratory
- grounded in STM32 replay experiments
