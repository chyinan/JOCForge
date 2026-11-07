# Line DRC row-member +0x10 boundary

Reference-only Wash supplement; it records the first next-use of a selected-row member, not its semantic identity or a DRC effect. `IMPLEMENTATION_READY=NO`.

In the selected row helper `FUN_1801431f0`, a QWORD is loaded from selected-row `RDX+0x10` at `0x180143205`. Its next direct use is `CMP RAX,R8` at `0x180143215`; the caller supplies zero in R8. This proves a zero comparison for that row member at this site.

The bounded slice does not establish the row member's semantic type/owner or a typed projection to the snapshot `+0xA0` U32. It stops at the comparison: no branch successor, codec/BSI use, or audio effect is inferred.