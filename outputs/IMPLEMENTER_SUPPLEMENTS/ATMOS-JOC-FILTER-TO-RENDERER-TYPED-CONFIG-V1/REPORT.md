# Atmos FilterDDPJocEnc to typed renderer config

Ghidra 12.2 static evidence traces the local Atmos backend from the
`FilterDDPJocEncPlugin` stage factory through `FilterDDPJocEnc`, its
`AtmosProcessor` adapter and lazy renderer initialization to the
`SymphonyRenderer` constructor. The constructor receives a typed aggregate
as argument 6 and transfers the aggregate's nested object-vector header
from `param6+0x18/+0x20/+0x28` into renderer `+0x78/+0x80/+0x88`.

The type-ID-0 Speaker Monitoring lookup uses renderer `+0x60` as a
subobject/outer-vector base. `FUN_41f1e0` reads begin/end at receiver
`+0x18/+0x20`, which are renderer `+0x78/+0x80`; capacity `+0x88` is not
read. This reconciles the two offsets: the renderer has an outer record
vector and a nested object lookup vector.

The typed aggregate carrier and its transfer into the lookup vector are now
bound on this local path. The concrete type-ID-0 element constructor,
upstream owner that supplies that element, and its `+0x58` selector writer
remain `UNBOUND/NO`. The examined RerenderBusConfig row is type ID 3/4/5 and
does not establish the selected ID-0 object. Keep the selector conditional
as an explicit typed input; do not invent a default or user prompt.

The plugin-local opaque `+0x18 -> +0x138` vector remains a separate
configuration object unless a typed join proves identity. No per-run
selector value or audio-equivalence result is established here.

## Scope

Static Ghidra evidence only; no DEE run, media/payload/PCM/key access, or
implementation contact. The supplement includes no binaries, raw
disassembly, decompilation, media, PCM, encoded payload, key, or license
material.
