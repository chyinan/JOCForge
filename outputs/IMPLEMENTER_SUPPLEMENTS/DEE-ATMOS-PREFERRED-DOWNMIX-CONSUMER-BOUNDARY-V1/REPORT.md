# DEE Atmos preferred-downmix parser/consumer boundary

Ghidra 12.2 static evidence shows the selected parser reads `lorocmixlev`, `lorosurmixlev`, `dmixmod`, `ltrtcmixlev`, and `ltrtsurmixlev` as separate fields through conversion helpers. A conditional branch hands a generic name/value/error record list to a result-tree builder; the targeted builder body does not read the downmix fields directly and makes no final F13C frame-writer call.

The field-level route from `preferred_downmix_mode` to the internal `dmixmod` enum remains unbound. When the field is omitted, the enum helper skips value parsing; the UI's inactive display string `Stereo` is not evidence of a native default. The parser slice therefore does not show that Lo/Ro or Lt/Rt changes the encoded core audio, nor that the values are ignored downstream.

This is a control-path boundary only. It does not explain the measured AU-aligned 5.1 core PCM residual and is not a reason to delay the selected window-table trial. Keep the preferred-downmix consumer/writer join open until an actual F13C consumer is traced or a bounded playback/PCM observation is available.

No encoder run, PCM, media, payload, key, or license material was read. The Ghidra program identity is project-recorded and was not rehashed from the current installation in this slice; no code/decompilation is included.
