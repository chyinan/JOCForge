# Human Cat DEE/Rust per-block `dynrng` comparison

A parse-only comparison of the matched 4,247-AU E-AC-3 outputs traversed all six audio blocks per AU. The `dynrnge` presence bit matches in all 25,482 paired fields. Among present fields, 10,284 U8 values differ across the full file. In AU interval `[2750,2875)`, all 750 presence bits match and 194 U8 values differ; every differing pair is DEE `0x01` versus Rust `0x00`.

The comparison is a bitstream-field observation, not a native producer trace or an audible-cause finding. The matched DRC writer package separately binds the block code/presence rule, but the actual selector, float inputs, and reference codes for these AUs remain unobserved. The large channel-3 PCM residual persists at decoder DRC scale 0, so these `dynrng` differences do not explain that residual. Their rendered effect when decoder DRC is enabled has not been isolated.

The parser buffered one syncframe at a time in memory, parsed only presence/U8 fields, and skipped mantissa spans using BAP-derived lengths. It produced no PCM and retained no syncframe or encoded payload. The counts and histograms are retained; raw media and parser source code are not.
