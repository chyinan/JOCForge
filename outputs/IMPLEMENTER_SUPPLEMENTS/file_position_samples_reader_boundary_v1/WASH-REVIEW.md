# Wash review: file-position samples property and reader extent

The hash-bound DEE converter consumes a literal `samples` metadata property as a nonnegative source-bound value. The selected converter image has one literal reference to that key and no observed local writer. A separate reader constructor/open path has an extent getter backed by a U32 metadata-tuple field converted to binary64, but the field's semantic type and owner join to the `samples` map are unbound.

For the authorized sample-6 BWF, the header gives `data_chunk_bytes=743,611,284` and `block_align=114`, which divide to 6,522,906 frames with zero remainder. This independently checks the user's sample-count claim; numeric equality does not prove DEE derives its property this way. Static analysis did not observe a direct division by block_align in the selected converter.

For JOCForge, a direct header-derived extent is an I/O-format parsing choice; this package does not claim that it is DEE's hidden metadata producer or a codec algorithm. The open DEE edge is one same-generation join from the source metadata property to the opened reader extent and terminal request/EOF.
