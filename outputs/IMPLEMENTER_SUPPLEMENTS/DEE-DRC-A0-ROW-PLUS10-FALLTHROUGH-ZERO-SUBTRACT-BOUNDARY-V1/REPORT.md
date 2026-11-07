# DRC row +0x10 fallthrough first-use boundary

After the selected row +0x10 QWORD is compared with the explicit zero argument, the fallthrough's first direct use subtracts that same zero-valued argument from the QWORD. The trace stops at this single instruction; it does not inspect the resulting value, later branches, or subsequent consumers.

The row member's semantic type/owner and its relation to snapshot +0xA0 remain UNBOUND. The observation establishes only this local arithmetic step. DRC, codec, BSI, and audio effects are not inferred; no global absence is claimed.
