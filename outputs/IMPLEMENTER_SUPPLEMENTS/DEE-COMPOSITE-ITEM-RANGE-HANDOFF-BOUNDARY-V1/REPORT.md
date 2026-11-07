# Composite item start/end range handoff

Reference-only Wash supplement; it records the selected-source item list and first range-object candidate, but does not join the row to typed sample fields. `IMPLEMENTATION_READY=NO`.

The reused pointer chain is `FUN_180553a60.param_2 → FUN_180550800.param_2 → FUN_1805506b0.param_2`; the selected source-adapter collection is a J vector with 0x58-byte entries and per-entry master/source descriptor K. `FUN_1805506b0` forwards the same list to `FUN_18054a900`.

`FUN_18054a900` walks J rows and loads qwords 0..10. The `[5]`/`[6]` loads are candidates only; the function does not type them as `startSample/endSample`. Its first `MasterSourceImpl` initializer writes local values to object `+0x20/+0x28`, sourced from row `[0]/[1]` in the default branch or a map path in another branch; their sample-range meaning is unbound.

A separate Reader receipt labels `FUN_1801831c0` input `param_2[5]/[6]` as `CompositeMasterItem.startSample/endSample`, but no owner/generation join connects that Reader vector to the J row list. Stop at this first missing typed join; do not equate equal row indices or values. No duration/PCM/PTS effect is claimed.