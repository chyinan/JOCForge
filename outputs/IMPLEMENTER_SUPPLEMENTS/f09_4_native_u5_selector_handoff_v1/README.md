# F09.4 native U5 selector and stuffer: bounded static closure

This Wash supplement closes the hash-bound Ghidra static path from the six-block lane marker strategy through the native 32-row table selector and the five-bit convexpstr stuffer. The native U5 table projection SHA matches the approved Table E.1.9 projection. The input-only row [2,0,0,2,0,3] selects code 5 in both; this is a static table match, not a runtime sample.

The producer scans blocks 5 to 0 and increments per-lane distance before testing each marker. Marker zero maps to R/0; nonzero marker at distance 1 maps to D45/3, distance 2 or 3 to D25/2, and distance 4+ to D15/1, then distance resets. A separate parallel short may force D45; it is copied from owner+0x135c0 to owner+0x15208, but the first writer of +0x135c0 is not located. Marker threshold units and the complete threshold input producer also remain unbound.

The selector exact-matches all six U2 cells in code order. A no-row result returns 0xC without storing a code; the caller ignores that return and sets the ready flag. The all-input no-row invariant or error propagation is therefore not proven. The frame-set writer serializes a preexisting U5 code and does not perform selection. F10 segment grouping and the six-block frame-set counter are separate mechanisms.

Implementation may use the statically closed table lookup and stuffer semantics within the approved six-block model. It must keep the force-D45 source, marker-threshold producer, and no-row path open; do not substitute a guessed marker or invent a code for 0xC. No DEE job, PCM, bitstream, media, playback, or private key was used.
