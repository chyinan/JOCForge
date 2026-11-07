# WAV lower-storage position and range initialization

## Bound static behavior

The selector-3 lower object is atmos_storage::AdmFileStorage. Its vtable slot +0x60 returns the current-position member at +0x10; slot +0x68 returns the limit member at +0x18. The constructor initializes current to zero and initializes limit from the source descriptor's +0x18 getter. That descriptor getter returns Desc+0x68, whose traced base-constructor initialization is zero. The inspected factory/open path contains no later explicit write to that field before lower storage construction.

Thus the examined construction path starts with current=0 and limit=0, an empty initial range. The later writer of a nonzero descriptor limit is still UNBOUND.

## Units and limits

The per-channel F32 sample-index/count unit is inherited as a static call-contract inference from a previously sealed planar reader wrapper that multiplies offset/count by four bytes. This package did not re-analyze the +0xd8 reader body; the inferred unit and any later cap behavior are not runtime-observed. The supplied parent V2 manifest was not present in this checkout and is retained only as a supplied hash reference.

Runtime values and provider selection are NOT_CAPTURED. No DEE job, PCM/payload read, key/license access, or output was produced. Raw Ghidra traces and attempt logs are excluded from this supplement.
