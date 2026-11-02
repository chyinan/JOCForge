# Selector-8 writer callsite boundary

## Wash disposition

The new upstream static receipt narrows the owner gap by examining one nearby `+0xa8` call and one level of its constructor path. That candidate is not the selected `atmos_storage_master_s` selector-8 writer callsite; the owner remains untyped.

## Candidate rejected as the selected writer

The candidate call is in `FUN_1804d43c0` at `0x1804d47c9`. It uses `[R12]` as receiver with `R12 = R14 + 0x10`; the return is used as an iteration upper bound. No typed identity connects this inner object to the selected `atmos_storage_master_s` instance. Its one-level helper `FUN_180281740` calls its `param_1` vtable `+0x8` and returns `param_2` unchanged; it does not identify the inner object's concrete class.

## Exact missing edge and minimum observation

Still missing: the actual caller of the selected `atmos_storage_master_s` vtable `+0xa8` writer for selector key 8, and the identity of its third argument `param_3`. The qword-0 pointee's class, constructor/first writer, and `+0x18` source remain `UNBOUND`; selected job values stay `NOT_OBSERVED`.

The minimum native observation is to capture caller, `this`, selector, `param_3`, qword-0 pointee vtable, and pointee `+0x18` at that writer entry; a one-byte write watch during construction can identify the `+0x18` first writer. This is an observation plan, not a runtime finding. No DEE run or media/key access occurred.
