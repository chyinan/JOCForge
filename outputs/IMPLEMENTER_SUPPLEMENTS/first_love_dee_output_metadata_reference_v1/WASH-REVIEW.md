# First Love DEE output metadata reference

This washed package contains a hash-bound summary only. The source package checker passed in normal Python and `python -O`; the original authorized input and the generated EC-3 were independently re-hashed while present, and OpenJOC 0.20.1 was independently run in metadata inspection mode against that output.

For the exact 36-channel, 48 kHz PCM24 ADM BWF input, the recorded DEE job used a full-file 1,024 kbps `audio.adm -> encode_to_atmos_ddp` profile with `file_position`, empty start/end, `measure_only=1770-4/false/100`, `offset=auto`, `ffoa=auto`, and no `custom_dialnorm`. The DEE log reported encoder done and both audio/output stages at 100%; its process exit code was not captured. The output was 32,964,608 bytes, SHA-256 `284fa11e3a5200a2bf6fd0eb3ccba5a11177f2c955cf8ab60ee7576f37339261`.

OpenJOC 0.20.1 parsed 8,048 access units with JOC payloads in all 8,048. It reported E-AC-3/JOC, 48 kHz, 5.1, 1,024 kbps, 15 coded objects/rows, complexity index 16, no malformed access units, and decoder admission for all access units. Deployed-compatibility validation passed. ETSI strict validation failed all 8,048 access units because of seven repeated payload-11/14 field deviations listed in `OBSERVATION.json`. These are separate outcomes; compatibility PASS does not erase strict FAIL.

This is a DEE reference artifact, not a JOCForge output and not an algorithm specification. It does not prove spatial rendering, playback, or audible quality. Render, playback, PCM listening, and bit-exact comparison remain NOT_RUN. The EC-3 is not included in this package.
