# Build Reproducibility

This document is descriptive. The normative release and conformance governance is
defined in [VERIFICATION_GUIDE.md](../VERIFICATION_GUIDE.md).

Use this file for supporting build-identity detail only. It does not define
release admissibility and it does not replace the canonical release path
`make gate`.

## Pinned Environment

- compiler: `rustc 1.91.1`
- toolchain source: `rust-toolchain.toml`
- reference arch class: 64-bit little-endian (`x86_64`, `aarch64`)
- core precision bedrock: `Scalar = I64F64`
- egress width: `S32LE`

## Reference Build Commands

```bash
cargo build --release -p dpw4 --features cli
```

The command above is the canonical reference build for the released replay
surface.

## Determinism And Reproducibility Checks

Canonical operator-facing determinism gate:

```bash
make gate
```

Normative underlying command:

```bash
cargo run --release -p dpw4 --features cli --bin sig-util -- validate --mode quick
```

Supporting release-binary identity check:

```bash
bash scripts/verify_release_repro.sh
```

`make gate` is the canonical release gate.
The cargo invocation above is the underlying implementation.
`bash scripts/verify_release_repro.sh` is a supporting same-machine dual-build identity
check for the `sig-util` release binary.
It freezes `SOURCE_DATE_EPOCH` to the current `HEAD` commit timestamp unless
the caller explicitly provides the variable.
For `1.2.0`, retained build reproducibility evidence is supporting-only and is
not required for release admissibility.

If its result is retained as part of a release record, archive it in the
canonical retained release-evidence location:

```bash
RELEASE_EVIDENCE_DIR=docs/verification/releases/<version>/ bash scripts/verify_release_repro.sh
```

## Related Evidence

- [VERIFICATION_GUIDE.md](../VERIFICATION_GUIDE.md)
- `docs/verification/releases/<version>/`
- [docs/audits/delta-bd_build-determinism.md](../audits/delta-bd_build-determinism.md)
- [docs/verification/CI_EVIDENCE.md](CI_EVIDENCE.md)
