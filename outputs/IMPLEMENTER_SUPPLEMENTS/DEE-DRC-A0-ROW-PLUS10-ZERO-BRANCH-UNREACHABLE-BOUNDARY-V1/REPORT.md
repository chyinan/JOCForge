# DRC row +0x10 compare branch boundary

The selected-row converter's first observed row access loads a QWORD from row +0x10. Its next direct use compares that value with the call's explicit zero third argument. The immediately following branch is an unsigned less-than-zero test, which is false for this bound call edge.

The bounded trace stops at the taken target callsite. It does not inspect the fallthrough path, the target helper, or later converter instructions. The QWORD's semantic type/owner and any typed projection to snapshot +0xA0 remain UNBOUND. DRC, codec, BSI, and audio effect are not inferred; no global absence is claimed.
