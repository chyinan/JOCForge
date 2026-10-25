# F13C writer physical-extent status

The selected writer has a cursor/base state and writes 16-bit units. It also tracks a logical bit offset and records expected/final bit counts in a separate context. These facts define a logical write budget; they do **not** establish the writable extent of the underlying allocation.

The pointer field from the prior native writer trace has not been identity-joined to the Ghidra writer's cursor/base argument, its owner, or the allocator that created that buffer. No physical capacity or maximum writable byte extent is therefore known. `physical_buffer_capacity` remains `UNBOUND`; no buffer contents were read.

This package does not authorize RAM serialization or a native run. Closing the edge requires a hash-bound field/owner mapping from the existing pointer expression to the selected writer argument, then a constructor/allocator result and typed physical capacity. Until that identity is established, do not substitute logical bits-written counters, expected/final bit-count records, or equal values for allocation capacity.
