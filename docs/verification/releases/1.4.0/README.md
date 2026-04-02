# Release Evidence Bundle (1.4.0)

This directory is the retained release record for verification-depth release `1.4.0`.

Scope of this cut:

- one explicit bounded correctness claim for the released sine path over a stated finite domain
- one composition-level invariant statement for the active triangle signal path
- one explicit proof-coverage and limits statement for the release-facing verification claim
- retained canonical command outputs for the `1.4.0` release decision

Retained checklist outputs in this bundle:

- `cargo_check_dpw4_thumb_locked.txt`
- `make_gate.txt`
- `make_replay_tests.txt`
- `make_demo_evidence_package.txt`
- `make_doc_link_check.txt`
- `make_check_workspace.txt`
- `verify_release_repro.txt`
- `release_reproducibility.txt`
- `kani_evidence.txt`
- `cargo_test_sine_bounded_correctness.txt`
- `cargo_test_triangle_control_surface.txt`
- `VERIFICATION_SCOPE.md`

Historical retained release records remain immutable:

- `docs/verification/releases/1.3.1/` remains the prior patch-release retained bundle
- `docs/verification/releases/1.2.2/` remains the older hardware-backed retained bundle
- no files under older retained release directories were modified for `1.4.0`
