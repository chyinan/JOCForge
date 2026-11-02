# Human Cat DRC per-block F32 source initialization

## Washed behavior

The matched DEE core constructs a DRC source context and initializes 30 consecutive 32-bit words to zero. The same addresses are later read as F32: six block rows, five F32 values per row, with a `0x14`-byte block stride. The first per-block values read are at offsets `+0x2170` and `+0x2174` within the source context.

Within the same static core call, the F32 producer writes a separate output row at `output_context + 0xbc + block*4`; the producer and later U16 code/presence consumer both use the same DRC output-context base. This joins the constructor, typed source reads, F32 output row, and consumer at a static call/state-base level.

## Boundaries

The zero initialization is not evidence that the values remain zero at runtime. A later generation-time writer may update the row through an alias, bulk copy, or another path; that writer has not been located. No matched-run F32 values, profile pointer contents, reference U16 values, or positive-store codes were captured. The `film_light` XML strings still do not map to a numeric selector, and mode 0/1 remain unlabeled as Line/RF.

The symbolic vector expresses the typed offsets and zero initializer only. It is not an execution or an expected audio result. No PCM, encoded payload, private key, license material, native source code, or disassembly is included.
