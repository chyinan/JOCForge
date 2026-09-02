# JOCForge generalized decision engine

Verified 2026-09-02.

## Data flow

`file_scan::read_pcm_frame` seeks one 1536-sample frame from the BW64 data
chunk. The CLI converts selected Objects and DirectSpeakers tracks into
`ReductionSource` values, calls `reduce_scene_with_previous`, factorizes the
reduced signals for the selected 5/7-carrier profile, and runs two passes:
first to create the FFmpeg I0 carrier stream, second to package each I0 with
the source-derived D0 and JOC/OAMD metadata.

## Reduction contract

`reduction.rs` is pure. It validates finite equal-length signals and finite
positions, chooses deterministic farthest-point seeds using 16 mean-signature
bins plus spatial/RMS terms, assigns all sources, combines each cluster with
positive RMS weights, aggregates position, limits peaks, and returns per-source
accounting plus diagnostics. `reduce_scene_with_previous` reuses previous
representatives when source identities are unchanged. The active coded count
is `min(authored_count, 15)`.

## Factorization and writer contract

`factorize_source_derived` uses a 64-sample deterministic analysis view and
ridge-regularized normal equations, while carriers use the complete current
frame. Fewer objects than profile channels are supported by source-derived
extra carrier copies with zero matrix columns. The writer emits actual active
JOC object count, variable OAMD total count, full one-band matrix coefficients,
and the public coarse 96-symbol Huffman table. D0 features are bounded to
conventional 6-bit values so OpenJOC can render the output.

## Evidence

The controlled 15/16/20/28-object fixtures encode successfully. Hanabi is
38-track, 29-authored-source input (28 Objects + 1 DirectSpeakers); all three
profiles encode to 9042 AU and 60,183,552 bytes. Final OpenJOC strict inspect,
render, and FFmpeg decode all pass with zero malformed EMDF AUs. Metrics and
rendered WAVs are local under `artifacts/decision-engine/` and ignored.

Remaining clean research/test gaps are the rate sweep, moving-object corpus,
published leakage thresholds, frequency-local analysis, sparse matrices, joint
compatibility optimization, and complete non-LFE bed semantics.
