# DRC row +0x10 next-use boundary

The selected-row converter first loads a QWORD member from row +0x10. Its next direct use compares that value with the explicit zero third argument. This establishes a zero comparison as the first use, but not the member's semantic type or role.

No typed projection connects the QWORD to snapshot +0xA0. The trace stops at the compare and does not inspect branch successors or later instructions. DRC, codec, BSI, and audio effect remain UNBOUND; no later consumer or global absence is claimed.
