# Height trim dynamic-query input boundary

Reference-only Wash supplement. It does not establish that the height trim key is queried or that the result is the registered field instance. `IMPLEMENTATION_READY=NO`.

The `height_trim_5_1` key is statically registered at `Priv+0x9e8`. A separate DI receipt shows the generic query `FUN_18014f9f0` forms a dynamic name from a linked-record node; the known tail thunk `FUN_18013e9d0` passes RDX unchanged to that query. No code caller references the thunk's vtable slot (`RVA 0x640bf8`), so no concrete RDX source/key bytes can be tied to the height key.

The DI receipt describes a different property and does not prove the height key is queried. The returned-object identity to `Priv+0x9e8`, provider/generation, numeric conversion, unit, and effect remain `UNBOUND`/`NOT_RUN`.