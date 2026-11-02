# JOC per-block U7 stride-selector profile boundary

## Bound consumer and selector mapping

In the Atmos JOC image, `FUN_1800a3710` reads a 16-bit per-block selector at
`state+0x140e8+block_index*0x0a+0xd8`. Its lookup is `[0,1,2,4]`: selector 0
disables the branch; 1, 2, and 3 select strides 1, 2, and 4. Static callsites
bind this table base to both the per-block consumer `FUN_1800942d0` and the
six-block loop in `FUN_180098080`.

The constructor `FUN_1800a19e0` clears six consecutive shorts, establishing
rows 0 and 1 as zero defaults. Rows 2 through 5 are not initialized by that
clear and remain `UNBOUND`. A separate profile bulk-copy path copies profile
tables into a descriptor-selected slab and installs component pointers at
`+0x1e0`, `+0x2d0`, and `+0x508`; that path does not write the inline selector
rows.

## Selected-profile limit

The actual selector values for rows 2 through 5 in the `1024 kbps / 5.1 /
E-AC-3 JOC` profile remain unknown. Do not infer them from bitrate, channel
layout, `preferred_downmix_mode`, the RenderBus ID-0 branch, or the E-AC-3
core selector table. The per-block selector is a separate JOC input.

For the implementation candidate, keep block selectors explicit. This slice
supports constructor defaults 0 only for rows 0 and 1; it does not supply a
selected-profile value for the remaining rows. The one-hot vector below is a
test input proposal, not a native observation.

## Scope and exclusions

Ghidra 12.2 static only. No DEE run, media/PCM/payload/key read, or
implementation contact occurred. The source package has hash manifests and
passed normal and `python -O` checkers; it did not provide a separate
`package_seal.json`. No binary or decompilation is included in this Wash
supplement.
