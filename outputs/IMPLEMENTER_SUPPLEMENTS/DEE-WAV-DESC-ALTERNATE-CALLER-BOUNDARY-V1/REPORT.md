# Alternate AdmFileStorageDesc constructor boundary

## Bound static behavior

The upstream Ghidra receipt follows the alternate direct caller FUN_180406cd0 into the AdmFileStorageDesc constructor. It allocates a separate object, invokes the constructor with param_2, then writes a dynamic param_3 value to that new object's +0x38 field. The constructor's inherited initialization sets Desc+0x68 to zero; this caller has no post-construction write to +0x68.

## Remaining boundary

The value written to +0x38 and its producer remain unknown. Although the same static class is constructed, this allocation has no typed pointer/owner/generation join to the descriptor consumed by the selector-3 lower-storage path. This is not evidence about that selected instance or a global absence of other +0x68 writers. The task-supplied V2 parent manifest was not available in this checkout; it remains a supplied-hash reference only.

The upstream seal and normal/python -O checker passed. No DEE run, runtime capture, decode, media/PCM/payload/key/license access, or implementation contact occurred. Raw logs and code are omitted.
