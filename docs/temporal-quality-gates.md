# Clean temporal quality gates

> Verified: 2026-09-03
> Evidence: `artifacts/temporal-stabilization/hanabi-before-after.json`

## Metric definitions

The temporal report uses three identity domains:

- `AUTHORED_SOURCE_ASSIGNMENT`: `authored_source_assignment_changes` counts a
  source moving between coded slots. Its lifetime resets when that mapping
  changes.
- `REPRESENTATIVE_SOURCE_IDENTITY`: `representative_source_changes` counts an
  authored representative source leaving the coded representative set. Its
  lifetime tracks consecutive frames in which that source remains a
  representative, regardless of slot order.
- `CODED_SLOT_IDENTITY`: `coded_slot_replacements` counts a different
  representative occupying the same output ordinal. Its lifetime tracks slot
  presence, independently of which source represents it.

No metric uses an event from one domain with a lifetime from another.

The previous clean matrix exposed only an ambiguous assignment field and
described it as identity churn. That was insufficient: Hanabi had 39,503
authored membership changes but zero representative-source replacements. The
strengthened audit reports all three domains and retains transition
discontinuity metrics.

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
| authored-source assignment changes | 39,503 | 140 | 99.65% reduction |
| authored-source assignment changes per minute | 8,191.564 | 29.031 | materially lower |
| maximum 8-frame authored assignment churn | 92 | 4 | materially lower |
| minimum authored assignment lifetime | 1 | 3 | known one-frame failure removed |
| median authored assignment lifetime | 1 | 643 | materially higher |
| representative-source changes | 0 | 0 | stable representative-source set |
| representative-source lifetime | 9,042 | 9,042 | stable over programme |
| coded-slot replacements | 0 | 0 | stable output ordinals |
| minimum coded-slot lifetime | 9,042 | 9,042 | stable slot presence |
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
frame-local signal/activity costs rather than source movement or coded-slot
replacement.

## Acceptance gates

- Stationary control: zero authored assignment churn, representative-source
  changes, and coded-slot replacements after initialization.
- Adversarial corpus: the unregularized control must show churn, while the
  default continuity policy must reduce every case without non-finite metrics.
- Real-programme audit: representative-source changes must be explained;
  representative-source lifetime must be at least 8 frames; coded-slot and
  authored assignment lifetimes are reported separately; minimum authored
  assignment lifetime must be at least 3 frames; median authored assignment
  lifetime must be at least 120 frames; maximum 8-frame authored assignment
  churn must be at most 8.
- Transition bounds: maximum PCM boundary discontinuity must be at most 0.5;
  maximum position jump must be at most 2.5; these values are above the clean
  controlled observations and below the pre-fix Hanabi failure.
- Quality: source reconstruction and spatial representation maxima must not
  exceed their pre-fix Hanabi maxima; silent drops remain zero.

The controlled and adversarial corpus pass these gates, and the stabilized
Hanabi audit passes them. This is a structural stability gate, not a claim of
production perceptual quality.

The encode metrics sidecar additionally records the maximum element-wise
`matrix_coefficient_jump` and `carrier_coefficient_jump` between adjacent
frames, plus `factorization_reconstruction_error` and
`speaker_render_error`. The authoritative closure applies the clean bounds
`2.0`, `1.8`, `0.6`, and `0.5` respectively; missing or non-finite values fail
the run. This keeps temporal coefficient continuity measurable separately from
authored membership churn and representative identity replacement.
