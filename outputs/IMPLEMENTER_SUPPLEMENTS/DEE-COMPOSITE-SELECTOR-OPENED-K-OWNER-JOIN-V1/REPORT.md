# Selector descriptor to opened-K owner

## Bound static field lineage

The Ghidra 12.2 package binds a 64-byte per-item source record to owner J. The master-open dispatch receives J and the address of J+0x08 as its K output slot. Existing K consumers read the opened-source descriptor through that same slot: U16 count at K+0x0A, row pointer at K+0x10, six-byte rows.

The reader stores each per-item J pointer in its source-owner vector. Its later 0x58-byte selector descriptor projection copies J+0x08 into descriptor row +0x08, and the selector contract reads its K pointer from that row field. This closes the static field/pointer lineage from the selector descriptor row to the K slot owned by the same per-item J.

## Open runtime boundary

The static trace does not capture successful open status, the concrete K pointer, selected extension handler, or a runtime source-generation token. It does not establish that physical ADM metadata count N populates K+0x0A for a particular run. Those values remain NOT_CAPTURED or UNBOUND. No track-support or sample-branch claim follows from this package alone.

