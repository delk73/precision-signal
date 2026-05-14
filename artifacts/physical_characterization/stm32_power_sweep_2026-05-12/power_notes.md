# Power Evidence Notes

Power evidence is bench provenance only.

External power is dominant.

ST-LINK/USB connected for reset/debug continuity.

Residual interface-side contribution not independently isolated.

External rail observation:
- flat
- no visible sag
- no visible oscillation

3V3 rail observation:
- flat
- tracked downward
- reached ~1.x range
- no visible oscillation

Exact dropout threshold not precisely measured.

Run 008 current observation:
- DM110 current readings were not stable enough for precise per-run current measurement.
- Observed current behavior appeared repeatable: lower/quiet draw after reset or power adjustment, followed by delayed and more burst-like draw during apparent execution/capture activity.
- This was treated as qualitative bench provenance only and was not used for pass/fail classification.
