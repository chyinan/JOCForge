# Keyless comparator Python source-chain boundary

## Result

A candidate-specific Python **file-derivation** chain is declared for the user-designated comparator. This is not a sealed run receipt and does not establish a complete ADM-to-E-AC-3 encoder prototype. The candidate-specific script starts from a pre-encoded model E-AC-3 stream, then declares metadata/frame edits and CRC repair; the record does not bind that source stream or the Python run to the user-confirmed ADM. The missing external CRC2 helper also prevents a complete implementation comparison.

## Candidate identity and lineage status

The designated comparator identity remains the prior Wash-bound SHA-256 `383d7de4a57fd9bb32b8f871af1d2b5b0ad686a4b25e5fc6e51cd1be3adf6e86`, length 17,395,712 bytes. The user confirms the ADM identity SHA-256 `7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1`. The source matrix does not access either media file.

The sealed source package `keyless_prototype_comparator_source_matrix_wash_v1` (manifest SHA-256 `c80f5b3aefd79f990804eb3cf711b42911207e0b24c57bf35415752e6ff97853`) passed normal and optimized checks over 22 source/metadata evidence files. Its source-declared chain is:

`isolate_joc_recognition.py` → reported `auth-crc32-and-zero-padding` variant → `verify_compatible_keyless_probe.py` copy/hash assertion → designated comparator path.

This chain is **not execution-sealed**: the generation record does not include source hashes/run ID, the Python run-to-ADM relation was not captured, and the referenced external helper modules were unavailable. A separate `keyless_profile_probe.py` reports a different SHA and primary-only CRC profile; it is not the designated comparator generator.

## Source-level comparison, conditional only

- **EMDF protection:** candidate-specific Python source declares CRC32 primary4 plus CRC8 secondary1; current Rust F14K source uses CRC32/ISO-HDLC plus CRC8/SMBUS. This is a source-level profile match only, not a bound run or byte-level verification.
- **Frame CRC2:** Python calls an unavailable external `regenerate_crc2` helper after replacement and zero-padding; Rust computes CRC16 and checks a zero residual. Exact equivalence is unverified.
- **First published AU:** Python consumes published allocation records and reported frame/carrier metadata; Rust suppresses sources 0/1 and maps source 2 to published AU 0. No sealed mapping connects the candidate-specific Python source2 to AU0.
- **Validity:** neither source comparison nor OpenJOC compatibility counters establish FFmpeg/Audition acceptance. The user-reported attempt-10 import failure remains a user observation; this package does not inspect its bytes or infer a root cause.

## What to retrieve next

The smallest useful closure is the candidate-generation receipt plus the exact hashes of the pre-encoded source stream, Python entrypoint, external helpers, and frame-allocation metadata, all bound to the same run and user-confirmed ADM. If the source is only a post-process over pre-encoded audio, record that explicitly: it is not evidence of full ADM-to-JOC encoding. Then compare a newly generated Rust file's parseability and decoded audio essence against that exact keyless candidate under the selected 48 kHz / 1024 kbps / 5.1 profile. Do not require whole-file SHA equality when the only intended difference is private authentication; do require a normally parseable stream and measured audio equivalence.

No Python/Rust code, helper implementation, `.ec3`/WAV bytes, PCM, key, HMAC, or license material is included in this Wash package.
