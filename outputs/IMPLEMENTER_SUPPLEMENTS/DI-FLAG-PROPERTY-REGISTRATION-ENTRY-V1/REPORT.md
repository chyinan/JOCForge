# DI property key to bool constructor storage boundary

## Direct static registration edge

The hash-bound Ghidra 12.2 receipt traces the property key `loudness:measure_only:dialogue_intelligence` from `FUN_180136220` through `FUN_18012beb0` into `FUN_180133420`, with destination expression `param_1+0x270`.

`FUN_180133420` constructs the type label `dee::bool`, then calls `FUN_18016a3e0` with the same destination pointer. That helper initializes the `SimpleType<bool>` base table at RVA `0x647938` in the same storage; the caller then overwrites the vptr at that address with `dee::Types::Bool::vftable` at RVA `0x640fd8`. The property-key-to-constructor-storage path and same-destination base initialization are statically bound.

## Runtime owner and effect remain open

This does not prove that a later generic property-map lookup returns this object as the runtime receiver for `FUN_18014f640`. The generic-map lookup, owner/generation, and any link to measurement root `+0xf6c`, source record `+0x114`, or a DI audio consumer remain `UNBOUND`/`NOT_IDENTIFIED`. The final Bool vtable overwrite is preserved; vtable similarity is not runtime identity evidence.

The existing `FUN_18014f640 → FUN_18016a540` same-receiver wrapper edge remains unchanged. No DI A/B, DEE, audio, PCM, or payload run occurred.

## Provenance

Upstream `MANIFEST.sha256` file SHA-256 `0d6d2d10b67e27659567825fb4bec472c5bb33d3cfdfcd383104ba34db331f83`; upstream package-seal SHA-256 `9bd97339c8ab809de9b7aa820ac2693170c911db0d9b2ad6c99a23796922a8af`. Wash rehashed all seven listed files and verified all three parent manifest/seal pairs. This supplement omits the raw Ghidra trace excerpt.
