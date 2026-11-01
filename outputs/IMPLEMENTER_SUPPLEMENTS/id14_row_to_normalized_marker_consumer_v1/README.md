# ID14 row to normalized marker-consumer scope

A single hash-bound run observed one selected six-byte row through the DDP packet, copied row buffer, normalized U16 array, and marker consumer. For row index 10, the observed source and copied members were byte +3 = 0xBA and U16 +4 = 0x0000; the normalized consumer word was 0x800A and its bit-15 test was taken. The pointer chain and stride/index relations are bound by the run record.

This does not name the raw row members as ADM or OAMD fields, and does not claim U16 +4 alone caused the marker. The U6 writer instruction did not execute; no bitstream was emitted or parsed. The row-to-model-EMDF generation join remains open.
