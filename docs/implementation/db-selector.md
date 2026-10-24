# F09G db selector from precomputed means

This direct slice consumes exactly six precomputed F32 mean bit words and a prior U16 selector. It validates all six means as finite and nonnegative and the selector as `0..=3`. Only the first five means participate in the maximum scan; equal values select the later visited lane. The sixth (LFE) mean is checked but excluded from full-band db selection.

For the maximum mean, the helper computes headroom from its F32 exponent bits using the F09.2 rule. A zero mean or headroom at least 48 yields level `1.0`. Otherwise it performs the F09G polynomial operations in the specified order with a separate F32 rounding per multiply/add/subtract: scale, linear term, squared quadratic term, offset-minus-linear plus quadratic, result scale, and headroom step. A non-finite intermediate returns `NUMERIC_FAILURE`.

The selector compares `abs(level - anchor[previous])` strictly greater than hysteresis. If triggered, it scans from code zero and advances while the corresponding threshold is strictly greater than level; equality stops the scan. If hysteresis does not trigger, the previous selector is returned unchanged. The helper returns the proposed next selector as a value and commits no state.

The four washed `db_selector` input cases supply six mean words and a prior selector. Tests compare maximum mean, level, difference, trigger flag, and selector result; expected values are postcompute assertions only. The 17 frozen-model comparison count is review evidence for the selected package, not an implementation of the other F09G consumers.

The helper does not calculate the four-lane mean-square accumulators, gain peak summaries, delayed PCM, cross-block selector lifecycle, frame planning, or full F09G. Invalid selector or mean input maps to `EXPONENT_INPUT_ERROR`; non-finite polynomial intermediates map to `NUMERIC_FAILURE`. Missing or invalid parameter artifacts are rejected by the separately implemented policy provider.

Source package MANIFEST SHA-256: `c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a`.

Gain/db specification SHA-256: `f6e73ca2e361c06413974f4b7725fdc94f8e1d13a6a299062edde9dc3a7e6c12`.
