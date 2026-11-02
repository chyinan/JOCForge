# Atmos JOC RenderBus selector source

This addendum closes how the processor selector is obtained, while preserving
the V2 branch rule. The renderer copies its RenderBusConfig list, finds the
entry whose vtable `+0x08` type ID is `0` (Speaker Monitoring), forwards that
object as argument five, and `FUN_180435120` reads its vtable `+0x38` getter.
The returned integer controls the already-washed branch: zero selects
SpatialCoderPlatform; nonzero selects PreProcessorBypass.

The traced `RerenderBusConfig` constructor/getter is not the selected object:
its visible type IDs are 3/4/5. Its fallback selector values 3/4/5 therefore
must not be used as the default for the selected type-ID-0 object. The
selector contract is to preserve and consume the typed `+0x38` result without
hardcoding a branch.

Neither the concrete selected type nor the selector value for either prior
ADM run is available in the receipts. Those remain UNBOUND. The conditional
dispatcher remains a usable best-effort MVP candidate; exact per-run audio
behavior is still open. This is static evidence only and includes no binary,
decompilation, audio, PCM, payload, key, or license data.
