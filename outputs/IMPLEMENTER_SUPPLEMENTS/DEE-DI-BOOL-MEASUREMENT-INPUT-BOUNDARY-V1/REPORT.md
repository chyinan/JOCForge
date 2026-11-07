# Dialogue Intelligence Bool to measurement input boundary

## Bound candidate path

The exact-key candidate is a `SafeType<bool>` at `Priv+0x290`; its typed reader validates the object and reads `this+0x50`, corresponding to `Priv+0x2e0`. In the static `Priv::measure` route, `FUN_1801535b0` calls that reader and writes the returned byte to the measurement setup structure at `+0x94`. The same setup pointer is forwarded in `R8` through `FUN_180176980` to `FUN_1801769a0`, then passed to the first per-record consumer `FUN_1801764e0`.

## First break

The bounded direct-reader review found no `+0x94` load in `FUN_1801769a0` or `FUN_1801764e0`; the latter reads neighboring `+0x9c`. The Bool value is therefore bound into the measurement input structure, while its use by the measurement computation and effect on final `Priv+0xf6c` remain `UNBOUND`. Runtime selection of the `dialogue_intelligence` query key and owner/generation remain unobserved.

No DEE/UI/runtime run, PCM/media/payload/key/license read, or implementation access occurred.

## Provenance

Source package `di_flag_bool_measurement_consumer_addendum_wash_v1` has manifest SHA-256 `e448cfc7c496e260825b6d4dea6229f2769dc92c1adf39d869b41469dd858ffa` and package-seal SHA-256 `5219e93218a933903224c8ba3e91dabf3df8a9a8b2ae9b281fa15c1cbc7d5076`; both checker modes passed.
