# DRC row-helper converter boundary

The selected helper initializes a string-like output object and does not dereference members of its selected-row pointer. Its first use of that pointer is an opaque pass to a nested converter. The helper then copies four other local values into generic output slots and returns.

The snapshot +0xA0 value is not joined to the row pointer or copied locals by a typed owner edge. The helper body does not establish whether the nested converter reads any field corresponding to +0xA0. The first break is the helper-to-converter argument boundary. The DRC value, codec/BSI transfer, and audio effect remain UNBOUND. The converter body, other callers, global references, and +0xA4 were not inspected.
