# Reader descriptor count owner boundary

The Reader wrapper is constructed from a CompositeMaster input and stores an inner Reader state. The getter reads a descriptor pointer from the inner state at +0x58, then reads the descriptor's U16 count at +0x0A into Reader N.

The inspected inner-state initializer copies 64-byte source-vector records. It does not expose a U16 store to this Reader descriptor's +0x0A or a typed transfer from that CompositeMaster vector into the descriptor count. The exact descriptor count writer remains UNBOUND.

A separate K count receipt uses an individual-plus-group U32 accumulator narrowed to U16 at K +0x0A; its generation is UNBOUND and it is not joined to this Reader descriptor. A matching +0x0A write on another metadata record has a different owner and is excluded. Physical CHNA N to Reader descriptor count and same-owner/generation linkage remain UNBOUND. No global absence claim or runtime support range is established.

Ghidra 12.2 static boundary only. No DEE run, media/PCM/payload/key/license read, or process control occurred.
