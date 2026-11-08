# Dialogue Intelligence measurement boundary

A washed static trace places a candidate Bool at measurement setup +0x94, conditional on the host query/record path selecting the registered descriptor. The first scoped per-record consumer, FUN_1801764e0, reads setup +0x9c and +0x140; this trace did not capture a read of +0x94. The aggregate F06 path uses per-record F32 at record+0x14; no typed Bool-to-result edge is bound.

Two prior measurement-only A/B receipts show sample-dependent diagnostic changes under DI false/true. The measurement unit is not stated in the native log. They are prior observations, not new runs, and do not identify the Bool algorithm consumer or establish an audio/JOC effect.

Keep query selection, selected owner/generation, first typed algorithm use, and Bool-to-result behavior UNBOUND. No new DEE run or runtime capture is claimed.
