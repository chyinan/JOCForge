# E-AC-3 JOC data-rate writer return boundary

A generic U32 write reaches a destination object's byte field +0x4 through the writer receiver field +0x38. The immediate helper returns a different subaddress, writer receiver +0x86b0, which the caller passes as its second argument to the next callee. That callee consumes only its first root-object argument, loaded from a separate field at +0x182e0; it does not consume the second argument. The bounded continuation therefore does not read the writer destination's +0x4 field.

The pointer return and argument transfer are bound statically. The destination object's type, first reader, and codec/BSI identity remain UNBOUND. No equality of offsets or values is used to join these objects. The package covers one direct continuation only and makes no global absence claim.
