# Helper source-time producer closure for the analyzed candidate

This Wash addendum closes the AXML start-time producer and DBMD absolute-FFOA producer for one hash-identified helper candidate. The direct helper query used an ASCII-named copy whose recorded SHA-256 matches the authorized sample-6 identity. Static source-report hashes and the package payload seal were independently checked; the source checker passed under normal Python and `python -O`.

## Candidate helper rules now bound

- `audioProgramme.start` is parsed from `HH:MM:SS.fraction` to seconds and stored as binary64 source-info StartTime. Missing start defaults to zero; invalid clock components are rejected. The fixture value `01:00:10.00000` maps to 3610 seconds and the helper directly reported 3610 seconds.
- A DBMD chunk is required. For a nonnegative decoded tuple, relative time is `hours*3600 + minutes*60 + subsecond_ticks/100000`, added to StartTime to form absolute source FFOA. A negative first tuple component retains the -1 missing sentinel. Decode failure uses the zero-initialized decoded structure.
- The helper normalizes absolute source FFOA against StartTime: negative stays -1; nonnegative becomes `max(0, absolute_ffoa - StartTime)`. Its final output is StartTime plus `max(relative_ffoa, 0)`. The intermediate DBMD tuple conversion is not the final helper output writer.

## Remaining limits

The fixture's 580-byte DBMD tuple was not decoded. Therefore the observed final pair 3610/3610 cannot distinguish a valid zero delta, missing/negative FFOA, or the decoder's zero-default path. The candidate helper image is not identified as the image loaded by the historical DEE host process. No new DEE job or codec execution ran. These rules are closed for the candidate image and fixture only; DEE host equivalence and generality remain open.

## Scope for implementation

This supplement supersedes the earlier Wash statements that the AXML-to-source-info StartTime writer was unbound, and that the intermediate tuple conversion was the final FFOA writer. It does not prove historical DEE host identity or the fixture's DBMD decoding branch. Use it as a candidate-helper metadata rule only; keep the DEE-host join and generalization explicitly open. It does not change the audio encoding algorithm or make an E-AC-3/JOC output claim.
