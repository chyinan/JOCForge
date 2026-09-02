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
| unexplained cluster leakage | 0 | independently recomputed coded-signal residual | `<= 1e-12` | Rebuilding each coded signal from its source membership and gains must leave only floating-point residual. Intentional merge energy remains a separate diagnostic. |
| silent object drops | 0 | 0 | `== 0` | Source accounting is a hard invariant. |
| maximum coded peak | 0.0699959 | 0.0699959..0.1388048 | `<= 0.90` | Public headroom invariant from the reducer. |

The raw reducer `cross_object_leakage` values for controlled-16/20/28 were
0.0513984, 0.3138990, and 1.0. They describe intentional shared coded
representations in those fixtures and are not relabeled as unintended
leakage. The full source-to-coded contribution matrix is in
`artifacts/validation-closure/leakage-matrix.csv`.

## Independent closure metrics

The authoritative runner is `scripts/run-validation-closure.ps1`. It requires
these finite fields in every source metrics sidecar and applies the published
quality thresholds to the controlled acceptance corpus. Hanabi values are
reported as observations; they are not silently treated as a production
quality claim.

| Field | Threshold | Clean derivation |
| --- | ---: | --- |
| `unexplained_cluster_leakage` | `<= 1e-12` | Recompute coded PCM from explicit source indices and contribution gains, then divide residual energy by coded energy. |
| `speaker_render_error` | `<= 0.5` | Compare a deterministic normalized left/right position projection of authored source PCM with the same projection of reduced coded PCM. |
| `factorization_reconstruction_error` | `<= 0.6` | Apply the measured carrier/matrix pair on the deterministic analysis view and compute normalized RMS residual. The bound is above the fresh five-carrier clean baseline (`0.5286687`) with explicit structural headroom. |
| `matrix_coefficient_jump` | `<= 2.0` | Maximum element-wise delta between consecutive scaled matrices. |
| `carrier_coefficient_jump` | `<= 1.8` | Maximum element-wise delta between consecutive normalized carrier vectors; the limit follows from the 0.90 carrier peak bound. |

`cross_object_leakage` is retained as an intentional-cluster diagnostic and
does not determine acceptance. The speaker metric is a clean mathematical
compatibility proxy; it does not claim a decoded proprietary speaker-render
reference or production perceptual quality. Missing fields, missing source
rows, non-finite values, and threshold violations fail closed.

The compatibility closure also consumes
`artifacts/validation-closure/spectral-phase.json`, generated from the
repository-owned ten-fixture corpus. Its hard gates measure
magnitude-spectrum deviation, band energy, spectral tilt, transient retention,
inter-channel phase, normalized coherence, polarity, quadrature, stereo
width, and transient boundary discontinuity. Parameters and rationales are
documented in `docs/spectral-phase-quality-gates.md`; the multi-object row is
supplemental rather than a perceptual-quality claim.

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

- `SIGNAL_RECONSTRUCTION_GATE`: `PASS` when the independent source/matrix
  residuals meet the published structural thresholds; this does not establish
  perceptual E-AC-3 quality.
- `LEAKAGE_GATE`: `PASS` when the recomputed unexplained cluster residual is
  at most `1e-12`; intentional merge energy is not relabeled as leakage.
- `SPATIAL_GATE`: `PASS` for the controlled source-position metrics; no
  independent libear reference render was available on this host.
- `COMPATIBILITY_GATE`: `PASS` when OpenJOC/FFmpeg render/decode succeeds, the
  clean speaker projection is at most `0.5` for every supported profile, and
  the hard spectral/phase gates pass; these remain structural compatibility
  checks, not a perceptual claim.
