# Separate D8 record writer, queue consumer, and getter boundary

## Newly washed static lineage

On the hash-identified Atmos path, a worker owned by a reader object runs a producer that allocates and initializes a separate 0x128-byte record. The producer copies a 16-byte provider pair into record-relative bytes +0xD8 through +0xE7 and transfers the record pointer into the worker's ring/queue.

A consumer method on that same reader object uses the same worker-state pointer, pops the queued record pointer, and passes that same record to a serializer. The serializer returns a view beginning at record-relative +0x98; record +0xD8 lies at +0x40 inside that view. The serializer itself does not dereference +0xD8.

This closes a static producer -> same-worker queue -> consumer -> serializer-view path for this separate record family.

## Getter and F06OF boundary remains open

The selected PcmMd getter reads +0xD8/+0xE0 from its own receiver, but the inspected graph does not pass the popped record pointer or the serializer view to that getter receiver. No shared owner pointer or generation token connects the paths. Equal member offsets and overlapping byte ranges are not identity evidence.

The selected PcmMd builder still has event count zero and a null event pointer; this is limited to that builder. Other producer/mode paths are not excluded. F06OF target slot and generation remain UNBOUND. Native capture and DEE input processing remain NOT_RUN / NOT_CONFIRMED.

No codec algorithm, production readiness, or global reachability claim follows. No PCM, EC3 payload, private key, or raw disassembly is included.
