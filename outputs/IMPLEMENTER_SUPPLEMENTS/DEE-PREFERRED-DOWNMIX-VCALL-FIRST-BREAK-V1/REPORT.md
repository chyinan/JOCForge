# Preferred downmix per-source virtual-call first break

## Bound static call path

The selected encode-pass path calls FUN_1801535b0; a second direct caller is also present. At the first indirect call within that function, the receiver is param_1+0x210 and dispatch occurs through vtable slot +0x30. The Ghidra exact-call xref query found no outgoing target reference for this instruction.

## First break

The virtual target and receiver's typed owner are unresolved. No typed join connects this receiver to MixMode+0x50 or its getter, and no selected audio/BSI consumer or preferred-downmix effect is established. The first break is the vptr load and dynamic slot call. No displacement-wide scan or runtime/audio inference is made.

The upstream manifest SHA-256 is d6faad88e4ccb7dc71768f99484de17dc9c9fe339c3f18bac9ae39eee43eca48; its seven-payload/eight-checksum verifier passed normally and under python -O. The installed DLL was not rehashed. Raw Ghidra trace material is excluded.
