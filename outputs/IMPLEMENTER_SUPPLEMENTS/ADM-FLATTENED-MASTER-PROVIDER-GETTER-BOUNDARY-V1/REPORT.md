# FlattenedMaster provider and channel-count getter boundary

## Bound resolver branch

In the statically examined param8-nonzero resolver branch, FUN_1804CBC20 constructs an atmos_storage_flatten_master wrapper through FUN_1804F5250. The wrapper stores the supplied format descriptor at +0x58 and nested provider at +0x60, then constructs FlattenedMasterSource with those same pointers at child +0x28 and +0x20 respectively.

The wrapper's vtable +0xA0 method is FUN_1804F5AF0. It forwards through FUN_1804CBBC0 to the inner provider's dynamic vtable +0xA0. That inner target and returned owner remain unbound. The runtime param8 branch is not observed.

## Typed format outputs

The child FlattenedMasterSource has statically bound getters: +0x68 returns U32 from descriptor +0x04; +0x78 returns U8 from +0x08; +0x80 returns a U16 zero-extended to U64.

Getter +0x80 branches on child+0x48. With the flag zero, FUN_180536320 reads U16 from the same format descriptor at +0x0A. With the flag one, FUN_1805890F0 supplies a pointer from child+0x38 and the getter reads U16 at that pointer +0x0A; the fallback pointer owner remains unbound. FUN_180536E40 initializes the flag to zero and can set it when its bounded channel-map pass changes or misses an entry. The runtime flag is not observed.

A parent receipt identifies the outer format +0x0A as an internal Atmos channel-count field and shows it is copied to packet +0x08. This still does not connect the descriptor to physical ADM N or a selected ADM reader/profile.

## First break

The remaining first break is the inner provider's dynamic slot +0xA0 target and returned owner, including its join to the selected reader/profile. Keep N-to-format+0x0A and N-to-core ProcessFrame M unbound. This package does not establish arbitrary-track support or a 36/38 outcome.

