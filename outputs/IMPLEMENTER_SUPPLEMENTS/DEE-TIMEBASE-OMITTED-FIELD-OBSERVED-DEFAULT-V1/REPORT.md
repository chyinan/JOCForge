# Omitted `time_base` field: observed effective default boundary

A successful, same-attempt return-only comparator observation was collected with the `time_base` XML element absent. The comparator returned signed integer 1 twice inside two `Priv` range invocations. By the comparator contract, this means the consumed string did not equal `embedded_timecode`; it is not itself a literal string capture.

The exact currently installed XSD hash-checked in this Wash pass declares `time_base` optional with default `file_position` and allowed values `file_position` / `embedded_timecode`. The already washed DDP plugin descriptor likewise has a StringType default of `file_position`, and its setter enforces the same allowed-value list. Combined with the successful DEE run and the non-embedded comparator result, the best-supported effective value for this specific omitted-field attempt is `file_position`. This is sufficient scoped evidence for the omitted-field behavior.

The capture did not reveal whether the value came from upstream XSD default materialization or the plugin descriptor's initialized default; record `default_source_provenance=UNATTRIBUTED`. No enum-setter event was separately captured, so do not claim a direct runtime setter receipt. The observation is specific to this successful Love Song N36 job and does not bind the Priv invocation to ADM U/K/J/T/B/D generations. No conclusion is made about other jobs, parsers, or time-base modes.

The comparator observer read no string bytes, changed no arguments or returns, retained no owner pointer, and captured only its integer result plus an invocation token and input hash. The output, alias and temp were removed after the attempt. No PCM, encoded payload, key or license content was read or persisted.
