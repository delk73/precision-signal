# CLI Surface Evidence

This document records the minimal operator-path evidence used to promote safe
CLI surfaces from Reference to Release.

Evidence requirements per command:

- runnable command
- observable deterministic output or invariant
- success exit code

All commands below are repo-local, require no hardware, and use only local
filesystem state under `target/cli_surface_evidence/`.

## Workspace Setup

```bash
mkdir -p target/cli_surface_evidence
```

## `precision generate`

Command:

```bash
cargo run -p dpw4 --features cli --bin precision -- generate --shape saw --freq 440 --rate 48000 --seconds 1 > target/cli_surface_evidence/generate_a.dp32
cargo run -p dpw4 --features cli --bin precision -- generate --shape saw --freq 440 --rate 48000 --seconds 1 > target/cli_surface_evidence/generate_b.dp32
cmp -s target/cli_surface_evidence/generate_a.dp32 target/cli_surface_evidence/generate_b.dp32
sha256sum target/cli_surface_evidence/generate_a.dp32
```

Expected:

- both commands exit `0`
- `target/cli_surface_evidence/generate_a.dp32` exists
- `cmp -s` exits `0`
- repeated runs are byte-identical

Observed:

- `sha256(generate_a.dp32) = 58def0ebdc6c29d22726df38a93d5f07022d2fc72f6dad89a9615d11fc25a1bd`

## `precision artifacts`

Command:

```bash
cargo run -p dpw4 --features cli --bin precision -- artifacts --out target/cli_surface_evidence/artifacts_a
cargo run -p dpw4 --features cli --bin precision -- artifacts --out target/cli_surface_evidence/artifacts_b
diff -rq target/cli_surface_evidence/artifacts_a target/cli_surface_evidence/artifacts_b
sha256sum target/cli_surface_evidence/artifacts_a/saw_20_headroom.det.csv
sha256sum target/cli_surface_evidence/artifacts_a/long_run_0_1hz_headers.bin
```

Expected:

- both commands exit `0`
- both output directories exist
- `diff -rq` exits `0`
- repeated runs produce identical artifact sets
- `saw_20_headroom.det.csv` hash matches the pinned value in [VERIFICATION_GUIDE.md](../../VERIFICATION_GUIDE.md)
- `long_run_0_1hz_headers.bin` exists for downstream header audit

Observed:

- `sha256(saw_20_headroom.det.csv) = ec99d4d0407bb48ec442e629e66f08f13547913c0583b31fe1c0e48df067edc1`
- `sha256(long_run_0_1hz_headers.bin) = 647e9639b2188c00f763df70d4150236bb6f67a30e23c1465e5ce64fa8c770c4`

## `precision inspect`

Command:

```bash
cargo run -p dpw4 --features cli --bin precision -- inspect --file target/cli_surface_evidence/generate_a.dp32
```

Expected:

- command exits `0`
- stderr prints:
  - `Status:      VALID HEADER`
  - `Protocol v:  1`
  - `Sample Rate: 48000 Hz`
  - `Bit Depth:   32-bit (S32LE)`
- no panic

## `precision verify`

Command:

```bash
cargo run -p dpw4 --features cli --bin precision -- verify --file target/cli_surface_evidence/generate_a.dp32
```

Expected:

- command exits `0`
- stdout prints `✅ VERIFIED: DP32 Reference File`
- stdout prints `Duration: 1.0000 sec`
- no panic

## `header_audit`

Command:

```bash
cargo run -p dpw4 --features cli --bin header_audit -- target/cli_surface_evidence/artifacts_a/long_run_0_1hz_headers.bin
```

Expected:

- command exits `0`
- stdout prints `✓ Audit PASSED. Checked 1000000 headers.`
- no panic

## Promotion Result

The following CLI surfaces satisfy the minimal operator-path evidence bar in
this audit:

- `precision generate`
- `precision artifacts`
- `precision inspect`
- `precision verify`
- `header_audit`
