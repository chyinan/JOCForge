# R138 OutputWrapper registry initialization boundary

The `dee::OutputWrapper` constructor initializes source/property registry
member `+0x178` to an empty sentinel container. The scoped scan of its vtable
methods found readers but no store or write-through to that member. The
`dee_audio_source_type` SetProperty path reads the registry and builds the
filtered `+0x1A0` range; it does not populate the source registry.

Once populated, preserve the registry's insertion/range order; the downstream
`+0x1A0` result is the matching ordered subset. The first post-construction
source-registration writer is still unbound. This slice does not identify the
registry with job-file order, an upstream stage container, or ADM converter
channel order. The next minimal closure is the first typed source-registration
call from the direct construction owner.

This is metadata ordering, not an audio-transform result. No binary,
decompilation, media, PCM, payload, key, or license material is included.
