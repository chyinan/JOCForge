# DEE 6.7 ID14 index-1 conditional state edge

Wash verified a candidate-specific static condition in the Atmos plugin:

- FUN_1801bfdb0 writes parent field +0x1832c=1 when its param_10 equals 0x15.
- The direct state initializer FUN_1801f7840 copies that U32 into constructed state +0x0c and publishes the state pointer at parent +0x18318.
- FUN_1801f6ca0 reads state +0x0c. The value-1 branch stores state +0x24c=1 after the current ID14 writer; the writer reads the current +0x24c snapshot and emits it as U3.

This is a static conditional rule for a future ID14 value. It is compatible with the official Flat7x reference index 1, but param_10 semantics, the selected task/config owner, current invocation value, and same-generation join to the target D0 output remain UNBOUND. Keep seven coefficient channels, eight container track channels, E-AC-3 chanmap locations, ADM physical rows, and 15 JOC objects as separate domains.

No code, media, payload, PCM, key or license is included. Runtime and input vectors are NOT_RUN; production_allowed=false.
