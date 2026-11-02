# R138 OutputWrapper post-filter source-row order

## Best-supported host-local rule

Within the bounded `dee_audio_source_type` branch of `OutputWrapper::SetProperty`, the wrapper source range is read as a begin/end view and advanced entry by entry. Nested property ranges also advance by iterator. The scoped range-view and append helpers preserve the order they receive; no explicit sort or unique operation was found in this path.

A preceding match/used-marking pass can select or omit entries. Therefore preserve the **post-filter OutputWrapper source-range order**. For each selected source record, copy the source label/filename and nested string fields from that same record's `+0x20` subobject before advancing. The target `MediaMetadata` writer's `index = ordinal + 1` rule applies to the vector it receives, but the vector-to-wrapper-row identity remains `UNBOUND`.

Do not equate post-filter wrapper order with ADM CHNA order or the converter's `ConverterChannelInputId` vector. Keep those as separate domains; the AO/ATU crosswalk applies only within the converter's typed same-ordinal check.

## Scope and confidence

Ghidra 12.2 static trace of the host source-range and row-assembly helpers. This closes the local row association and preserves received order after filtering. The upstream order producer, converter-source-vector join, selected R138 `+0xA0` object join, and generation remain unobserved. No DEE run, media/PCM/key access, or output effect was measured.

## Exclusions

No source code, binary, decompilation, media, PCM, key, or license material is included.
