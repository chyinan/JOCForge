# Run-100 OpenJOC JOC/Atmos metadata crosswalk V2

## Result

The run-100 FFprobe metadata reports stream profile `Dolby Digital Plus + Dolby Atmos`, but reports no explicit JOC flag and no object count. An OpenJOC 0.20.1 metadata-only `inspect --json --objects --emdf` was run on a file whose SHA-256/length exactly match run-100's `reference.ec3` output record. The original output path is not persisted in run-100's receipt; the inspected local copy is identified by content hash only.

OpenJOC classifies the raw stream as `eac3_joc` by in-band syntax. It reports `joc.present=true`, `presence_status=present`, 4,247 parsed payloads, and 4,247 AUs with successfully parsed JOC Addbsi extension signaling. The parser checks the exact two-byte Addbsi: byte 0 bit 0 is the extension flag, byte 0 upper bits are reserved, and byte 1 is the complexity index. Raw EC3 has no container `dec3` declaration in this report.

OpenJOC's EMDF JOC profile is index 3 `five_x_phase`; this is distinct from FFprobe's Atmos stream-profile label. Object metadata is reported separately from the JOC coding structure: OAMD metadata is present with 16 programme object slots, while JOC coded reconstruction rows are 15. The Addbsi complexity index is 16 and the parser validates it against the OAMD programme object count, not the coded-row count.

The parser traversed 4,247 access units with `stream_parse=pass` and no malformed AUs. ETSI strict EMDF validation is `fail` for observed signaling deviations while deployed compatibility is `pass`; neither status negates the separately observed JOC signaling. The run-100 DEE record only establishes process exit, not encoder completion; OpenJOC reports `render_verified=false`.

## Scope

The input to OpenJOC was the hash-matched run-100 EC3 output copy (SHA-256 `34d18a6fb0f477cbccd0b7d633a80b4191c0c535385ff66b8d9cbf7ff1440448`, 17,395,712 bytes). No ADM WAV sample was read this turn, and no PCM was decoded. The EC3 payload was not copied into the package; only the metadata JSON was retained.

Run-100's configuration uses `time_base=file_position`, while R138's separate hash-bound configuration uses `embedded_timecode`; these runs are not merged. The run manifest does not contain an input SHA, so run-100's authorized ADM sample-6 lineage remains bound by the existing hashed config/log/alias and pre-existing source/copy receipt, not by a per-run input digest.
