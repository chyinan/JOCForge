# F13C callback input/output pair identity boundary

## Result

Ghidra evidence ties the candidate indirect slot call at `0x1403c1bb9` to an input pair copied into the local `uStack_48` and passed as `param_3`. The callback-output fields written through `FUN_1404813f0` belong to a separate output pair built in `auStack_830` and transferred into `plStack_a78`. The input-pair slot call is not evidence of a consumer of the new output fields.

## Bound paths

At `0x1403c2eee`, `FUN_1403c2e50` calls `FUN_1403c0ad0`; `param_3` is the address of `uStack_48`. The pair copied there originates from `FUN_1403c2e50`'s `param_4`, via `FUN_1403a6960` at `0x1403c2ed1`.

Separately, `FUN_1403c2d80` builds a new callback object in `auStack_830`. `FUN_1404813f0` writes offsets `+0xf0/+0xf4/+0xf8` through the callback object's `+0x70` subobject. `FUN_140450eb0` transfers the resulting pair into `plStack_a78`. The source package establishes no shared object pointer between this output pair and the input `param_3` pair.

## Open edge

The first direct consumer of the output fields, preferred/downmix semantics, DDP-JOC/EC3 writer, and final F13C writer remain `UNBOUND`. The next minimal query is to follow `plStack_a78` after the pair transfer, establish the exact receiver pointer, then check whether it reads the output callback object's `+0xf0/+0xf4/+0xf8` fields.

The Ghidra project records image SHA-256 `4b4c624d94a5b47d97b62b28073e1bd351b2deeeea039a7ccfaa2af96562c8a7`; the installed executable was not rehashed. No encoder, PCM, payload, key, or audio output was accessed.
