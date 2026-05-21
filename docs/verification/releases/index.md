# Retained Release Evidence

This directory is the GitHub Pages route for retained release-evidence bundles
and historical verification references.

Use the active authority path first:

- [VERIFICATION_GUIDE.md](../../VERIFICATION_GUIDE.md): verification router and
  compact core authority
- [docs/RELEASE_SURFACE.md](../../RELEASE_SURFACE.md): release-surface
  classification and routing
- [docs/authority/cli_contract.md](../../authority/cli_contract.md): sole
  active CLI contract authority
- [docs/replay/FW_F446_CAPTURE_v1.md](../../replay/FW_F446_CAPTURE_v1.md):
  active STM32 F446 RPL0 execution capture authority
- [docs/replay/INTERVAL_CAPTURE_CONTRACT_v1.md](../../replay/INTERVAL_CAPTURE_CONTRACT_v1.md):
  STM32 timing characterization/support capture contract

## Active Retained Release Record

Active retained release record: `1.8.0`

- [Release 1.8.0](1.8.0/): firmware-including release — RPL0 restore, precision.meta.v2, dpw4 streaming hash
- [Release 1.8.0 index.md](1.8.0/index.md): primary human-readable release
  summary and retained file inventory

## Retained-Release Mechanics

This page owns retained-release mechanics. `VERIFICATION_GUIDE.md` routes
release-focused readers here and does not duplicate the detailed release
procedure.

Retained release evidence lives under:

```text
docs/verification/releases/<version>/
```

For the active retained release record, inspect:

```text
docs/verification/releases/1.8.0/
docs/verification/releases/1.8.0/index.md
```

Use the retained bundle, not transient workspace artifacts, when evaluating a
release claim.

## Release Preparation Routes

Current release tooling exposes two retained-release preparation routes:

- `make release-1.8.0`: compatibility retained-record orchestration for the
  active `1.8.0` firmware-including record.
- `make release-proof VERSION=<ver>`: generic release-proof orchestration for
  new release execution.

After a retained bundle exists, check it with:

```text
make release-bundle-check VERSION=<version>
```

The bundle check validates retained-bundle coherence. It does not replace the
evidence-producing release route.

## Retained Kani Evidence

Kani evidence is retained release/proof-boundary evidence. It is not a board
bring-up step.

The active `1.8.0` retained-record route requires
`docs/verification/releases/1.8.0/kani_evidence.txt` to exist before
`make release-1.8.0` runs. Other release-bundle tooling may produce
`kani_evidence.txt` directly as part of bundle generation. Follow the current
tooling behavior for the route being used; do not assume every downstream
release command reruns Kani.

The canonical Tier-1 runner is:

```text
bash scripts/verify_kani.sh
```

Optional proof tiers are retained only when a release record explicitly includes
them.

## Firmware-Including Release Evidence

Firmware-including release claims require retained firmware evidence for the
active STM32 RPL0 capture path. For `1.8.0`, that evidence is under:

```text
docs/verification/releases/1.8.0/
```

The firmware evidence set includes the archived capture artifact, capture hash
check, repeat-capture manifest, repeat hash check, and firmware release evidence
summary when present in that per-version directory.

Firmware release flows use the active `make fw-gate` path. Its default reset
mode is ST-LINK reset through `FW_GATE_RESET_MODE ?= stlink`; manual reset is
legacy/debug support only, not the normal release path.

`make release-proof VERSION=<ver> RELEASE_PROOF_FIRMWARE=0` is a non-firmware
proof route. It must not be used as evidence for a firmware-including release
claim.

## Historical Retained Record Policy

Historical retained records are preserved as release-scoped evidence for the
version that produced them. Do not copy evidence forward from older releases
into a new retained bundle. If a historical record lacks newer generated summary
files, the record remains valid as historical evidence when its per-version
index explains the retained contents.

## Historical Verification References

- [docs/verification/CI_EVIDENCE.md](../CI_EVIDENCE.md): retained historical CI
  evidence
- [docs/verification/FIRMWARE_CAPTURE_EVIDENCE.md](../FIRMWARE_CAPTURE_EVIDENCE.md):
  retained historical firmware capture evidence
- [docs/verification/D-03_TriangleDPW4_Audit.md](../D-03_TriangleDPW4_Audit.md):
  retained TriangleDPW4 audit note

## Historical Retained Release Evidence

- [Release 1.7.0](1.7.0/): narrowed primary precision non-firmware release bundle
- [Release 1.6.0](1.6.0/): narrowed retained release bundle with bounded firmware capture evidence
- [Release 1.5.0](1.5.0/): bounded Rust replay release bundle
- [Release 1.4.0](1.4.0/): prior verification-depth release bundle
- [Release 1.3.1](1.3.1/): prior patch-release retained bundle
- [Release 1.2.2](1.2.2/): hardware-backed retained bundle with v1 replay manifest
- [Release 1.2.1](1.2.1/): earlier hardware-backed retained bundle with v1 replay manifest
- [Release 1.2.0](1.2.0/): earliest retained hardware-backed bundle with historical v0 replay manifest

## Documented Retained-Evidence Gap

- [Release 1.3.0](1.3.0/): tag/changelog entry exists, but no retained release bundle was committed; explanatory note only, with no reconstructed historical evidence

Release-scoped classification remains in
[`docs/RELEASE_SURFACE.md`](../../RELEASE_SURFACE.md). Core verification routing
remains in [`VERIFICATION_GUIDE.md`](../../VERIFICATION_GUIDE.md).
