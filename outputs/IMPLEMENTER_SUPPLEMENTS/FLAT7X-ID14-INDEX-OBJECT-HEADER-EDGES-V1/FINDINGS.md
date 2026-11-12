# Flat7x ID14 header producer and field edges

## Result

Wash bound three DEE 6.7 static edges relevant to the official demo's ID14 structure:

1. ID14 reads the current invocation's U32 downmix value from state +0x24c into a local header context; the header writer emits that local as U3. For downmix index 1, the adjacent native channel-width consumer selects width 7 and passes it to a coefficient-channel routine. The +0x24c next-value update occurs after the current writer, so the current target invocation/generation value remains UNBOUND.
2. The code13/code14 path derives semantic object_count as code13 minus code14. For selector 1..5 its base is 15; the ID14 header writes object_count minus one as U6. Under the official target value object_count 15, the encoded U6 value is 14. Same-run selection/generation remains UNBOUND.
3. The channel-width 7, DDP track channel count 8, E-AC-3 chanmap locations, ADM physical rows, and JOC object count 15 are separate domains.

## Transport and parser boundary

The referenced official structure records ID14 in one D0 block-2 EMDF container with payload IDs 11 and 14. The native planner selects first block 2 for positive body-byte requests 1..2040. The official container range C=301..377 with C=B+4 gives request bodies B=297..373, a length-compatible range only. The reviewed selected ID14 consumer routes through generic U8 queue copy; this slice does not bind a native semantic parser or a same-generation runtime join.

## Scope

This supplement supports checking existing header/profile structure against direct native field edges. It provides no audio algorithm, PCM, frame bytes, semantic JOC decode, or runtime sample proof. complexity16 and current invocation values remain UNBOUND/NOT_RUN; production_allowed=false.
