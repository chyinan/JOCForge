# Wash review: run-100 E-AC-3 metadata

Evidence-only supplement. Source manifest SHA-256 is in MANIFEST.json.

- The hash-bound run/config/log chain points to an ADM alias whose hash matches authorized sample-6; the source BWF was independently rehashed by Wash to the same SHA-256. The run manifest itself did not record a job-time input SHA, so provenance is BOUND VIA HASHED CONFIG/LOG ALIAS AND PREEXISTING COPY RECORD with that explicit limitation.
- The run record reports process exit observed, but encoder completion is not confirmed.
- The reference output identity matches its result record (17,395,712 bytes; SHA-256 in RUN100-METADATA.json). FFprobe metadata-only reports raw E-AC-3, 48 kHz, 6 channels, 5.1(side), and 1,024,000 bit/s. FFprobe's profile text includes Dolby Atmos, but it did not report an explicit JOC flag or object count.
- Run-100 uses file_position. R138 uses embedded_timecode. These are separate config lineages; this output does not prove R138 behavior or JOC signaling.
- No PCM decode or output payload copy/save occurred. Output path and media payload are not included.

This confirms a scoped E-AC-3 output metadata result for sample-6, not a complete ADM-to-JOC closure or verified JOC bitstream.
