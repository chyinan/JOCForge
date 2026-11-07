# DEE U group-entry to K six-byte row writer

## Bound selected writer path

The selected opened-source writer derives K's U16 count header from the existing U group/individual count helper and allocates `6 * K_count` bytes for six-byte rows, with a high-half overflow fallback. Its group loop emits one row per accepted `U Bed::Channel` entry. The destination slot is the already-zero-based physical track index in `Bed::Channel+0x10`.

The observed row stores are:

- `+0`: low U16 of `Bed::Channel+0x08`; semantic name not assigned.
- `+2`: low U8 of a converted I32 label from `Bed::Channel+0x0C`; unsupported `-1` is rejected before the store, but the conversion table is not reconstructed.
- `+3`: no store in this body; initial value not evaluated and must not be filled with an assumed zero.
- `+4`: low U16 of the outer zero-based group ordinal, not the inner entry ordinal.

## Limits

The parent receipts statically bind CHNA UID → physical slot → U `Bed::Channel+0x10`, and the current package binds one accepted group entry → one K row for this writer path. Actual K values/runtime branch, all-row admission for any file, semantic meaning of `+0x08`, byte `+3` initialization, full label conversion table, and exact K group-row projection remain partial or unobserved. The downstream `1..128` gate is separate from this writer's U16 narrowing; there is no local `1..128` predicate in this selected body.

No DEE run or media/PCM/payload/key/license read occurred.

## Provenance

Source package `ADM-U-GROUP-K-ROW-WRITER-V1` manifest SHA-256 `cf865e0eb2617a25a8d2619b9a381c80b8aff8034ca90854f7ad8d1bc86a65cf`; normal and `python -O` checker passed.
