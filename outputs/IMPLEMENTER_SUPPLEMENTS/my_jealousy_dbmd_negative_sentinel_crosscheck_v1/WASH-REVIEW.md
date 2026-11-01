# My Jealousy DBMD negative-sentinel cross-check

This is the third authorized ADM input vector for the candidate time helper. The source package checker passed in normal Python and `python -O`. Wash independently read only the 576-byte DBMD chunk; its hash and the type-9 sentinel field bytes match the sealed vector. No audio-data bytes were read and no raw DBMD data was kept.

The valid type-9 tuple is signed hour/sentinel -1, unsigned minutes 0, and signed little-endian ticks 0. The candidate helper accepted the typed records without decode fallback and returned StartTime=FFOA=3610.0 seconds. The negative-sentinel rule predicts the same result.

Together with sample-6 and First Love, this confirms the same negative-sentinel behavior for three authorized samples. None exercises the nonnegative DBMD delta branch; no fourth sample was selected. No DEE job or codec run occurred in this metadata-only task.
