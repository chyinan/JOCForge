# E-AC-3 JOC rate writer destination owner boundary

A generic U32 writer stores to child field +0x4 through an untyped destination pointer loaded from the writer receiver field +0x38. The scalar comes from a separate object pointer at receiver +0x2a8, then field +0x10. The bounded trace does not find a constructor/initializer for the child or a typed read of child +0x4.

The immediate post-return helper yields writer receiver +0x86b0, passed as a second argument to a callee that consumes only its first root argument from +0x182e0. No pointer edge joins that subaddress or root context to the +0x38 destination. Destination owner/type, first reader, and codec/BSI identity remain UNBOUND_STATIC. No broader xref or runtime claim is made.
