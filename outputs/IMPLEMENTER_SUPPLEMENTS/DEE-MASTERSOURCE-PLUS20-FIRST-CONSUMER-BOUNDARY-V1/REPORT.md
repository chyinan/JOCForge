# MasterSourceImpl first field consumer

Reference-only Wash supplement. It records a pointer-dispatch edge from `MasterSourceImpl+0x20`; it does not identify either field as start/end samples or establish a range/audio effect. `IMPLEMENTATION_READY=NO`.

The constructor `FUN_180536180` writes arguments to object `+0x20` and `+0x28`. It reloads `+0x20` into RCX and calls `FUN_1804cba30`; the constructor does not pass `+0x28` on that edge. `FUN_1804cba30` treats its input as `longlong*`, reads the first qword as a vtable pointer, then dispatches vtable slot `+0x70` with no explicit arguments. The concrete target owner/generation is unbound.

The separate typed Reader receipt's `startSample/endSample` fields are not joined to this object. `+0x28` first reader, duration/range meaning, PTS, and audio effect remain unbound/not run.