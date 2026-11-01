# Attempt-10 parser rejection layer boundary

## Result

The user reports that FFmpeg and Adobe Audition cannot open the attempt-10 output. Preserve that as a user-reported interoperability failure; this Wash pass did not reproduce it. A previously sealed OpenJOC 0.20.1 whole-file inspection counted 4,247 frames/AUs and returned a first strict diagnostic at AU 0: decoded E-AC-3 exponent `-1` is outside the parser's allowed range `0..24`. OpenJOC's compatibility path passed, but that is not evidence of FFmpeg/Audition acceptance.

The source trace shows `INVALID_JOC_CARRIAGE` is an aggregate label around `parse_joc_access_unit` errors. The inspected call path reaches E-AC-3 audio-block exponent validation while scanning skip-field carriers. The evidence therefore points to a **candidate E-AC-3 syntax/cursor defect**, not a proven EMDF/JOC metadata defect. It does not identify the bit interval, exponent lane/block, or prove this is the cause of FFmpeg/Audition rejection.

## Evidence source

Source package: `attempt10-parser-compatibility-wash-v1`, MANIFEST SHA-256 `65bce3601c10d36d223e8a9852e4222d04cfcfbadf023f974a11a64062b52b78`. Its normal and optimized checkers both passed, verifying four sealed summaries and no included output payload.

The inspected OpenJOC executable identity is SHA-256 `2e84afcfa5e197332a84f9386ffa4c69f0c12e8ca5cf95c23690af320cfa1a39`. The source reports:

- attempt-10: 4,247 frames/AUs; strict `fail`; compatibility `pass`; 3,481 diagnostics; first diagnostic AU 0, byte offset 0, exponent `-1` out of `0..24`.
- designated comparator: 4,247 frames/AUs; strict `fail`; compatibility `pass`; no diagnostics.

The comparator's parser profile also fails strict validation, so strict failure alone is not a measure of audio corruption. The exact allowlisted metadata deviation in either file was not saved and remains unbound.

## Implementation direction

Inspect the selected Rust F12 first-AU bit-writing and cursor transition at the exponent region against the approved DEE/Python evidence, then regenerate. For every new candidate, require a normal E-AC-3 parser to open it and report the first exact syntax error if it does not; test the user's stated FFmpeg/Audition compatibility criterion on the candidate. Keep the first-AU exponent diagnostic separate from EMDF deviations. Do not use this parser observation as a replacement for DEE algorithm evidence.

No output bytes were opened or re-parsed in this Wash pass. No raw decoder report, audio, PCM, key, HMAC, or license content is included. The parser observations are not an independent proof that all AUs are damaged or valid.
