# Precision CLI Reference (Normative)
**Document revision:** 1.2.2  
**Applies to:** release 1.2.2 (content unchanged)  
**Surface status:** Normative (release 1.2.2)  
**Status:** Normative (Tooling Surface Lock)

## Versioning Terminology

- Document revision labels editorial history for this CLI reference.
- Release versions identify the shipped software release.
- Artifact/header versions referenced by this CLI are part of the artifact contracts and are not CLI document revisions.

## Overview
The `precision` binary is the reference Rust CLI for generation, inspection,
verification, artifact production, and validation of DP32 Reference Standard
signals. It is gated behind the `cli` feature in the `dpw4` crate.

For release-surface classification, use [docs/RELEASE_SURFACE.md](../RELEASE_SURFACE.md).
For verification governance, use [VERIFICATION_GUIDE.md](../../VERIFICATION_GUIDE.md).

## Command Surface

### `generate` (Default)
Generates a DP32 or WAV reference signal.

| Argument | Description | Default |
| :--- | :--- | :--- |
| `--shape` | Waveform shape (`saw`, `square`, `triangle`, `triangle-dpw1`, `pulse`) | `saw` |
| `--freq` | Frequency in Hz | `440.0` |
| `--rate` | Sample rate in Hz | `48000` |
| `--seconds` | Duration in seconds (Indefinite if omitted) | N/A |
| `--gain` | Output gain in dBFS | `-3.0` |
| `--container-wav` | Wrap output in RIFF WAV container | `false` |

*   **Inputs**: CLI arguments.
*   **Outputs**: Binary signal stream to `stdout`.
*   **Side Effects**: Advisory warnings (e.g., Triangle aliasing) to `stderr`.
*   **WAV Requirement**: `--container-wav` requires `--seconds`.

### `inspect`
Reads and validates a DP32 file header.

| Argument | Description |
| :--- | :--- |
| `--file` | Path to DP32 file (Defaults to `stdin` if omitted) |

*   **Inputs**: File path or `stdin`.
*   **Outputs**: Formatted header metadata to `stderr`.
*   **Exit Codes**: `0` on success, `1` if magic is invalid or header is malformed.

### `verify`
Performs structural integrity and alignment checks on a DP32 file. (Note: This does not compute a payload hash).

| Argument | Description |
| :--- | :--- |
| `--file` | Path to DP32 file (Required) |

*   **Inputs**: File path.
*   **Outputs**: Verification status and calculated duration to `stdout`.
*   **Checks**:
    *   File size $\ge 64$ bytes.
    *   Magic must be `b"DP32"`.
    *   Bit depth must be `32`.
    *   Payload size must be a multiple of 4 (alignment check).
*   **Exit Codes**: `0` on success, `1` on parse or structural failure, `2` on integrity failure.

### `artifacts`
Generates forensic artifact CSV/bin pairs through a supported CLI surface.

| Argument | Description | Default |
| :--- | :--- | :--- |
| `--out` | Output directory for artifacts | `docs/verification` |

*   **Inputs**: CLI arguments.
*   **Outputs**:
    * rich advisory traces: `*.csv`
    * determinism-gate traces: `*.det.csv` (integer/hex only)
    * header-only streams: `*_headers.bin`
*   **Exit Codes**: `0` on success, `1` on failure.

### `validate`
Runs the canonical deterministic release gate.

| Argument | Description | Default |
| :--- | :--- | :--- |
| `--out` | Validation workspace | `target/precision_validate` |
| `--mode` | `quick` or `full` | `quick` |
| `--json` | Emit single JSON report to `stdout` | `false` |
| `--keep` | Keep `run1`/`run2` on PASS | `false` |

*   **Operator Routing**: `make gate` is the canonical operator-facing entrypoint. `precision validate --mode quick` is the normative underlying command.
*   **Checks**:
    *   `version_consistency`
    *   `toolchain_pin`
    *   `header_stream_integrity` using `long_run_0_1hz_headers.bin`
    *   `determinism_bit_exact` over the normative `.det.csv` set:
        *   `saw_20_headroom.det.csv`
        *   `pulse_relational_8k.det.csv`
        *   `triangle_linearity_1k.det.csv`
        *   `sine_linearity_1k.det.csv`
        *   `master_sweep_20_20k.det.csv`
        *   `long_run_0_1hz.det.csv`
*   **Non-Normative Canary**: `phase_wrap_440` is generated and checked for consistency across repeated runs, but it is not part of the normative hash-locked release baseline. Its WARN status is informational only.
*   **Outputs (human)**: Stepwise PASS/FAIL/SKIP plus final `VERIFICATION PASSED` or `VERIFICATION FAILED`.
*   **Outputs (`--json`)**: `{ "status", "out_dir", "checks", "dpw4_version", "features" }`.
*   **Exit Codes**: `0` on pass, `1` on any failure.

## File Formats

### DP32 (Canonical)
*   **Header**: Exactly 64 bytes (aligned to 64).
    *   `0-3`: `b"DP32"`
    *   `4-7`: `u32` Version (1)
    *   `8-15`: `u64` Sequence
    *   `16-19`: `u32` Sample Rate
    *   `20-23`: `u32` Bit Depth (32)
    *   `24-63`: Reserved Padding
*   **Payload**: Contiguous stream of Little-Endian 32-bit signed integers (`S32LE`).

### WAV (Inspection/Listening)
When `--container-wav` is used, the tool emits a standard RIFF WAVE file:
*   **Container**: 32-bit PCM mono.
*   **Clamping**: Internal signals are saturated to `i32::MIN..=i32::MAX` before emission.

## Verification Semantics

### SHA-256 Hashing
The "Forensic Audit" is performed by hashing the **raw S32LE samples only**, excluding the 64-byte DP32 header.

*   **What is hashed**: The payload section of a DP32 file or the raw generated sample stream.
*   **Byte Order**: Little-endian (bytes as they appear in the file payload).
*   **Authoritative Results**: SHA-256 hashes produced by or verified against the `forensic_audit` test suite are the absolute definition of correctness.

### Pass/Fail Criteria
*   **Pass (Structural)**: The `verify` command confirms all header fields are valid and the payload size is aligned to 4 bytes.
*   **Pass (Forensic)**: The payload SHA-256 hash matches the Golden Reference (as authoritatively defined in `forensic_audit.rs`).
*   **Fail (Structural)**: Any invalid header field, incorrect magic, or misaligned payload size (detected by `verify`).
*   **Fail (Forensic)**: Any bit-level deviation in the payload relative to the Golden Reference.
*   **Not Checked**: The value of the `pad` bytes in the header is reserved and not currently part of the normative hash.
