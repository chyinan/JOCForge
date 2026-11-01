# run-100 reference E-AC-3 metadata-only receipt

Run-100's evidence manifest hash-binds its job XML and DEE log. The XML's ADM input alias resolves to bytes whose SHA-256 matches the authorized sample-6 BWF; the safe log lines confirm the configured alias was the input source. A fixture-side source/copy identity record predates the job config and asserts the alias copy hash matches its source. The run manifest itself does not persist a per-run input SHA-256; that limitation is retained.

`reference.ec3` exists, and its current SHA-256 and length match run-100's result record. FFprobe 8.1 parsed format/stream metadata only. It reports E-AC-3, 48 kHz, six channels in `5.1(side)`, 1,024,000 bit/s, and profile text `Dolby Digital Plus + Dolby Atmos`. A separate JOC flag and object count were not reported by FFprobe.

Run-100's config uses `time_base=file_position`; R138's separately hash-bound config uses `embedded_timecode`. This output is scoped to run-100/sample-6 only and is not merged into R138 or treated as selected-profile acceptance. No PCM was decoded and no payload was copied into this package.
