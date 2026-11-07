# Attempt-13 time_base XML value receipt

This is an addendum to the static time_base writer-gap boundary. Attempt-13's job XML contained `time_base=file_position` and the DEE process exited 0. This binds the authored XML value and process outcome only; it does not bind that value to the native DDP StringType or prove a range/seek result.

The factory default `file_position` is still distinct from a runtime field capture. `Priv+0x550` is statically connected to the same-Priv range consumer, but the accepted host XML value owner/generation has no typed writer/callsite join to that field. Keep `NATIVE_TIME_BASE_VALUE=NOT_CAPTURED`, `XML_VALUE_TO_PRIV_VALUE_WRITER=UNBOUND`, and `RUNTIME_RANGE_OR_SEEK=NOT_RUN`.