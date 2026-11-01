# Wash review: sample-6 runtime row-order observation

R3's second full-file DEE run captured the first DDP row-copy call for 38 rows of 1,536 frames in one process/thread. The input and internal destination tables contain 38 unique row pointers each with no pointer intersection or same-ordinal pointer equality. Hash-bound static DDP code copies `frame_count * 4` bytes from input row `i` to internal row `i`, preserving ordinal at that copy boundary.

R3's converter CHNA lookup and reader-adapter breakpoints were armed but not hit before the first DDP copy or target exit. Therefore this runtime event does not bind ADM identity/CHNA track number to reader output row ordinal or DDP input row ordinal. The source rows at DDP entry remain unknown despite count 38. No row values or PCM samples were read or persisted.

The R3 output metadata reports 4,247 E-AC-3 AUs with JOC signaling in all AUs. Its output hash is R3's receipt claim captured before removal; Wash did not rehash the media payload. Another DEE run (R4 run-142) has a different output hash, so these are not a bit-exact pair. Both are posthoc structure references only; neither defines JOCForge's audio mapping.
