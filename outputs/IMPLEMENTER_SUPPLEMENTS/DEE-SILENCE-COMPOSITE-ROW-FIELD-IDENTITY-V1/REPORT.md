# Composite silence row field identity

Reference-only Wash supplement. It binds diagnostic labels to row offsets but does not establish any audio effect. `IMPLEMENTATION_READY=NO`.

`FUN_180184960` formats a composite row with `in: %s, prepend: %llu, start: %llu, end: %llu, append %llu`. The format arguments bind row `+0x20`/`+0x28`/`+0x30`/`+0x38` to prepend/start/end/append respectively. `%llu` establishes unsigned 64-bit formatting at this diagnostic boundary only; it does not establish upstream storage type or semantic units. The function returns a formatted string and does not write row fields.

The next unresolved question is where these fields affect Reader samples, PTS, or output length. This receipt contains only diagnostic formatting; no silence insertion or audio/PCM result is inferred.