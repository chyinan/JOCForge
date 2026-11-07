# StageRunner observed property-string boundary

## Disposition

This is a reference-only Wash supplement. It is not an implementation contract and does not close `time_base` input or audio behavior. `IMPLEMENTATION_READY=false` for this scope.

## Bounded facts

For the four attempt-13 `SetProperty` calls that were actually observed, the hash-bound static callers assign these keys: `dee_audio_source_type`, `dee_audio_target_type`, `dee_audio_filter_source`, and `dee_audio_filter_target:<index>`. None is `time_base`.

In these four observed calls, the helper consumes the key and value arguments as a 32-byte string-like object: length qword at `+0x10`, capacity qword at `+0x18`; capacity below `0x10` uses inline bytes at the object base, while capacity at least `0x10` uses an external-data pointer at the base. The helper writes a NUL at `data+length`. The exact failure exception from the earlier direct UTF-8 read was not captured. Reading the object base as a C string is not a valid interpretation for the observed external-storage keys.

Attempt-13's four time-base-hook events all have `property_key_status=READ_FAILED` and `property_value_status=SKIPPED_KEY_UNREADABLE`; no native time-base value was captured. The four static callers above do not identify a time-base producer, and their value fields are not joined to an owner/generation.

## Remaining boundary

The actual `time_base` producer/callsite and a typed join to the component's time-base field remain `UNBOUND`. The inline-only read proposal in the source research package is `PROPOSED_NOT_RUN`; external storage is explicitly not read. Attempt-13's same-process ADM observations do not join `time_base` to ADM U/K/B/D owners or generations. Preserve `NOT_CAPTURED` and `UNBOUND`; do not infer physical track count from the U container count.

No source/decompilation, input media, PCM, encoded payload, private key, or license is included here.
