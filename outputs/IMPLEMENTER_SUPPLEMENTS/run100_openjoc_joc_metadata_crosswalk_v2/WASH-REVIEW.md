# Wash review: run-100 E-AC-3 JOC metadata

Evidence-only, metadata-only inspection of the hash-matched run-100 E-AC-3 output. The source package manifest SHA-256 is recorded in MANIFEST.json.

- FFprobe profile label: Dolby Digital Plus + Dolby Atmos. FFprobe did not report an explicit JOC flag or object count.
- OpenJOC 0.20.1 classified this raw E-AC-3 stream as eac3_joc by in-band syntax. It reports JOC present and successfully parsed Addbsi JOC signaling in all 4,247 AUs.
- The JOC EMDF profile is index 3, five_x_phase. OAMD contains 16 object slots; JOC coded reconstruction rows are 15; Addbsi complexity index is 16 and was validated against the OAMD object count. These are separate domains.
- Stream parsing passed with 0 malformed AUs and all 4,247 AUs decoder-admitted. ETSI strict EMDF validation failed on the recorded signaling deviations; deployed-compatibility validation passed. render_verified remains false.
- The inspected output is bound by hash/size to the run-100 result record and to an authorized sample-6 config/log/alias chain. The run manifest did not record its own input SHA; run-100 file_position is not merged with R138 embedded_timecode. Encoder completion is not confirmed.

This proves metadata-level JOC signaling for this one output file, not the entire R138 chain, render/playback, or multi-input consistency. No PCM was decoded and no EC-3 payload is included.
