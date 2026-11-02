# R138 OutputWrapper source-range producer

The verified local producer for `OutputWrapper` member `+0x1A0` starts from an
empty range. In the `dee_audio_source_type` branch of `SetProperty`, the
builder walks the existing source/property registry at `+0x178`, appends
entries matching the supplied property value, and marks matched properties as
used. The append helper increments the destination count. Within this scoped
chain, the emitted range therefore preserves the registry traversal order
after filtering; no explicit sort or unique step was observed.

The practical rule for this local metadata range is to preserve that filtered
order. Do not identify it with job-file order, the ADM converter channel
vector, or the seven-file scan order: the `+0x178` producer and those joins
remain unbound. Duplicate disposition is also unbound. The earlier same-row
field-preservation result remains separate and unchanged.

This is an OutputWrapper metadata/order contract, not an audio transform.
Ghidra 12.2 read-only evidence only; no runtime vector, DEE run, media, PCM,
key, or license was used. The Wash package includes no target source,
decompilation, or binary.
