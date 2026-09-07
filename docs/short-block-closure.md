# Short-block closure

ETSI TS 102 366 Table E.1.3 defines `numblkscod` values for 1, 2, 3, and 6
audio blocks per syncframe. A General E-AC-3 JOC programme interval must
accumulate six blocks, or 1,536 samples, across the selected I0 and dependent
programme sets. Short independent frames use `convsync=1` at the start of the
group and `convsync=0` for subsequent frames.

JOCForge now exposes this public contract as a pure
`resolve_short_block_plan` API. It accepts any sequence of public 1/2/3/6
block partitions whose total is six and returns the corresponding block codes,
sample count, and `convsync` sequence. It also exposes the public sequential
D0..D7 dependent-ID bound through `resolve_dependent_topology`.

The advanced `conformance-short` command now emits synthetic five/seven-channel
short frames with bounded zero-energy audio fields. The four canonical
partitions (`1+1+1+1+1+1`, `3+3`, `2+2+2`, and `1+2+3`) pass pinned OpenJOC
inspect/render and FFmpeg decode for idx0..idx4. These are conformance fixtures,
not the normal PCM-derived bitrate path; source-derived short-frame encoding and
CMAF packaging remain separate work.
