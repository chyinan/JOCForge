# AtmosMezz ADM Start time output field boundary

A selected static AtmosMezz validation path parses an ADM time-token object into a double and stores it in the `AdmFileStorageDesc` Start-time field. A separately bound master writer reads that descriptor field and reports it as `Start time` in seconds. A time-token calculation that uses source sample rate also feeds a separate integer range-difference field; that calculation must not be conflated with the Start-time output field.

The exact XML element/attribute or ADM/BWF chunk that supplied the parsed time token is UNBOUND. A seven-file metadata scan observed AXML `audioProgramme@start=01:00:10.00000`, but that numeric similarity is not a typed source join. The FFOA descriptor field has no bound ADM source writer in this evidence slice; timebase origin is also UNBOUND. The previously traced `offset=auto`/`ffoa=auto` property path is downstream of these reported values.

This is static source-flow evidence only; no DEE/atmos_info execution or sample/data-chunk/PCM/payload/key/license read occurred in this slice. The included vector is an input observation only; no predicted or observed output is attached to it.
