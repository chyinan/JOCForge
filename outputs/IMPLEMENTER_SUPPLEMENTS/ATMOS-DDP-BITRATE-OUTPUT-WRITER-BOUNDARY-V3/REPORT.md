# Atmos DDP bitrate type-1 output-writer boundary

## Bound mapping and consumer

Static Ghidra evidence maps `data_rate` enum codes `0..5` to core integers `384, 448, 576, 640, 768, 1024`. The 1024 value is emitted by `FUN_1801ab520` as typed field ID 3; the unit interpretation as kbps is inferred from a separate 448-kbps diagnostic, not a recovered field annotation.

The rate-bearing root table is stored at encoder context `+0x810`, then the same table is passed to `FUN_1801b1590`. Type-1 entries dispatch through vtable slot `+0x28` at `DAT_1806521e8 + 1*0x30`. All six enum codes use field ID 3 and this writer-slot path.

## Remaining edge

The indirect type-1 callback target is not resolved. Therefore field ID 3 is not yet tied to a named E-AC-3 BSI bit, `frmsizecod`, or output frame-rate field. No DEE output or frame size was observed; this does not establish that a requested rate is emitted correctly.

## Scope

Wash verified the updated source manifest and its 11 listed evidence-file hashes. The package has no separate seal or normal/`python -O` checker receipt; those are `NOT_RUN`. The supplement omits Ghidra scripts/logs, binaries, payloads, keys and licenses.
