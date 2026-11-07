# RF DRC StringType snapshot teardown boundary

## Bounded consumer candidate

The prior washed RF path binds a `StringType` snapshot at parser owner base `RSI+0x730`. The new hash-bound static follow-up finds `FUN_18013aad0` as a same-displacement candidate and passes `param_1+0x730` to `FUN_18013adb0`, the generic `dee::Types::Type` destructor.

This is teardown: it does not read the StringType value or write a numeric RF DRC field. The destructor receiver and the parser writer's `RSI` base are separate static bases; no pointer, owner, or generation edge proves the same object.

## First break and limits

The precise open edge is the typed owner handoff from the parser's `RSI+0x730` snapshot to this destructor candidate. Numeric RF DRC consumption remains `UNBOUND`. Other same-offset stack/indexed expressions are not treated as the same owner; this is not a global absence claim. No DEE/runtime/media/PCM/payload/key/license or implementation activity occurred.

## Provenance

Source package `dialogue_rf_drc_profile_snapshot_consumer_addendum` seal SHA-256 `82041775969d4805098d66ab31762f774d3204e1189532e305e46a08dfbdce1b`; normal and `python -O` checkers passed.
