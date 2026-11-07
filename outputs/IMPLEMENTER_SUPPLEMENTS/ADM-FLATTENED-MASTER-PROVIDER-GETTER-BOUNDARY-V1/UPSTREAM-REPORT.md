# FlattenedMaster provider getter boundary

**Recipient:** Wash only  
**Result:** `RESOLVER_FACTORY_WRAPPER_TYPE_BOUND; OUTER_SLOT_A0_FORWARDER_BOUND; FLATTENED_SOURCE_U16_GETTER_BOUND; INNER_PROVIDER_TARGET_AND_SELECTED_READER_JOIN_UNBOUND`

## Reused provider boundary

This addendum reuses the hash-bound N-to-M first-break package and the prior `CompositeMaster` provider-slot receipts. It does not redo the ADM metadata vectors, packet writer, ProcessFrame M consumer, or key-1/key-2 configuration paths.

## Concrete resolver-factory branch

In the `param8 != 0` branch of `FUN_18054A900`, `FUN_1804CBC20` creates a `0x68`-byte `atmos_storage_flatten_master` object through `FUN_1804F5250`. The constructor installs vtable `0x1807FD688`. Its slot `+0xA0` is the concrete `FUN_1804F5AF0` method.

The factory returns the wrapper pointer and a companion pointer to `wrapper+0x18` (`FUN_180420580`). During setup, `FUN_1804F5B00` stores the supplied format descriptor at `wrapper+0x58` and the supplied provider pointer at `wrapper+0x60`. It also constructs a `FlattenedMasterSource` from those same two pointers. `FUN_180536180` stores the provider at `FlattenedMasterSource+0x20` and the format descriptor at `+0x28`.

`FUN_1804F5AF0` forwards slot `+0xA0` to `FUN_1804CBBC0(wrapper+0x60)`, which dispatches to the inner provider's own virtual slot `+0xA0`. The first downstream record is three qwords in the existing map-writer path; qword 0 is treated as an owner pointer. The concrete inner-provider vtable target and its owner type remain `UNBOUND`. The `param8 != 0` resolver branch is static; the runtime `param8` selection is `NOT_OBSERVED`. The `param8 == 0` direct-source branch remains a separate candidate.

## First typed outputs and format+0x0A

The constructed `FlattenedMasterSource` vtable is at `0x180806CD0`:

- slot `+0x68` returns a U32 from the descriptor at `+0x28+0x04`;
- slot `+0x78` returns a U8 from the descriptor at `+0x28+0x08`;
- slot `+0x80` returns a U16 value zero-extended to U64.

For the `+0x80` getter, the `FlattenedMasterSource+0x48` flag selects the source. When it is zero, `FUN_180536320` reads the U16 at `FlattenedMasterSource+0x28+0x0A`. When it is one, `FUN_1805890F0(child)` supplies a pointer from `child+0x38` and the getter reads that pointer's U16 at `+0x0A`; the pointer's producer/owner is outside this closed edge. The flag starts at zero and `FUN_180536E40` sets it when its bounded channel-map pass changes the map buffer or cannot find a requested entry. The actual flag is not observed.

`FUN_1804F5B00` stores these typed getter values in the wrapper's metadata record at `+0x1C` (U32), `+0x20` (U8), and `+0x22` (U16). On the flag-zero branch, the descriptor pointer is statically the same pointer held at `wrapper+0x58` and `FlattenedMasterSource+0x28`.

The U16 at the outer format descriptor `+0x0A` is an internal Atmos channel-count field: `FUN_1801852E0` checks it for zero and uses it to size per-channel state, and `FUN_180186870` copies it into packet `+0x08`. This identifies the field's plugin-local domain. It does not bind that descriptor or the nested provider owner to the selected ADM reader/profile or physical track count N.

## Exact first break

The wrapper's slot `+0xA0` is resolved to a forwarding method, but the inner provider call at `FUN_1804CBBC0` remains virtual. Its concrete target and returned owner are not statically joined to the selected host reader/profile. In the resolver branch, the format descriptor and nested provider arrive as separate source-record arguments; their parent/task generation and relationship to an ADM reader remain `UNBOUND`. Therefore the typed getter clarifies the internal U16 channel-count field without closing the requested owner lineage.

## Scope

Ghidra 12.2_PUBLIC inspected the hash-matched Atmos DLL project read-only with analysis disabled. No DEE process was run. No PCM, media payload, EC-3 payload, key, or license material was read. No host reader/profile selection or runtime provider target was captured.
