# Potential AI DEE output metadata reference

This Wash supplement contains only a hash-bound summary. The source evidence package passed its normal and optimized validators. Wash independently recomputed the authorized input and output hashes, then inspected the retained output with the user-supplied OpenJOC 0.20.1 binary in metadata-only mode.

For the exact 36-channel, 48 kHz PCM24 ADM BWF input, one full-file 1,024 kbps DEE job used `audio.adm -> encode_to_atmos_ddp`, `file_position`, empty start/end, `measure_only=1770-4/false/100`, `offset=auto`, `ffoa=auto`, and no `custom_dialnorm`. The DEE log reported encoder done and 100% audio/output progress; its process exit code was not captured. The output is 26,411,008 bytes, SHA-256 `3f9a2b8b890e12e6ee9f09de28ce9eb89626da8f2a9900a562006d661cae59c2`.

OpenJOC 0.20.1 parsed 6,448 access units with JOC payloads in all 6,448. It reports E-AC-3/JOC, 48 kHz, 5.1, 1,024 kbps, 15 coded objects/rows, complexity 16, no malformed units, continuous frame/metadata timing, and decoder admission for all access units. Deployed compatibility passed; ETSI strict failed all 6,448 AUs on the seven payload-11/14 field deviations listed in `OBSERVATION.json`. Compatibility PASS does not erase strict FAIL.

This is a DEE reference, not a JOCForge output or an audio-algorithm specification. It does not establish spatial rendering, playback, or audible quality. Render, playback, PCM listening, and bit-exact comparison remain NOT_RUN. The EC-3 payload is not included.
