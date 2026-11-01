# Force-D45 history: cold start and copy path

This package washes the Ghidra 12.2 static trace of the per-substream owner history feeding the force-D45 override.

The owner is carved from a calloc-backed encoder arena. FUN_1802e51b0 explicitly clears five lane records at owner+0x135c0; each record is four U16 words with an 8-byte stride. FUN_180316730 reads the first U16 of each lane record and copies it unchanged into owner+0x15208 at block-major offset 2*(block*5+lane). FUN_180311900 advances the low 16-bit block ordinal from zero and performs these copies before the later FUN_1802fc200 strategy calculation.

The downstream FUN_1802fd710 check treats the copied U16 as zero/nonzero: a nonzero cell forces D45/3 when the computed strategy is not already 3. No unit or scale is established. The first generation is statically zero; no later direct writer was found in this image, but indirect or cross-image mutation was not excluded. Runtime nonzero values and generation identity were NOT_RUN.

This supports modeling the cold zero and copy semantics only where the selected model has the same owner lifetime. It does not justify permanently fixing the history field to zero. The independent selector no-row 0xC return remains a separate open edge. No DEE run, media, PCM, payload, bitstream, key or decompilation is included.
