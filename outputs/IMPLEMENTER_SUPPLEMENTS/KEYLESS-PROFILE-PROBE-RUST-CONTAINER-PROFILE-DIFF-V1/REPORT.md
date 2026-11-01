# Keyless profile probe vs Rust container profile

## Finding

Hash-bound source comparison shows a concrete container-profile difference between the keyless CRC32 probe and current Rust source:

- Python probe: EMDF CRC32 primary slot (4 bytes), no secondary protection slot; it also sets both ID11 and ID14 payload configs to `codec_data_present=true` and `frame_aligned=true`, with duplicate flags false and priority/processing fields zero.
- Rust selected profile: CRC32 primary4 plus CRC8 secondary1. Its deployed-compat config treats OAMD/ID11 as unaligned and omits duplicate/priority fields; JOC/ID14 is frame-aligned with explicit fields.
- Both source paths use a matching E-AC-3 CRC2 algorithm, and their coarse published-source mapping and carrier block 2 agree.

These are source-level differences, not a proven cause of attempt-10 or the reported Audition/FFmpeg result. The Python probe candidate's generated hash is metadata-only and was not recomputed; the user reports its audio is correct and Audition recognizes it. The probe consumes cached model EMDF and F32 audio plus an external codec; it does not compute JOC matrices or encode from the ADM. Do not call it the full ADM-to-JOC Python prototype.

The user-designated `candidate-keyless-compatible-joc.ec3` (hash `383d…`) comes from a separate source-declared post-process chain. The probe candidate `candidate-keyless-crc32-joc.ec3` (reported hash `a24b…`) is another artifact. Keep these names, inputs, and claims separate.

## What can be tested next

Use the candidate as a user-designated working-profile reference, not as algorithm truth. To isolate parseability, vary only the secondary CRC slot / ID11 config in a controlled Rust output while preserving the DEE-approved audio algorithm. Require a normal E-AC-3 parser to open the result, and compare decoded audio essence against the user-designated keyless reference. Whole-file SHA equality is not required. Do not adopt a source difference as a production rule until its profile is supported by DEE or the identified full Python prototype.

## Limits

The source matrix checker verified 14 source/metadata hashes in normal and optimized Python. It opened, decoded, played, or hashed no `.ec3`/WAV contents. User-reported playback/import outcomes are not independently reproduced. No HMAC key or secret value was read or copied. No source code or external encoder algorithm is copied into this supplement.
