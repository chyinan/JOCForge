# C5/D3 temporal stabilization remediation

Verified 2026-09-03.

## Metric validity

Temporal evidence uses three explicit domains. `authored_source_assignment_changes`
measures authored source membership changes (`source_index ->
 coded_object_index`); `representative_source_changes` measures representative
source identities entering or leaving the representative set;
`coded_slot_replacements` measures a different representative occupying the
same output ordinal. Authored assignment, representative-source, and coded-slot
lifetimes are reported separately. The sidecar records each coded object's
representative source index so all domains are reproducible.

## Root cause

Hanabi's previous-seed path preserved the 15 representative sources, but
`assign_sources` selected each non-seed source from the current frame's
signal/spatial/energy cost only. Frame-local activity and signal changes caused
membership winner swaps even though source positions were static. Before
stabilization: 39,503 membership changes, 2,562 activity-boundary events, zero
source-motion events, and zero representative identity changes.

## Continuity policy

`ReductionConfig.temporal_continuity_weight` adds a dimensionless additive cost
when a source switches from its prior-frame cluster. The default clean value is
0.15; weight zero is the explicit pre-fix control. A switch remains possible
when the current representation cost improvement exceeds the continuity cost.

## Evidence

The regenerated before/after Hanabi audit is in
`artifacts/temporal-stabilization/hanabi-before-after.json`. Authored-source
assignment changes fall from 39,503 to 140, maximum 8-frame churn from 92 to 4,
minimum assignment lifetime from 1 to 3 frames, and median assignment lifetime
from 1 to 643 frames. Representative-source changes and coded-slot replacements
remain zero; representative-source and coded-slot lifetimes span all 9,042
frames. Reconstruction and spatial maxima improve. All three regenerated
profiles pass pinned OpenJOC strict inspect and FFmpeg decode, and deterministic
repeat is true.

The adversarial corpus covers near-equal costs, boundary crossing, alternating
levels, temporary silence, correlated divergence, and separated-to-correlated
sources. The unregularized corpus produces churn; the default policy reduces
all six cases to zero authored-source assignment changes, representative-source
changes, and coded-slot replacements without non-finite output.

## Review and verification

Fresh delegated review timed out twice, so a manual adversarial review was
performed. It found zero Critical and zero Important issues; the one validation
script strict-check false-positive risk was fixed before the final run. Final
engineering commands all pass: `cargo fmt --all -- --check`, workspace check,
clippy with `-D warnings`, 46 workspace tests, and `git diff --check`.
