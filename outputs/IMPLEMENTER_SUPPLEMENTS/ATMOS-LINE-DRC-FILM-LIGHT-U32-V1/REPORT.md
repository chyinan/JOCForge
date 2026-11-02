# Line DRC film_light profile code

## Wash disposition

The upstream R5 initializer addendum closes the previously unknown `film_light` profile-tree U32 from a DLL-local CRT initializer. The exact value is `0`. The downstream Line DDP_COMPR join remains unbound, so this does not yet identify the encoder field value at the final consumer.

## Static construction

- CRT process attach calls the initializer-array dispatcher; entry 25 points to `FUN_180001510`.
- `FUN_180001510` initializes the DLL-local profile tree at `0x180a17cb0`, allocates its empty-tree sentinel, and inserts six embedded key/U32 pairs.
- The `film_light` pair uses U32 `0`; the node allocator copies the key to `node+0x20` and the U32 to `node+0x40`.
- The previously traced `FUN_1801535b0` copies a matching node's U32 to `param_2+0xa0` only on a non-sentinel exact string match.

## Remaining value bridge

The static evidence still does not connect `FUN_1801535b0 param_2+0xa0` to `FUN_180178dc0 param_1[1]` or the Line DDP_COMPR value written by `FUN_1801aa730` to `param_3+0x28`. Keep that same-value/downstream link `UNBOUND`; do not conflate profile-tree code, raw Line enum, or encoder config field. No DEE run, PCM, key, or license access occurred.
