# DDP bitrate type-1 writer registry boundary

## What is bound

Static Ghidra traces bind `data_rate` codes `0..5` to core integers `384, 448, 576, 640, 768, 1024`. The value is passed as field ID 3 in a type-1 core table. `FUN_1801ab520` stores the table wrapper at encoder-context `+0x810`; `FUN_1801ab800` later passes the same root table to `FUN_1801b1590`, which dispatches type-1 entries via vtable slot `+0x28` at `DAT_1806521e8 + 1*0x30`.

## Registry initialization boundary

The bounded Ghidra scan covered the DLL CRT `_initterm` range (`0x18063a540..0x18063d098`): 1,387 slots, 1,292 resolving to function targets, with zero direct xrefs from those initializer targets to the type registry. The type-1 row pointer at RVA `0x652218` is zero in the on-disk static image. That is **not** proof that runtime leaves it null; the callback may be installed elsewhere or created dynamically.

The type-1 callback target remains `UNBOUND`, as does field ID 3's named E-AC-3 BSI/frame field. Actual output bitrate/frame size is `NOT_RUN`. All rate enum cases use the same field/type/serializer route in the static report, but exact emitted rates are unobserved. The unit label `kbps` remains inferred.

## Scope

Wash independently recomputed the current source manifest hash and 18 evidence-file hashes. The source package has no separate seal or checker receipt. No DEE run, payload/PCM read, key/license access, or implementation source change occurred.
