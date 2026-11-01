# F06ND selected category-5 process boundary

R3's hash-bound Ghidra package reports that the selected AtmosProcessor frame path receives a PCM frame descriptor, validates frame size and channel count, copies channel samples into an internal buffer, calls the processing backend, and prepares an output descriptor. The reviewed process interface has no category-5 parameter or InputControlEvent vector. For this selected process path, a category-5 byte layout is not applicable and no category-5 FIFO write is observed.

The same native image contains a separate generic storage API that accepts InputControlEvent vectors. The selected category-5 submission has not been joined to that API or to a downstream BSI/output consumer; that effect remains UNBOUND. No layout is inferred for it.

The source package's logical-contract hash is recorded, but the referenced source file was unavailable at the declared path for an independent rehash in this workspace. The native DLL hash was independently recomputed. The package remains PARTIAL, with implementation readiness false. No source, disassembly, binary, media, PCM, or private key is included.
