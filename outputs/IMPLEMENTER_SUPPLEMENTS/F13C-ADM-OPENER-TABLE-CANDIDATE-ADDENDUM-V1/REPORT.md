# ADM opener-table candidate boundary

## Scope

This addendum adds a hash-bound native static candidate path from selected ADM metadata through an opener-owned six-byte record table. It does not bind the selected reader/factory or establish that this table supplies the DDP packet rows observed in another path.

## Added evidence

- For the hash-bound My Jealousy metadata projection, row 10 identifies CHNA UID `ATU_0000000B`, TrackIndex 11, and object `AO_100B`. A targeted CHNA read covered only record ordinal 10 (96 bytes); no audio data bytes were read. The AXML values are from the previously washed metadata projection, not a new AXML read in this addendum.
- Ghidra static analysis traces CHNA UID lookup to one-based TrackIndex, then subtracts one to select a converter opener slot. In the Object branch, the candidate row stores the low U16 of an AudioObjectID-derived channel-input key at +0, sentinel `0xFF` at +2, and object ordinal at +4. The traced writer does not assign +3; its initial value is not established.
- This gives a typed ADM-side candidate producer path. It does not establish a pointer or generation join to the DDP inner-packet row table. The existing DDP row-10 observation has raw +3 `0xBA`, raw U16 +4 `0`, and normalized U16 `0x800A`, but its classifier byte at +2 was not captured. Those values and row ordinals do not identify the candidate record as the same object.

## Exact open edge

The selected `audio.adm` reader/factory is not located. The converter opener master's `this+0x50` table is not pointer-joined to setup `FUN_1802851A0` `param4[2]`, and that setup table is not same-activation/generation joined to the row pointer/count consumed by DDP `FUN_180178DC0`. The ADM identifiers therefore remain a candidate semantic carrier, not the actual native DDP row's assigned semantic name.

## Limits

Candidate vectors are synthetic and do not supply an expected DDP packet row. No DEE run, PCM/payload/bitstream read, license/key read, or U6 writer execution occurred. Bitstream emission remains `NOT_RUN`; the selected-reader-to-DDP-row identity remains `UNBOUND`.
