# JOC per-block U7 selector: profile-slab boundary update

This revision adds the bounded profile-copy follow-up and supersedes V1 for
the selected-profile source boundary. `FUN_1800a3710` still consumes inline
row 2 at `state+0x141d4`; the selector remains a 16-bit code with lookup
`[0,1,2,4]` (0 disables, 1/2/3 select strides 1/2/4). The constructor
defaults rows 0 and 1 to zero; rows 2 through 5 remain `UNBOUND`.

The closest profile path is `FUN_180074980 -> FUN_1800a1d00`. It passes the
selector component `state+0x140e8`, profile source
`*(state+0x298c8)+0xf5b8`, and a descriptor-selected slab pointer derived
from `state+0x298d0` in 0x400-byte units. The helper copies into that separate
slab and installs component pointers at `+0x1e0`, `+0x2d0`, and `+0x508`; it
does not write inline selector row 2. The alternate call has the same
destination/source relation. Slab index is not block index.

Another vector at `state+0x13608` is produced from other state fields and
consumed by `FUN_18009f550`. It can contain 4, while the inline U7 selector
lookup accepts selector codes 0 through 3. No typed join to the inline rows
was found; do not use it as a substitute for row 2.

Therefore the selected-profile per-block values remain
`[0,0,UNBOUND,UNBOUND,UNBOUND,UNBOUND]`. The one-hot `[0,0,1,0,0,0]` remains
input-only diagnostic data. The exact row-2 producer or copy into
`state+0x141d4` is still the shortest open edge.

## Scope

Ghidra 12.2 static only. No DEE run, media/PCM/payload/key read, or
implementation contact occurred. No binary or decompilation is included.
