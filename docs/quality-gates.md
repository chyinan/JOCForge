# Clean quality gates

> Verified: 2026-09-02
> Corpus: `artifacts/validation-closure/`
> Scope: clean controlled fixtures and the Hanabi audit. These gates detect
> structural regressions; they are not a production perceptual-quality claim.

## Derivation policy

The no-reduction control is the 15-source fixture. Reduced-scene observations
are the 16-, 20-, and 28-source fixtures. Thresholds use the maximum observed
reduced value with 25% diagnostic headroom; the purpose is to catch a clear
regression, not to define good sound. Intentional source merging is reported
separately from unintended leakage.

| Metric | No-reduction control | Reduced-scene range | Gate | Rationale |
| --- | ---: | ---: | ---: | --- |
| source reconstruction error | 0 | 0.0939073..0.3805623 | `<= 0.4757030` | Detects a clear increase over the measured N>15 baseline. |
| spatial representation error | 1.70e-17 | 0.0101535..0.1376798 | `<= 0.1720998` | Detects a clear aggregate-position regression. |
| unintended leakage | 0 | 0 after explicit merge labeling | `== 0` | Any cross-source contribution not explained by an explicit coded-cluster merge is a failure. |
| silent object drops | 0 | 0 | `== 0` | Source accounting is a hard invariant. |
| maximum coded peak | 0.0699959 | 0.0699959..0.1388048 | `<= 0.90` | Public headroom invariant from the reducer. |

The raw reducer `cross_object_leakage` values for controlled-16/20/28 were
0.0513984, 0.3138990, and 1.0. They describe intentional shared coded
representations in those fixtures and are not relabeled as unintended
leakage. The full source-to-coded contribution matrix is in
`artifacts/validation-closure/leakage-matrix.csv`.

## Temporal gate

The stationary control has zero steady-state churn. The crowded moving control
has 5 total assignment changes, a maximum 8-frame window count of 2, minimum
identity lifetime 8 frames, maximum position jump 0.4164535, maximum PCM
boundary discontinuity 0.0182486, maximum gain jump 0.1465067, and maximum
stereo-proxy discontinuity 0.0137795. The controlled corpus therefore passes
the structural temporal gate.

Hanabi is reported separately as an observed audit, not silently normalized to
the controlled threshold: 39,503 total changes, 92 in the largest 8-frame
window, minimum and median identity lifetime 1 frame, maximum position jump
2.3501615, maximum PCM boundary discontinuity 0.8326302, maximum gain jump
0.9999999, and maximum stereo-proxy discontinuity 0.4811318. This is a
research warning for future temporal stabilization, not a production-quality
claim.

## Quality result categories

- `SIGNAL_RECONSTRUCTION_GATE`: `RESEARCH_ONLY`; the metrics are reducer-side
  diagnostics and do not establish perceptual E-AC-3 quality.
- `LEAKAGE_GATE`: `RESEARCH_ONLY`; the current matrix proves explicit source
  accounting and intentional merges, but is not a decoded full-band
  contribution oracle.
- `SPATIAL_GATE`: `PASS` for the controlled source-position metrics; no
  independent libear reference render was available on this host.
- `COMPATIBILITY_GATE`: `RESEARCH_ONLY`; OpenJOC render/decode is available,
  but the complete independent speaker-reference comparison is not closed for
  every profile/rate.
