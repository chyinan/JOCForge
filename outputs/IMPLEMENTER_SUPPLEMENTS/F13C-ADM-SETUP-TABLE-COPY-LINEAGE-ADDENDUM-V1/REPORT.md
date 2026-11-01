# ADM setup-table copy lineage and DDP identity boundary

## Scope

This addendum resolves the selector-3 opener-to-setup table path as static data lineage. It corrects only the earlier uncertainty about whether the opener table and setup input were the same pointer: they are distinct allocations linked by ordered copies. It does not connect the setup-owned table to a selected DDP packet row or owner generation.

## Bound static lineage

For selector 3, the opener writes the CHNA-derived six-byte table at opener master `+0x50`. The parent setup stage clones its header and a new `count * 6` table. The clone copies each record's first four bytes and final two bytes at the same ordinal. The selector-3 callback forwards the cloned header as `param4`; setup `FUN_1802851A0` consumes `param4[2]` and initially copies those records at the same ordinals into its own master `+0x50` table. These are separate allocations, not one shared pointer. Setup can later rewrite the final U16 for Bed records.

This narrows the previous Wash status: opener table pointer identity with `param4[2]` is **false**, while the selector-3 record-copy lineage is statically bound. It does not prove runtime owner/generation identity.

## Exact remaining join

The selected `audio.adm` reader/factory remains unbound. No same-activation owner/generation token links setup's table to the DDP inner packet at `FUN_180178DC0` and the normalized row input at `FUN_1801AC670`. `AO_100B` remains an ADM-side candidate only; the observed row-10 scalar values and the normalizer's uncaptured `+2` classifier byte do not establish identity. Candidate vectors intentionally return `NOT_SUPPLIED` for actual DDP packet rows.

No runtime capture or DEE job was run; no PCM, media payload, key/license, or bitstream bytes were read. Bitstream emission remains `NOT_RUN`.
