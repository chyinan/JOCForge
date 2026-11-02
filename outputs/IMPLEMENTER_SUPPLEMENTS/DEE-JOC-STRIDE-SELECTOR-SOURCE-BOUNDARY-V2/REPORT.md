# DEE JOC per-block stride-selector source boundary addendum

## Result

The block-local selector consumer is statically bound in `dee_audio_filter_ddp_atmos.dll` SHA-256 `3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2`: `FUN_1800a3710` reads `state+0x140e8+block*10+0xd8` and maps the first four table values `[0,1,2,4]` to disabled, stride 1, stride 2, and stride 4.

A full-image Ghidra scan of direct 16-bit `+0xd8` stores found five candidates: two target stack slots and three target other object members. None joins to the selector row. The closest candidate, `FUN_1800a19e0`, is called from `FUN_1800729a0` with `RCX=state+0x140e8`, but the callee uses `RDI=RCX+0x48` and stores `R9W` at `[RDI+0xd8]`, i.e. `state+0x140e8+0x120`; `R9D` is cleared before the store. That is a different member, not the block selector. Another `RSI+0xd8` value flows to `RSI+0x192d4` and is also a different field/base.

This scan covered direct scalar stores; bulk-copy initialization was not covered. The best-supported source remains an upstream per-block/lane-plan value, possibly configuration-derived. It is consumed once per block; whether a prepass fills all blocks or updates it per frame remains unknown. The 1024 kbps / 5.1 label does not directly establish a selector value.

## Implementation rule and stopping point

Keep the selector as an explicit per-block input and apply `[0,1,2,4]`; do not derive it from bitrate or channel layout. A static field writer was not found in the targeted direct-store search, so use the current model's best-supported block/lane-plan input and continue the focused output trial. Further static work is only useful if it locates a bulk-copy/initializer that changes that selector; do not expand into unrelated `+0xD8` fields.

This is a JOC payload rule. It does not explain the E-AC-3 core exponent parser error. No DEE run, media/PCM, or key/license access occurred.
