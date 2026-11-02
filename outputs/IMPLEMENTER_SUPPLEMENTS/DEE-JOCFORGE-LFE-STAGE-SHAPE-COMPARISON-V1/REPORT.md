# DEE and JOCForge LFE stage-shape comparison

## Wash conclusion

There is now a concrete **candidate stage-shape difference**, not yet a proven numeric or audible difference. The native DEE static core uses a shared 256-word F32 transform followed by distinct LFE main/companion processing and seven-value mixers. The current Rust candidate has a four-section lane-3 LFE post-filter and a separate seven-bin F08 transform, followed by F09/F10 exponent planning and candidate-frame BAP quantization. No coefficient/state-owner mapping yet shows these are equivalent or exactly different at the sample/output level.

## Native DEE static branch

- Each internal lane runs a shared 256-word F32 block transform using symmetric even/odd pairs over 64 outer iterations.
- The LFE lane uses separate main and companion wrappers. A nonzero stride selects two 64-entry F32 tables; zero stride selects a 64-step cosine/sine recurrence. The traced wrapper then applies a 16-point butterfly-style precursor with index map `[0,4,2,6,1,5,3,7]` and distinct main/companion seven-value F32 mixers scaled by `1/16`.
- The traced code uses separate F32 SSE multiply/add/subtract operations. No explicit FMA, float-to-integer conversion, clamp, or saturation appears in these functions. Active MXCSR rounding mode was not captured.
- The shared 256-word window table hash matches the current Rust provider table hash; this one table match does not bind an input row to a native core lane or prove end-to-end parity.

## Current Rust candidate path

The hash-bound current-source audit records: ADM graph identity selects the unique semantic LFE host; Type10 row 0 carries callback `g-2`; spatial receiver lane 3 has a four-section LFE post-filter; MAIN/F07 includes LFE while F07A transient excludes it; F08 has a dedicated seven-bin transform; F09/F10 plan seven exponents; candidate frames quantize seven LFE coefficients by BAP and set `lfeon=1`. The public `encode()` path still returns `SourcePipelineUnavailable` before the source audio pipeline, so no current-product file is bound to this path.

## Actual delta still to measure

This static evidence does not bind ADM physical row 3 to the selected DEE core LFE lane, nor bind layout, LFE flag, FBW count, row stride, or selected stride branch. Native tables/mixers are not mapped to the Rust four-section filter/F08 coefficients or state owners. No cutoff or resampling ratio was identified in the traced DEE functions. Numeric parity and filter response were not run; the reported damaged EC-3 file was not bound to this source snapshot or runtime receipt.

Use the candidate stage-shape difference to drive a focused coefficient/branch comparison. Preserve the row-to-core join, selected stride, and numeric/audio parity as open; do not claim a specific audible delta from these static facts alone.
