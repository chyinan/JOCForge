# Native E-AC-3 exponent writer call-path boundary v2

## Result

The current Rust first-block source has a concrete selected example: fullband lane index 0 (`L`), strategy 1. It writes a U6 bandwidth code in the first lane pass, then U4 base exponent, 72 U7 grouped-exponent codes, and U2 gain range. Its 217 bins yield 216 differences and 72 groups. The first two group intervals from cursor `C` before the U4 base are `[C+4,C+11)` and `[C+11,C+18)`; the U2 gain field follows all 72 groups. The attempt-10 lane/strategy is not joined to this example.

The updated, hash-bound Ghidra 12.2 trace confirms the approved `dee_audio_filter_ddp_atmos.dll` image hash but follows an F32 coefficient/quantizer path; it does not reach the corresponding native U4/U7 frame-syntax writer. The `0..60` F32-derived S12 scale and the literal six passed as an item count are not evidence for U4/U7 syntax. Native writer, actual strategy/lane, cursor, and attempt-10 causality remain `UNBOUND`.

The parser-side one-group vector cannot substitute for this 72-group lane. It is a syntax boundary only, not an encoder algorithm. Do not infer a Rust defect or adopt parser code as a writer rule.

## Evidence source

Source package: `native-eac3-writer-calllineage-wash-v1`, updated MANIFEST SHA-256 `6fa7eaaa2f5d963e19de65b80ed94f15eb7a64bf2ad7035ff6d6c80ce17b0650`. Normal and optimized checkers passed. Its Ghidra path is static and did not read or parse attempt-10.

## Next closure

Find the native frame-syntax emitter that writes a typed U4 base followed by adjacent U7 groups, bind its strategy/group-count and cursor to the selected 1024 kbps frame path, and then compare to this Rust lane. Until that native writer edge is found, the current evidence supports no claim that the 72-group layout matches or differs from DEE.
