# Individual-row slot to K capacity boundary

The selected individual-row writer obtains a physical destination slot from a U32 record field. It computes a six-byte row offset and stores the K-row tag, identity, and enumeration ordinal. The selected K count is stored as U16; the allocation request is K count multiplied by six bytes, and the code checks only whether the returned pointer is non-null.

Between loading the slot and writing the row, the selected sequence has no explicit slot-to-K-count comparison or equivalent local guard. The non-null allocation check is not a capacity check. The preceding predicate and accessor internals, upstream validation of the slot field, actual allocation extent, slot<K invariant, and N-to-K typed join remain UNBOUND. This is a bounded static writer result, not an observed out-of-bounds write or a global absence claim.

Ghidra 12.2 bounded pseudo-disassembly only; no DEE run, PCM, payload, key, or license access occurred.
