# ADM/BWF opener channel-table producer

For each ADM object or bed channel, the converter resolves its UID through CHNA to one-based TrackIndex. TrackIndex minus one selects the physical row in the opener-owned six-byte table.

The record payload carries a different identity. For object rows, U16 byte 0 carries an AudioObjectID-derived source-channel key; byte 2 is the object sentinel and U16 byte 4 is the object ordinal. For bed rows, U16 byte 0 carries a bed source-channel key, byte 2 is the bed label/type and U16 byte 4 is the bed ordinal. Thus the first U16 is not CHNA TrackIndex.

The setup callback receives a separate param4[2] pointer. Static analysis has not bound it to the opener table in the same parent job/generation. The remaining minimum observation compares the opener table pointer/count and setup input pointer/count in the existing R3 job.

Source package manifest SHA-256: ba389271c6ee81599ba2b2136e5eb5ae48e445ee7cf54984cc18fd0c77b961a8. Its normal and optimized checks passed. This summary contains no disassembly, source code, binary, PCM, private key or absolute path.
