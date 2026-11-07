# MeteringMode static raw U32 map

Reference-only Wash supplement. It binds initializer records to raw 32-bit values only; these values are not semantic mode ordinals and do not establish measurement behavior. `IMPLEMENTATION_READY=NO` for enabling new measurement effects.

The existing sealed Ghidra initializer for `DAT_180a17e08` inserts five key/value records. Static raw entries are `1770-4→0`, `1770-3→1`, `1770-2→2`, `1770-1→3`, and `LeqA→4`. The map lookup can project a raw U32 to a processing record, but the XML StringType owner is not joined to the processing getter receiver or selected Meter/provider generation.

No branch meaning, signedness semantics beyond raw U32 storage, Meter effect, or audio effect is inferred. Runtime remains `NOT_RUN`.