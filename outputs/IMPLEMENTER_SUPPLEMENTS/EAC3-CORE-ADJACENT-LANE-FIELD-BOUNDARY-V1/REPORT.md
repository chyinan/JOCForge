# E-AC-3 core adjacent per-lane field boundary

Ghidra 12.2 static evidence shows `FUN_14032b520` writes a 16-bit value in
`[0,3]` to `state+0x14150+2*j`, with `j=5*block+lane`. At its direct caller,
`FUN_140343530`, the same base `state+0x140f0` is later passed to
`FUN_14032d580`. The direct consumer reads D at `state+0x1418c+10*block+2*lane`,
N at `state+0x141c8+2*j`, and the exponent row; it does not read the adjacent
`+0x60` field and no direct copy from that field into D/N was found in the
bounded call path.

The adjacent value is derived from a float amplitude path and reset to zero
with the D/N grid. Its wider semantic role and later consumer remain
`UNBOUND`; the same numeric range as a selector is not enough to call it a
stride selector. A separate `FUN_14036f8f0` `param_12+0x60` access uses base
`state+0x20540`, so it is not this field.

This is E-AC-3 core-only evidence. Do not use it as the JOC per-block
selector or as a producer for JOC `FUN_180...` state. No DEE run, media,
PCM, payload, key or license access occurred.
