# R138 StageRunner InitOutput property boundary

The successful Ghidra retry binds two direct `InitOutput` calls: the audio
input property is read from StageRunner `+0x428`, and the video input property
from `+0x5E8`; both pass through `StageRunner::SetProperty` to
`OutputWrapper::SetProperty` as typed key/value records. These calls are not
`dee_audio_source_type` and do not write `OutputWrapper+0x178`.

This rules out these two InitOutput property callsites as the first source-
registry writer. The first typed `dee_audio_source_type` producer and its
container-to-`+0x178` edge remain the exact open point. Preserve the registry's
own insertion order; do not substitute these StageRunner field offsets or the
separate `.stream_object` queue for that order.

The earlier Ghidra startup warning was nonfatal; the later failed lookup was
only a wrong log filename/wildcard, and the narrow two-callsite Ghidra retry
completed successfully. This package is static metadata-path evidence, not an
audio-transform finding. No DEE run, media, PCM, key, or payload was accessed.
