# RPL0 Witness v1

**Status:** support evidence by default. It becomes release authority only when
retained in a per-version release bundle and explicitly promoted by that
release's authority chain.

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
- `flags = 0`
- v1 reserved field at `0x96` equals `0`
- `schema_hash = SHA256(schema_block)`
- frame count within sanity cap
- frame region structurally complete
- no trailing bytes after the declared frame region
- frame indices monotonic from 0

The witness is intentionally narrower than future RPL0 evolution. It does not accept v0 artifacts or artifacts with non-standard frame sizes.

## Report Format

The witness emits line-oriented text to stdout. Reports use stable field names and deterministic field order. Core fields appear first and remain parseable even when a value is unavailable.

PASS reports begin with:

```text
RESULT: PASS
ARTIFACT_COUNT: 1
FRAME_SIZE: 16
FRAME_COUNT: <n>
WITNESS_DIGEST: <16 hex digits>
FIRST_INVALID_FRAME: none
```

FAIL reports begin with:

```text
RESULT: FAIL
ARTIFACT_COUNT: 1
FRAME_SIZE: none | <declared frame_size>
FRAME_COUNT: none | <declared frame_count>
WITNESS_DIGEST: none
FIRST_INVALID_FRAME: none | <frame index>
ERROR: <specific deterministic error string>
```

All reports also include:

```text
WITNESS: rpl0-independent-witness-v1
INPUT: <path>
CLAIM: independent parse and deterministic fold of retained RPL0 artifact
```

PASS reports additionally include:

```text
FORMAT: RPL0/v1
HEADER_LEN: <n>
SCHEMA_LEN: <n>
```

`FIRST_INVALID_FRAME` is a frame index only for frame-order or frame-parse failures. It is `none` for header, schema, size, hash, trailing-byte, and other pre-frame failures.

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
make replay-witness-check VERSION=<version>
```

`make rpl0-witness-check` runs the local witness regression tests. It does not
require hardware, firmware, or network access.

`make replay-witness-check VERSION=<version>` validates retained release
witness evidence under `docs/verification/releases/<version>/` by recomputing
the witness from `fw_capture.bin` and comparing it with the retained
`rpl0_witness_fw_capture.txt` digest.

## Boundary

This witness is support evidence unless a release authority chain names the
retained witness check as required. The `rpl0-witness-check` target is not wired
into `make gate`, `make fw-gate`, or `make bench-check`.

For 2.0-style authority bundles, `make release-bundle-check VERSION=<version>`
checks that the retained witness report exists, is indexed, and is included in
the generated summary hashes. It does not recompute witness semantics; that is
the job of `make replay-witness-check VERSION=<version>`.

To promote retained witness output to release authority, a release-evidence PR
must explicitly include witness output in the retained bundle and name
`make replay-witness-check VERSION=<version>` in the authority chain.

## Implementation

Implemented in `scripts/rpl0_witness.py`.

Tests are in `scripts/test_rpl0_witness.py` and
`scripts/test_check_replay_witness.py`.

The implementation uses only Python 3 standard library (`struct`, `argparse`, `hashlib`, `pathlib`). It does not import `inspect_artifact`, `artifact_tool`, or any other module from the existing replay/comparison path.
