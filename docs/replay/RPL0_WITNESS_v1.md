# RPL0 Witness v1

**Status:** support evidence only. Not release authority unless retained and promoted by a later release process.

The RPL0 witness is an independent support check for retained RPL0 artifacts.

It verifies that a retained RPL0 artifact can be parsed and deterministically folded by a code path separate from the main replay/comparison implementation.

The witness is intended to reduce self-reference in replay review. It does not define the RPL0 format, replace the `precision` CLI, replace the replay/comparison engine, or serve as release authority by itself.

## Claims

The witness may claim:

- the input was parsed as an RPL0 v1 artifact
- the frame region was structurally valid for the accepted input class
- frame indices were monotonic from 0 to `frame_count - 1`
- a deterministic witness digest was computed through an independent implementation path

The witness must not claim:

- firmware correctness
- target correctness
- reset correctness
- flashing correctness
- sensor correctness
- physical timing correctness
- full independent replay equivalence
- general embedded-system correctness
- release authority unless retained and promoted by a later release process

## Accepted Input Class

The initial accepted input class is the current STM32F446 UART RPL0 v1 capture artifact shape:

- `magic = "RPL0"`
- `version = 1`
- `header_len >= 0x98`
- `frame_size = 16`
- frame count within sanity cap
- frame region structurally complete
- frame indices monotonic from 0

The witness is intentionally narrower than future RPL0 evolution. It does not accept v0 artifacts or artifacts with non-standard frame sizes.

## Report Format

The witness emits line-oriented text to stdout. All reports include:

```text
RESULT: PASS | FAIL
WITNESS: rpl0-independent-witness-v1
INPUT: <path>
CLAIM: independent parse and deterministic fold of retained RPL0 artifact
```

Additional fields on PASS:

```text
FORMAT: RPL0/v1
HEADER_LEN: <n>
SCHEMA_LEN: <n>
FRAME_COUNT: <n>
FRAME_SIZE: 16
FIRST_INVALID_FRAME: none
WITNESS_DIGEST: <16 hex digits>
```

Additional fields on FAIL:

```text
ERROR: <specific deterministic error string>
FIRST_INVALID_FRAME: none | <frame index>
```

The `WITNESS_DIGEST` is a deterministic fold over parsed frame fields (`frame_idx`, `irq_id`, `flags`, `rsv`, `timer_delta`, `input_sample`). It is not the canonical RPL identity hash (`SHA-256(header + schema + frames)`). It is an independent witness path over the same frame data.

## Invocation

```bash
python3 scripts/rpl0_witness.py path/to/artifact.rpl0
python3 scripts/rpl0_witness.py path/to/artifact.rpl0 --out witness.txt
```

Exit codes:

- `0`: PASS
- `1`: FAIL (structural or validation error)
- `2`: usage error or unreadable input file

## Make Target

```bash
make rpl0-witness-check
```

Runs `scripts/test_rpl0_witness.py`. Does not require hardware, firmware, or network access.

## Boundary

This witness is support evidence. The `rpl0-witness-check` target is not wired into `make gate`, `make fw-gate`, `make bench-check`, or release-bundle checks.

To promote retained witness output to release authority, a later release-evidence PR must explicitly include witness output in the retained bundle and update the release surface classification accordingly.

## Implementation

Implemented in `scripts/rpl0_witness.py`.

Tests are in `scripts/test_rpl0_witness.py`.

The implementation uses only Python 3 standard library (`struct`, `argparse`, `pathlib`). It does not import `inspect_artifact`, `artifact_tool`, or any other module from the existing replay/comparison path.
