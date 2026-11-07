# DRC row-converter first-read boundary

The selected-row pointer reaches a nested converter as an opaque argument. The converter's first observed read through that pointer is one QWORD member at row +0x10. The function prototype and member semantics are unresolved at this boundary.

No typed owner or projection joins this QWORD to the snapshot +0xA0 U32. The trace stops at this first read; it does not infer that the QWORD is the DRC value and does not inspect later instructions or consumers. DRC/codec/BSI effect remains UNBOUND. No DEE run or media, PCM, payload, key, or license access occurred.
