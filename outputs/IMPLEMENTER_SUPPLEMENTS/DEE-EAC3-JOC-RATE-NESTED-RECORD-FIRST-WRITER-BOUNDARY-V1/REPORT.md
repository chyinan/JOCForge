# E-AC-3 JOC rate-conditioned nested record writer

The nested pointer comes from writer receiver +0x2a8. When it is non-null, a rate-conditioned table row copier writes U32 values to nested fields +0x4, +0x8, and +0x10. Row selection depends on the rate value and writer-state control fields +0x0c and +0x1c.

The nested object's constructor, type token, and semantic owner are not bound. A later generic U32 store reads nested +0x10 and writes another child field +0x4, whose owner is also unbound. No codec/BSI identity or encoded audio effect follows from this path.
