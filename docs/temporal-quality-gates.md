# Clean temporal quality gates

> Verified: 2026-09-02
> Evidence: `artifacts/temporal-stabilization/hanabi-before-after.json`

## Metric definitions

`assignment_changes` counts authored source membership changes between
consecutive reductions. It is useful for detecting transition churn but is
not itself a coded-object identity change. `coded_identity_changes` compares
the representative source identities of coded objects; vector ordering alone
does not count as an identity change. Coded identity lifetime is measured over
consecutive frames for each representative source identity.

The previous clean matrix exposed only `assignment_changes` and described it
as identity churn. That was insufficient: Hanabi had 39,503 membership changes
but zero representative identity replacements. The strengthened audit reports
both values and retains transition discontinuity metrics.

## Continuity objective

For a source whose prior assignment remains available, each candidate switch
adds the clean-side dimensionless `temporal_continuity_weight` to its current
signal/spatial assignment cost. The default is `0.15`; a switch is still
allowed when the current representation is better by more than that cost. The
weight is explicit in `ReductionConfig`, can be set to zero for the pre-fix
control, and is not derived from a proprietary encoder.

## Observed before/after

| Metric | Before | After | Result |
| --- | ---: | ---: | --- |
| authored assignment changes | 39,503 | 140 | 99.65% reduction |
| changes per minute | 8,191.564 | 29.031 | materially lower |
| maximum 8-frame assignment churn | 92 | 4 | materially lower |
| minimum assignment lifetime | 1 | 3 | known one-frame failure removed |
| median assignment lifetime | 1 | 643 | materially higher |
| coded identity changes | 0 | 0 | stable representative identities |
| coded identity lifetime | 9,042 | 9,042 | stable over programme |
| maximum position jump | 2.35016 | 2.25870 | improved |
| maximum PCM boundary discontinuity | 0.83263 | 0.26301 | improved |
| maximum gain jump | 0.9999999 | 0.9999990 | not materially changed |
| maximum source reconstruction error | 0.48195 | 0.45111 | no regression; improved |
| maximum spatial representation error | 0.19697 | 0.15631 | no regression; improved |
| maximum cross-object leakage | 1.0 | 1.0 | unchanged intentional-merge diagnostic |

Root-cause counts before stabilization were 39,503 stable-representative
membership events, 2,562 source activity-boundary events, zero source spatial
motion events, 24,195 coded-position jumps greater than 0.5, and 9,924 gain
jumps greater than 0.5. This supports membership winner changes driven by
frame-local signal/activity costs rather than source movement or coded ordinal
permutation.

## Acceptance gates

- Stationary control: zero assignment churn after initialization.
- Adversarial corpus: the unregularized control must show churn, while the
  default continuity policy must reduce every case without non-finite metrics.
- Real-programme audit: representative identity changes must be explained;
  minimum coded identity lifetime must be at least 8 frames; minimum assignment
  lifetime must be at least 3 frames; median assignment lifetime must be at
  least 120 frames; maximum 8-frame assignment churn must be at most 8.
- Transition bounds: maximum PCM boundary discontinuity must be at most 0.5;
  maximum position jump must be at most 2.5; these values are above the clean
  controlled observations and below the pre-fix Hanabi failure.
- Quality: source reconstruction and spatial representation maxima must not
  exceed their pre-fix Hanabi maxima; silent drops remain zero.

The controlled and adversarial corpus pass these gates, and the stabilized
Hanabi audit passes them. This is a structural stability gate, not a claim of
production perceptual quality.
