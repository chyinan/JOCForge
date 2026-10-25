# F13C selected writer owner and physical extent

## Static owner chain now established

The selected Ghidra writer uses an embedded writer state containing an output cursor/base, logical bit offset, and accumulation state. Its direct selected caller passes that writer state from a containing owner. A separately analyzed initializer copies a backing pointer from an indexed owner table slot into the embedded writer state.

This closes only the reviewed Ghidra writer-state owner/initializer chain. The indexed table producer and allocator extent are not present in the bounded call graph. The previously reported IDA pointer expression has not been identity-joined to this Ghidra field; do not treat the two as the same pointer or owner.

## Physical-capacity boundary

The writer emits 16-bit units through the cursor/base state and tracks a logical bit count separately. The clean/native evidence reviewed here does not provide the physical writable byte extent behind the copied backing pointer, nor a capacity check against that extent. Logical expected/final bit-count records are not allocation-capacity evidence.

Status:

- `Ghidra writer state → containing owner/initializer = STATIC_CLOSED_FOR_REVIEWED_CHAIN`
- `upstream pointer-table producer/allocator = UNBOUND`
- `allocated physical writable extent = UNBOUND`
- `IDA pointer expression ↔ Ghidra writer state identity = UNBOUND`
- `RAM/native capture authorization = NONE`

No native target was run and no buffer contents were read. This is a status update for the native research path; it does not define the product encoder's local buffer API.
