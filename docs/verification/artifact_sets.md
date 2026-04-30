# Verification Artifact Sets

This document describes the major verification evidence sets used by the
repository. Normative status and governance remain in [VERIFICATION_GUIDE.md](../VERIFICATION_GUIDE.md).

## Determinism Validation Artifacts

The pinned quick validation set is:

- `saw_20_headroom.det.csv`
- `pulse_relational_8k.det.csv`
- `triangle_linearity_1k.det.csv`
- `sine_linearity_1k.det.csv`
- `long_run_0_1hz.det.csv`
- `master_sweep_20_20k.det.csv`

Canonical generation and checking are performed by `sig-util validate` and
`precision artifacts`.

## Forensic Hashing

Normative identity is defined over raw `S32LE` sample bytes serialized with
`to_le_bytes()`.

- raw little-endian sample bytes are the canonical hashing surface
- derived containers such as CSV and WAV are advisory transport formats
- hash-regeneration policy is governed by [VERIFICATION_GUIDE.md](../VERIFICATION_GUIDE.md)

## Formal Verification

Kani proofs cover selected fixed-point kernel boundaries in `dpw4` and
`geom-signal`, including polynomial safety, saturation, `sqrt`, `sin`, `cos`,
and tiered `atan2` shards.

Reference commands:

```bash
bash verify_kani.sh
RUN_HEAVY=1 bash verify_kani.sh
```

Per-harness logs are retained under `kani_logs/<package>__<harness>.log`.

## Physical Verification

Physical-time observation is documented separately:

- [docs/hardware/REFERENCE_HARDWARE.md](../hardware/REFERENCE_HARDWARE.md)
- [docs/debug/reset_run_characterization.md](../debug/reset_run_characterization.md)

These procedures are evidence and operational guidance. They do not redefine the
core arithmetic contract.

## Historical Verification References

Historical verification material remains preserved behind
[docs/verification/releases/index.md](releases/index.md).

- [docs/verification/CLI_SURFACE_EVIDENCE.md](CLI_SURFACE_EVIDENCE.md): retained historical CLI promotion evidence
- [docs/verification/CI_EVIDENCE.md](CI_EVIDENCE.md): retained historical CI evidence
- [docs/verification/FIRMWARE_CAPTURE_EVIDENCE.md](FIRMWARE_CAPTURE_EVIDENCE.md): retained historical firmware capture evidence
- [docs/verification/D-03_TriangleDPW4_Audit.md](D-03_TriangleDPW4_Audit.md): retained TriangleDPW4 audit note
