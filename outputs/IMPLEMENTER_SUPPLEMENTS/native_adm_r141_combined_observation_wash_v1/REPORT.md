# Wash review: R141 combined scalar observation

One R138-profile run was launched with the authorized sample-6 ADM using a hash-matched ASCII copy. The derived config preserves input type audio.adm, route encode_to_atmos_ddp, 1024 kbps, embedded_timecode, and the original start/end tokens.

Runtime observations:
- The embedded-timecode constructor received sample rate 48 kHz.
- A property named frame rate returned 24.0, but its join to the configured timecode_rate field is UNBOUND; do not call it the resolved R138 FPS yet.
- On the same parent record token, the serializer's +8 member was U16 16 with element count 1 and width 5. Its semantic field tag remains UNBOUND and it is NOT_ESTABLISHED to be DialNorm.
- The source-time setter was NOT_HIT_FOR_THIS_RUN. Source slots 10/11, post-write offset/FFOA, normalizer argument values, and explicit generation join were NOT_CAPTURED/UNBOUND.

The process target exited, but encoder completion was not confirmed. It created a 135,168-byte temporary output which was removed without hashing or parsing. Output metadata, JOC signaling, playback, and end-to-end acceptance are NOT_RUN/NOT_VERIFIED. This package is a bounded scalar observation, not full ADM-to-JOC proof.
