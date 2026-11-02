# Matched E-AC-3 DRC BSI field delta

This Wash supplement compares the same AU window from the matched full-file DEE and Rust film_light outputs. Header RF fields compre/compr and the first-block dynrng presence bit match. The first-block dynrng U8 differs in 32 of 125 AUs; the first observed difference is at AU 2843 (DEE code 1, Rust code 0).

The native writer uses separate presence and code tables for the first audio-block dynamic-range field. The table producer and numeric mode-to-profile source remain unbound. The two frame exponent strategies differ in the first differing AU, so the DRC values were found by parsing the syntax branch, not by comparing fixed bit offsets.

Decoder DRC scale 0 leaves the large channel-index-3 residual essentially unchanged (RMSE 0.00545409); these DRC bits cannot explain that residual. They may contribute to the smaller scale-dependent PCM delta, which is not isolated here.

FFmpeg 8.1 was used only as a decoder/field parser for comparison. No raw syncframe bytes, PCM, payload, key, or license material is included.
