# D8 getter owner and initial-state boundary

## Getter owner

On the hash-identified Ghidra path, the selected AtmosProcessor outer object stores an inner state pointer at outer +0x8. The selected PcmMd getter reads inner +0xD8 and +0xE0 and copies them to the output wrapper fields +0x28 and +0x30.

The inner state is allocated and initialized by the selected constructor. Its initializer zeros ten qwords from inner +0xB0 through +0xF8, including +0xD8 and +0xE0. One selected process update writes +0xB0, +0xB8, +0xC0, +0xC8, +0xD0 and +0xE0, but not +0xD8. This bounds the initial state and one update only; it does not exclude later writers or other routes. No runtime value was captured.

## Separate queued record path

The previously washed D8 candidate writer/queue path is a separate record allocation owned by an AtmosReaderCC worker state. Its queue consumer pops the same record pointer and passes it to a serializer view. No inspected call transfers that queued record or its view to the selected AtmosProcessor inner state used by the PcmMd getter. The reader worker-state pointer and AtmosProcessor inner-state pointer remain distinct.

Consequently, the record-relative +0xD8 candidate does not establish the source of PcmMd+0x28. The source field semantic name, later AtmosProcessor +0xD8 writer, same object identity, and generation token remain UNBOUND/NOT_CAPTURED.

## Limits

Native capture remains NOT_RUN, DDP input processing remains NOT_CONFIRMED, and F06OF target/generation remains UNBOUND. Other producers and modes are not excluded. No codec algorithm or production-readiness conclusion follows. No PCM, EC3 payload, private key, or raw disassembly is included.
