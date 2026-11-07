# Reader descriptor pointer null-initialization boundary

The Reader wrapper stores an inner Reader state whose getter later reads a descriptor pointer from inner-state +0x58 and takes Reader N from descriptor U16 +0x0A.

The selected inner-state constructor initializes +0x58 to null. Its CompositeMaster input is not copied into that pointer slot, and the inspected 64-byte source-vector initialization does not write the descriptor count. A separate same-offset metadata write and the separate K-count formula are not joined to this Reader descriptor.

The first unresolved edge is a later nonzero writer to inner-state +0x58. Physical CHNA N to this Reader descriptor and same-owner/generation linkage to K remain UNBOUND. No end-to-end channel range is established. Runtime was not captured; no DEE run or media/PCM/payload/key/license access occurred.
