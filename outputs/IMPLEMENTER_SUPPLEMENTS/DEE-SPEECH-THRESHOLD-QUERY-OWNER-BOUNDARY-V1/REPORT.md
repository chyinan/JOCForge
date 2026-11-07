# speech_threshold query-owner boundary

Reference-only Wash addendum. It narrows query and reader pointer paths but does not bind a runtime threshold or effect. `IMPLEMENTATION_READY=NO`.

The registered Integer candidate is `Priv+0x2e8` and is present in the constructor-built `Priv+0xec0` ParametersContainer candidate vector. The generic getter returns this candidate only if the current linked-record name exactly matches `loudness:measure_only:speech_threshold`; the query bytes and record-array owner/generation were not captured.

Current Ghidra xrefs bind a direct path `FUN_180179d30 → FUN_180179790 → FUN_1801792c0`, with the pointer passed through to the reader's `param_1+0x30` U32 source. There is no typed owner/generation link from the candidate `Priv+0x2e8` to that runtime reader object. The package also preserves a discrepancy: an older caller receipt names a call at `0x180178fae` to `FUN_1801792c0`, while current instructions show that site calling `FUN_1801799b0` and current references identify `FUN_180179790`. Cause unresolved; neither side is silently discarded.

Measurement/audio effect remains `UNBOUND`/`NOT_RUN`.