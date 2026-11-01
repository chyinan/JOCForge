# PCM24 reader to DDP descriptor: scoped owner boundary

This package independently washes the R4 Ghidra 12.2 static trace and pointer-only Run148 receipt.

The selected DDP AtmosReader method obtains its own MultiChannelBuffer row table, calls a generic source reader through vtable byte offset 0xc8, then stores that same internal table at descriptor offset 0. Descriptor offsets 8 and 16 carry channel and frame counts. The DDP consumer copies source row i to internal row i for frame_count*4 bytes. The worker passes one local descriptor first to the source read method and then to the DDP process method.

Run148 observed a 36-channel, 1536-frame DDP descriptor and 36 ordinal row tokens on the host-to-DDP thread. The selected ADM reader breakpoints did not hit. Matching virtual slot offsets is not proof that the runtime source receiver is the selected ADM reader. Its object identity, owner and generation remain UNBOUND. The encoder-done log and nonempty output file are context only: no bitstream inspection or playback was performed.

The run used a DEE external license file; its contents were not read or included. This package contains no code, decompilation, media, PCM, bitstream or pointer tokens. Implementation may use only the internal descriptor contract above; do not promote it into an end-to-end selected-input lineage claim.
