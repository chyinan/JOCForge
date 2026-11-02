# R138 generic setter receiver: outer/inner vtable boundary

## Updated static result

The host receiver for `OutputWrapper::SetProperty` is statically `dee::OutputWrapper`: the method is in its vtable and its constructor installs that vtable. The host constructs the argument as `dee::Types::StringType`, then loads a separate nested receiver `plVar3` from an assembled property/source entry and dispatches a generic virtual setter through slot `+0x28`.

The selected R138 process-state property at `+0xA0` is `dee::Types::MediaMetadata`, and `MediaMetadata::set` is vtable slot 5 at the same `+0x28` offset. This remains the most likely target for the `dee_audio_source_type` path, but the bounded constructor trace found no `plVar3` vtable write. The argument vtable, outer `OutputWrapper` vtable, property name, and matching slot do not prove the nested vcall target or that `plVar3` equals selected process-state `+0xA0`.

## Implementation disposition

Keep the best-supported row-preserving metadata model rule available for an input-plumbing trial: source vector order, one-based output index, filename/string-field copying, and typed AO/ATU crosswalk. Do not use the generic setter boundary to assert a confirmed R138 call. The receiver vtable and selected object pointer remain `UNBOUND`; this gap does not block trying the row rule.

## Exclusions

Ghidra 12.2 static evidence only. No DEE run, media/PCM/key access, or implementation contact occurred. No source code, binary, or decompilation is included.
