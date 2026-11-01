# Sample-6 DBMD negative-FFOA sentinel cross-check

This package updates the previous candidate-helper boundary where the fixture DBMD tuple was `NOT_CAPTURED`. Wash independently read only the RIFF header/chunk headers and the 580-byte DBMD chunk from the already-authorized, hash-identified sample. The DBMD payload hash matched the sealed source vector; no audio data bytes were read and no raw DBMD bytes were persisted.

The type-9 tuple decodes to signed hour/sentinel `-1`, unsigned minutes `0`, and signed little-endian subsecond ticks `0`. The accepted record sequence is 7/9/10 and all three checksum results passed. The candidate helper therefore took its negative-sentinel path, suppressed the absolute source FFOA write, retained the constructor default `-1.0 s`, and normalized the final helper FFOA to 3610.0 s for StartTime 3610.0 s. This explains the candidate helper's exact StartTime/FFOA pair for sample-6; the tuple was not a decode fallback.

## Remaining limit

The candidate helper hash remains UNBOUND to the helper image loaded by the historical DEE host. Keep the result candidate-helper-specific and sample-specific. It does not close audio encoding, E-AC-3/JOC syntax, spatial rendering, or the helper-image join; no DEE job ran in this input-vector verification.
