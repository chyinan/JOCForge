# Multi-dependent raw JOC closure

This is an advanced raw-elementary conformance boundary, not a normal encoder
preset. ETSI TS 102 366 V1.4.1 permits up to eight sequential dependent
substreams per independent substream (`D0..D7`). ETSI TS 103 420 V1.2.1
requires all associated dependents to reach JOC and places the OAMD/JOC EMDF
in the last dependent.

JOCForge's canonical source-derived writer now supports validated no-LFE
Table-47 raw profiles through `I0+D0..D5` where the selected profile's final
channel ownership closes. It emits actual source-derived audio for every
dependent, rejects overlapping maps, and places the sole JOC/OAMD carrier on
the final dependent. The pinned OpenJOC validator then checks strict parse and
full programme render.

The public raw count maximum remains eight. `D0..D6` and `D0..D7` are recorded
as `PUBLIC_LEGAL_NOT_IMPLEMENTED` for the current canonical JOC plan rather
than guessed, because the 5/7-channel Table-47 input and unique no-placeholder
ownership constraints do not close those counts without a separate LFE or
replacement-chain design. CMAF remains independently restricted to one `I0`
and optional `D0` by TS 103 420 Annex E.

FFmpeg is not a normative legality gate. Its public decoder currently decodes
substream ID 0 and explicitly skips additional nonzero substream IDs. Thus
`I0+D0` may report FFmpeg `PASS`, while a structurally valid `I0+D0+D1+…`
cell reports `UNSUPPORTED_DECODER_FEATURE` after OpenJOC strict/full-render
success. See [the detailed interoperability boundary](ffmpeg-interoperability-boundary.md)
and the [public semantics matrix](multi-dependent-public-semantics.md).
