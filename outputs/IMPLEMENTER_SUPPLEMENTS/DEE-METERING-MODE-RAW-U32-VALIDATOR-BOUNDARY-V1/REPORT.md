# MeteringMode raw-U32 downstream boundary

Reference-only Wash supplement. It binds one record-pointer handoff into a validator, not a MeteringMode field read or effect. `IMPLEMENTATION_READY=NO`.

`FUN_1801535b0` writes a generic raw U32 candidate at record `+0x90`. The existing caller loads this record pointer into R8 for `FUN_18014aba0`; the wrapper `FUN_180176980` does not reference R8 and forwards the record pointer to `FUN_180177230` as validator param_2. The bounded 104-instruction validator trace contains no `+0x90` access. The separately inspected `FUN_1801764e0` body also had no `+0x90` read in that scope.

This is a scoped no-read result, not a global absence claim. The first downstream typed MeteringMode consumer, selected provider/generation, and any measurement/audio effect remain `UNBOUND`/`NOT_RUN`.