# R138 generic setter receiver discriminant

## Bounded update

In the host `OutputWrapper::SetProperty` `dee_audio_source_type` path, the argument object is constructed as `dee::Types::StringType`. The generic virtual setter is invoked on a separate receiver pointer (`plVar3`) through slot offset `+0x28`; the argument's vtable does not choose the target. The selected R138 process-state schema independently identifies `+0xA0` as `dee::Types::MediaMetadata`, whose type-specific `set` method is vtable slot 5 at offset `+0x28`.

The most likely target remains `MediaMetadata::set`, based on the matching property/type schema and slot. The receiver vtable was not resolved, so keep the actual generic vcall target and `plVar3 == selected +0xA0` identity `UNBOUND`. Matching slot offsets alone do not prove the call target. This narrows the candidate path without blocking the row-preserving MVP rule from the parent supplement.

## Recommended model rule

Preserve input source-vector order; emit one-based row index and copy filename/string fields; keep AO/ATU typed converter identities separate. Do not reinterpret the `StringType` argument as the setter receiver or infer owner/generation from equal values.

## Exclusions

Ghidra 12.2 static evidence only. No DEE run, media/PCM/key access, or implementation contact occurred. No source code, binary, or decompilation is included.
