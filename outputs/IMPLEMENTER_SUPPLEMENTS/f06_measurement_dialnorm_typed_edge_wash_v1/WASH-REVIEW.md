# Wash review: F06 measurement to DialNorm boundary

This supplement confirms two separate configuration entries: `MEASURE_ONLY` and `custom_dialnorm`. The R138 XML contains `measure_only` and omits the explicit override. That does not identify the override's runtime handler or the default source.

The only closed DDP-side edge is a same-owner U16 at `+0x34` copied into serializer member `+0x20`, classified as E-AC-3 DialNorm by the earlier washed writer-field package. This does not connect F06's finalized measurement to `+0x34`.

The native `MeasuredHistory` type is still an incomplete one-byte declaration without recovered members. The model's `selected_lufs_u32` and logged loudness scalar have no typed native finalize field join. No actual DialNorm value/default is provided. Do not use FFmpeg/OpenJOC or inference to fill it; keep the audio-affecting value unbound. If an authorized run reaches this setter, the smallest next observation is the typed finalize member plus the same owner's `+0x34` U16 under one job/generation token.
