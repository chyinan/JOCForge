# Run37 ADM interval to reader and encoder boundary

Disposition: SAME-INVOCATION_SOURCE_INTERVAL_AND_FRAME_READ_OBSERVED; CLOCK TRANSFORM AND SOURCE ID UNBOUND. This is one bounded DEE reference invocation, not a current JOCForge product result.

## Provenance and verification

Source v15 manifest SHA-256: a526c30f1b086e588e0e1c42060dd494a018b1bfbb330d38134ab33ac3de9b06.
NATIVE-ADM-INTERVAL-TO-FRAME-READER-LINEAGE.json SHA-256: 1c8352df6dd06ff859b94253b10cd3a6eccf797a7d0b0f6b8ba7d308913de2a0.
Run37 job XML SHA-256: 8591f6060d85f8450ee22a94476773388380c1729a5dfab218143f647f210b8a.
Authorized input BWF SHA-256: 7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1.
Reference EC3 SHA-256: 077977564a8da534a52dee3feb1c791cc0825dd9cd7eb975575799b02450e686 (262144 bytes).
Source package verification and all member hashes passed normal and optimized Python. No PCM values, raw pointers, source, decompilation, addresses, license, or key are included.

## Same-call source interval and read

On the run37 outer source-loop call chain, a 64-byte composite-source record carried prepend=0, start=4320000, end=4418000, append=0. The four values were copied unchanged into reader-construction parameters; the native log reports the same start/end values.

The same source path constructed a reader with extent 98000 and cursor 0. The first frame requested 1536 samples; the span helper returned 1536 and the cursor reached 1536 at the encoder consumer. The same descriptor passed from the outer source loop to the encoder, which received channel-configuration count 38, sample span 1536, and event count 38.

## Unbound clock and identity edges

The same reader object's constructor calls a master-duration accessor returning 6522906, but its extent at the outer frame loop is 98000. The transformation/writer that changes this value is not identified. The numerical identity 4418000-4320000=98000 does not establish a typed clock mapping. Reader-relative cursor remains unbound to ADM sample position, host frame, or embedded timecode.

The 38 channel-configuration entries remain unbound to CHNA UID, physical track, global source ID, or selector column. Frame pointer-table writer/owner/capacity, and any generation link to U04/fit/spatial callback, remain open. Do not join this run to run24/run27/run29/run36 or the Meter Q+3 run.

The reference completed and emitted the known 262144-byte artifact, but no model comparison, U04 numeric output, Gate1, product encode, or playback is established.
