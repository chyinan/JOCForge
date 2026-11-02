# Atmos core input row capacity for the 38-track ADM

The traced ADM BWF reader configuration uses a U16 row count and a pointer to six-byte channel-map rows. A reader-to-core setup path zero-extends the count, passes the map pointer and count to the core row setter, and copies rows in increasing index order into a core-owned `6 * count` buffer. The first per-frame processing boundary checks the frame capacity and requires the descriptor count to equal the processor's expected count.

No fixed 36-channel cap, sort, or truncation appears in this bounded path. If the supplied count is 38, the allocation is 228 bytes and the copy loop can express all 38 rows. The seven-sample metadata receipts show `人是猫` has 38 CHNA tracks: ten bed tracks plus 28 Object tracks.

This closes capacity/representation for a supplied count of 38, not the actual runtime value for the target job. Its reader U16 and runtime row map remain `UNBOUND`; metadata track count is not a runtime receipt. For a best-effort implementation, preserve the supplied count and ordered rows rather than imposing 36. Validate the target run separately; do not claim the DEE runtime U16 was captured.

Ghidra 12.2 static only. No WAV/BWF, PCM, payload, key/license, or DEE run was used. This finding can rule out a fixed 36 cap in this bounded reader-to-core slice, but does not by itself explain the measured PCM residual.
