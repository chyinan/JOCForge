# Serializer-to-consumer parent binding

This hash-bound supplement supersedes the temporary event-grouping retraction for the original runtime parent marker and lifts the body-consumer candidate quarantine for the exact function identity described here.

The matching module and original breakpoint record identify the marker as a real function entry. In one bounded runtime invocation, serializer and consumer both report parent generation 1 on the same thread. The serializer produces length 312 into a caller-local buffer; the same pointer reaches consumer body input, and the serializer length is passed as producer_length. The consumer marker is 22584. The consumer state has start_index 2 and six unsigned per-lane counts [0,0,312,0,0,0]; the static rule sums lanes 2 through 5, yielding 312 bytes, matching both producer_length and serializer output length.

The caller invokes serializer before consumer, reads the serializer length slot, passes the same buffer and length when nonzero, and an entry thunk adjusts the owner to the consumer state subobject. Correct names are marker and producer_length; old arg2_u32/arg3_u32 labels were record-field names, not ABI ordinals.

Scope is one invocation. Same access unit, source generation, and consumer-to-writer relation remain UNBOUND. No raw body bytes, module addresses/code, disassembly, input/config/output digests, or key/auth material are included.
