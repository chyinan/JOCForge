# Sample-6 opener/setup pointer capture boundary

This Wash supplement records a single sample-6 full-file DEE run and the result of a runtime pointer-capture attempt. Its source manifest and all ten source-package artifacts were hash-checked; the source checker passed in normal Python and `python -O`.

The output file was present when hashed and inspected. MediaInfo 26.05 reported E-AC-3 with JOC, 1,024,000 bit/s, 48 kHz, six channels, 15 dynamic objects, one LFE bed channel, and complexity index 16. The file was deleted after metadata-only inspection. No render or playback was run; this is not output acceptance.

WinDbg attempted to observe the opener table-pointer store and the setup entry receiving `param4`, but both unqualified module-plus-relative-address expressions remained unresolved after module loading. Neither breakpoint hit. Therefore no pointer token, caller stack, or generation identity was captured. This is an instrumentation failure, not evidence that the native events are absent.

The opener-to-`param4[2]` pointer identity and same-generation join remain UNBOUND. The result duplicates the same input/output identity already recorded in the washed R3 sample-6 run; its additional value is explaining why the pointer observation failed. Keep the run evidence separate from algorithm rules.
