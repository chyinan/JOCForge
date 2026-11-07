# Line DRC row+0x10 zero-comparison boundary

Reference-only Wash supplement; it records one branch condition and stops at its callsite. It does not establish a DRC/audio effect. `IMPLEMENTATION_READY=NO`.

The selected-row `+0x10` QWORD is compared with R8 in `FUN_1801431f0`. The caller explicitly supplies R8=0. The immediate branch is unsigned `RAX < R8`, which is false for this call edge. Its taken block only loads a constant-like address into RCX and calls `FUN_180554ed8`; the callee and string content were not inspected.

The row field's semantic identity/owner and any typed join to snapshot `+0xA0` remain unbound. No other branch, successor, converter, codec/BSI input, or audio effect is inferred.