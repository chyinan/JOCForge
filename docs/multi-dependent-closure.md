# Multi-dependent closure

This is an advanced conformance boundary, not a normal encoding preset.

ETSI [TS 102 366 V1.4.1](https://www.etsi.org/deliver/etsi_ts/102300_102399/102366/01.04.01_60/ts_102366v010401p.pdf)
permits dependent substream IDs and channel maps, and the general raw E-AC-3
assembly space reaches sequential D0 through D7. ETSI
[TS 103 420 V1.2.1](https://www.etsi.org/deliver/etsi_ts/103400_103499/103420/01.02.01_60/ts_103420v010201p.pdf)
requires the OAMD/JOC EMDF on the last dependent when dependents are
present. JOCForge's probe follows those rules: sequential IDs, explicit
replacement maps, and no JOC EMDF on earlier dependents.

The current clean probe produces a structurally admitted 5.X D0+D1 form. The
pinned OpenJOC inspector and renderer accept the access unit, but the host
FFmpeg decoder rejects the complete multi-dependent elementary stream. The
CLI therefore removes the output and returns an actionable failure instead of
advertising the form as supported. The capability matrix remains
`SPEC_LEGAL_NOT_IMPLEMENTED` for multi-dependent cells.

This is an external decoder boundary, not an unexamined JOCForge error:
current FFmpeg `libavcodec/eac3dec.c` explicitly decodes only substream ID 0,
skips additional substreams, and requests an additional-substream sample. See
[FFmpeg's E-AC-3 decoder source](https://github.com/FFmpeg/FFmpeg/blob/master/libavcodec/eac3dec.c)
and its [dependent-stream support history](https://ffmpeg.org/pipermail/ffmpeg-cvslog/2018-March/113134.html).

The remaining work is an independently validated E-AC-3 carrier path that
preserves both public dependent ownership and FFmpeg core decode. No vendor
bytes or proprietary behavior are used in this probe.
