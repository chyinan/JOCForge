# F13C dmixmod stored-field lineage boundary

## New typed-object fact

Ghidra binds the parser handler `FUN_1403f4e90` to an object constructed by `FUN_1403f1dd0` (vptr `0x140bcf690`), at slot `+0xa8`; it receives a nested configuration block at object `+0x70`. The prior parser trace places a helper result in object `+0x1b0`, but that slot's semantic type remains unbound.

The same callback object has separate getters: `+0x28` reads a double from `+0x58`, `+0x38` reads a double from `+0x60`, and `+0xb0` returns the `+0x70` subobject. `FUN_1404813f0` uses those distinct values to write `+0xf0/+0xf4/+0xf8`; this route does not consume the parser result at `+0x1b0`. A later per-item child collection reads from a separate item pointer and is not a substitute.

## Still unknown

No typed `ddp_dmixmod` value or consumer is bound from `+0x1b0` to a BSI/side-info writer or independent downmix renderer. Thus LoRo/LtRt remains unclassified as audio-changing, metadata-only, or ignored. The same-ADM single-field A/B remains prepared but `NOT_RUN` while R4's DEE observation gate is active.

## Scope

Source package manifest/checker normal and `python -O` verification passed. The installed image hash was revalidated against the Ghidra project record. Static only: no DEE execution, PCM/media/payload, key/license, or implementation source access.
