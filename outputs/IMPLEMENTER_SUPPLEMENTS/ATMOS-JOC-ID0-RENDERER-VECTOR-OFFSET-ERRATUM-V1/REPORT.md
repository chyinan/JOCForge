# Atmos ID-0 renderer vector offset correction

The two previously cited offsets are different vectors in the same
`SymphonyRenderer`, not conflicting object identities. `Renderer+0x60`,
`+0x68`, and `+0x70` form the outer vector of 0x58-byte records. The ID-0
lookup receives `Renderer+0x60` as its subobject/receiver base, but
`FUN_41f1e0` reads the nested begin/end pointers at receiver `+0x18/+0x20`;
those resolve to `Renderer+0x78/+0x80`. Capacity at `Renderer+0x88` is not
read by that scan.

Thus earlier Wash text calling `Renderer+0x60` the ID-0 vector begin pointer
is corrected: it is the receiver/outer-vector base. The actual nested lookup
range is `[Renderer+0x78, Renderer+0x80)`. This changes no type-ID, selector,
or `+0x58` producer conclusion. The concrete selected ID-0 class, its
`+0x58` writer, and per-run selector remain `UNBOUND/NO`.

Ghidra 12.2 static only. No DEE run, media/payload/key access, or
implementation contact occurred in this slice.
