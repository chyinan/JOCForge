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

This does not yet emit short compressed E-AC-3 frames. The available FFmpeg
E-AC-3 encoder exposes no short-block frame-duration option, and JOCForge has
not introduced an independent five/seven-channel short-frame serializer. The
capability matrix therefore keeps short-block cells
`SPEC_LEGAL_NOT_IMPLEMENTED`; CMAF cells remain container-specific and do not
inherit General E-AC-3 short-block legality.
