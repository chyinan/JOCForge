# Line DRC serializer first-read boundary

Reference-only Wash supplement. It records the first input-side pointer read in one serializer body; it does not identify a DRC field, BSI write, or audio effect. `IMPLEMENTATION_READY=NO`.

The bounded Ghidra read of `FUN_1802001a0` stops at its first non-stack/non-RIP access: `MOV RCX,qword ptr [RCX+0x42f0]` at `0x1802001be`. This is a 64-bit pointer-width read from caller object `+0x42f0`, with no semantic field type established. The direct caller's RCX and derived RDX record pointer are not joined to snapshot `+0xA0`. No output write occurs before the stop; serializer write width is `NOT_REACHED`.

The +0x42f0 pointer is not credited as DRC data. Snapshot+0xA0 owner/generation, BSI/coded field, runtime activation, and audio effect remain `UNBOUND`/`NOT_RUN`.