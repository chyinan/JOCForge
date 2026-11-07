# Atmos DDP silence-duration sample boundary

The selected Atmos DDP stage declares prepend/append silence duration as strings with descriptor default `0.0`. In the inspected numeric-seconds path a nonnegative value is parsed as double, multiplied by source sample rate and converted to a signed integer with truncation toward zero. At 48,000 Hz `0.00390625` seconds gives 187.5 before conversion and 187 samples after conversion. This differs from the separate start/end range-validation rounding path.

An integer frame form using a lowercase `f` requires an indicated frame rate; fractional frame counts and negative numeric values are rejected on the inspected path. The two counts are copied to separate per-source encode records. Host omission overlay, exact native zero-fill location, output length delta, PTS/clock effect, and nonzero runtime behavior remain unobserved; prefix/suffix zero-fill and length addition are model inferences, not native-run results.

Zero-value configurations existed in prior DEE runs, but do not validate the nonzero path. The input-only contrast included here was NOT_RUN. No raw audio, source, decompilation, payload or key is included.
