# speech_threshold post-open engine read boundary

Reference-only Wash supplement. It binds the threshold config copy and one process path, but does not identify an algorithmic read or audio effect. `IMPLEMENTATION_READY=NO`.

`FUN_1801a9620` copies U32 config `+0x48` into engine `+0x48` before calling `FUN_1801af230`. In the later source-config branch, 1..100 are copied unchanged to subcontext `+0x16c`; zero writes fallback 15 to source config and does not write `+0x16c`. That fallback does not show a later engine-state update.

The selected process path `FUN_1801aa080` calls `FUN_1801a8d00(engine, engine+0x2ee0)`. Its first helper reads engine `+0x08/+0x18`, and the bounded body has no direct load from its `param_1+0x48`. A same-offset QWORD at `[RCX+0x48]` uses a nested object pointer from engine `+0x2eb0`; it is not the engine U32 `+0x48`. The next unexpanded helper receives no engine+0x48 value/address. The first algorithmic read remains unbound/not identified.