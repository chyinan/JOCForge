# R138 StageRunner stream-object queue separation

`InitOutput` reads `dee_audio_input_source` and numbered
`dee_audio_output_source:n` properties in increasing ordinal order, extracts
each `.stream_object`, and inserts the typed value into `StageRunner+0x1158`.
`RunAudioFilter` walks that queue and sends each property key/value to the
selected filter product's `+0x18` setter. The traced handler accepts only
`dee_misc_container` and `dee_logger`, storing them in process-state `+0xF20`
and `+0xF28`.

This stream-object path does not write `MediaMetadata+0xA0` or
`OutputWrapper+0x178`; no pointer/container join to either was found. Keep
StageRunner queue ordinal and OutputWrapper registry insertion order separate.
The remaining F06 edge is the first typed `dee_audio_source_type` producer
that populates `OutputWrapper+0x178`.

This is metadata/property routing, not an audio-transform rule. No DEE run,
media, PCM, payload, key, or license material was accessed; no raw source or
decompilation is included.
