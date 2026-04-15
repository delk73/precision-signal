# NON-NORMATIVE / EXPERIMENTAL

# Residue Boundary Note

This note compresses the WIP-007 through WIP-012 findings into one reusable
experimental reference. It is not part of the formal release contract, is not
enforced in runtime or CI, and does not promote this rule into the spec surface.

## Rule

For the checked-in quantization probe, the compact predictive rule is:

```text
predicted_first_divergence_frame(corpus, q)
= min i such that ((5 * corpus[i] + 3) & ((1 << q) - 1)) != 0
```

This rule is validated for the checked-in quantization probe. Its primary fit
region is `q in {2,3}`.

## Mechanism Summary

The current probe pipeline can be summarized as:

- transform: `t_i = 5 * sample_i + 3`
- quantized transform: `t'_i = (t_i >> q) << q`
- residue: `r_i(q) = t_i mod 2^q`
- accumulation gap: `A_i - A'_i = sum_{k<=i} r_k(q)`

Within the tested domain, the dropped residues are non-negative, so the prefix
gap is monotone. The first non-zero residue is therefore the first frame where
the accumulator inputs differ, and in the bounded tested domain that first
possible divergence is also the first observed divergence.

## Bounded Validity

This is a host-derived explanation, with supporting evidence from host and BBB
parity where applicable. It is only claimed for the current checked-in probe
pipeline and the tested domains covered by the existing WIPs:

- `C1`
- `C2`
- hand-designed corpus family
- exhaustive bounded adversarial domain `{1,6}^12`

The predictive rule is only claimed in the tested fit region `q in {2,3}`. This
note does not claim universality outside the current pipeline, outside the
tested domains, or for wider `q` values.

## Evidence Ladder

- `WIP-007`: local boundary mapping on `C1`
- `WIP-008`: cross-corpus behavior on `C1` and `C2`, including host/BBB parity
- `WIP-009`: residue-index predictive rule
- `WIP-010`: hand-designed falsification pass
- `WIP-011`: exhaustive bounded adversarial search over `{1,6}^12`
- `WIP-012`: mechanism derivation explaining why the rule matches the observed
  boundary in the tested domain

See [docs/wip/experiment_log.md](../../wip/experiment_log.md) for the underlying WIP entries.

## Non-Promotion Note

This artifact is experimental and non-normative. It is not part of the formal
release contract, is not enforced in runtime or CI, and should be treated as a
bounded explanatory note rather than a promoted specification claim.
