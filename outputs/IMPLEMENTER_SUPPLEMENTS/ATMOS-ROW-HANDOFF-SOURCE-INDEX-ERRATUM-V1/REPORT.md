# Atmos row-handoff source-index erratum

This addendum corrects one second-level digest transcription in the source
index associated with `atmos_joc_multichannel_row_handoff_wash_v1`.

Wash independently recomputed all 24 referenced file digests: 23 match. The
source index's digest for
`outputs/adm_bwf_ddp_source_row_table_origin_wash_v1/TRACE.json` differs by one
hex character from the on-disk digest. The on-disk digest matches the
`TRACE.json` entry in that source package's own seal, and the source package
checker passes both normally and under `python -O`.

This is a provenance-index correction only. It does not change the accepted
planar row handoff candidate, the descriptor field offsets, copy width, row
order, renderer dispatch, or any audio-equivalence claim. It does not block
the input-handoff MVP. The incoming Atmos package and the existing
`ATMOS-JOC-ROW-TO-RENDERER-HANDOFF-V1` supplement remain unchanged.

No source binary, decompilation, audio, PCM, encoded payload, key, or license
material is included.
