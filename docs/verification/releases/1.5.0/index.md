# Release Evidence Bundle (1.5.0)

This directory is the retained release record for minor release `1.5.0`.

Scope of this cut:

- one carry-forward verification-scope note for the inherited bounded sine correctness claim
- one bounded Rust replay command promotion: `replay-host diff`
- release scope anchored to the retained `artifacts/rpl0/` proof corpus only
- retained command transcripts and release-check outputs supporting that narrow claim

Retained release-scope notes:

- [VERIFICATION_SCOPE.md](VERIFICATION_SCOPE.md): carry-forward bounded sine correctness scope note for `1.5.0`
- [RUST_REPLAY_DIFF_SCOPE.md](RUST_REPLAY_DIFF_SCOPE.md): bounded Rust replay release claim and observed command behavior

Retained command outputs:

- [cargo_check_dpw4_thumb_locked.txt](cargo_check_dpw4_thumb_locked.txt)
- [kani_evidence.txt](kani_evidence.txt)
- [make_demo_evidence_package.txt](make_demo_evidence_package.txt)
- [make_doc_link_check.txt](make_doc_link_check.txt)
- [make_gate.txt](make_gate.txt)
- [make_release_bundle_check.txt](make_release_bundle_check.txt)
- [make_replay_tests.txt](make_replay_tests.txt)
- [release_reproducibility.txt](release_reproducibility.txt)
- [verify_release_repro.txt](verify_release_repro.txt)
- [replay_host_diff_identical.txt](replay_host_diff_identical.txt)
- [replay_host_diff_frame17.txt](replay_host_diff_frame17.txt)
- [replay_host_diff_missing_arg.txt](replay_host_diff_missing_arg.txt)
- [cargo_test_replay_host_operator_path.txt](cargo_test_replay_host_operator_path.txt)

Historical retained release records remain immutable:

- `docs/verification/releases/1.4.0/` remains the prior verification-depth release bundle
- `docs/verification/releases/1.3.1/` remains the prior patch-release retained bundle
- no files under older retained release directories were modified for `1.5.0`
