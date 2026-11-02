# Line DRC profile tree value and encoder input boundary

## Wash disposition

The updated upstream package verifies the Line string-to-tree lookup and downstream config field separately. It does not establish a numeric profile value for `film_light` or a typed/value join from the lookup result to the encoder's Line DDP_COMPR input.

## Static facts

- The static image's profile-tree global at `0x180a17cb0` is zero-filled in the inspected image; no `film_light` profile node/U32 is available there. The string literal `film_light` is a default string, not a numeric tree entry.
- `FUN_1801535b0` writes a U32 only on a non-sentinel tree hit followed by an exact string match. It copies `node+0x40` to `param_2+0xa0`. A miss skips this assignment; this function does not provide a profile U32 for that input on the miss path.
- The separately traced Line converter reads a raw enum from `FUN_180178dc0 param_1[1]`; the encoder configuration writer `FUN_1801aa730` writes a different `param_1[5]` U32 to `param_3+0x28`.
- No direct assignment/argument edge joins `FUN_1801535b0 param_2+0xa0` to `FUN_180178dc0 param_1[1]`, and no `film_light` U32 is established.

Do not infer a numeric value from enum declaration order, equal offsets, or the candidate table. This receipt does not authorize an audible DRC profile mapping. No DEE runtime, PCM, key, or license access occurred.
