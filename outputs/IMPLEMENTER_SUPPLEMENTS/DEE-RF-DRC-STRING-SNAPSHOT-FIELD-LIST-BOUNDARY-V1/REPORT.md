# RF DRC StringType snapshot field-list boundary

## Bound owner and pointer-list path

The RF profile StringType snapshot is created inside `FUN_180136220`. Its known caller `FUN_180135d40` passes the `Priv+0x20` subobject as the initializer owner, so the snapshot pointer is `RDI+0x750` at the caller. The caller stores that pointer in a 31-entry field-pointer array and passes the array to `FUN_18012bda0`, which allocates and copies the pointer bytes.

`FUN_18012bda0` does not dereference the RF StringType pointer or read the string value. The pointer-list copy is therefore only an opaque container handoff, not a numeric RF DRC consumer.

## Boundary

This adds an owner-bound pointer-list edge for the field address, but the later typed value consumer and numeric RF DRC effect remain `UNBOUND`. The prior same-displacement destructor candidate is not joined to this parser instance. DDPI `+0xA4` remains distinct and unjoined; line-mode `+0xA0` was not reanalyzed.

## Provenance

Source package `dialogue_rf_drc_profile_snapshot_owner_followup_addendum` has manifest SHA-256 `fc97b3e39b6649765b94529783d4631b8155a3981ba737374faa0e45cfb07672`, seal SHA-256 `ea3c76b5c26a995082d9356276d2c5f4943e7ded6c572cefb5560497c9af94aa`; its normal and `python -O` checkers passed.
