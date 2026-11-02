# Atmos JOC selected ID-0 RenderBus provenance

## Washed contract

Ghidra 12.2 static evidence binds the route from the renderer's RenderBus
configuration vector to the selector getter. The list lookup requests type ID
`0`; the matching object is then forwarded as factory argument 5 and passed
to the existing `vtable+0x38` selector getter. The getter result controls the
existing branch: `0` selects `SpatialCoderPlatform`, nonzero selects
`PreProcessorBypass`.

The newly washed constructor trace narrows the adjacent candidate path. The
examined plugin config constructor copies a caller-supplied opaque vector from
`+0x18` into plugin storage at `+0x138` using `0x20`-byte entries, then the
stage factory forwards that field to `FilterDDPJocEnc`. Its nested `Priv`
initializer consumes the first constructor argument but does not read the
incoming vector argument. This local path therefore does not establish that
the copied vector is the typed RenderBus config vector, does not construct the
selected ID-0 object, and does not write that object's selector field at
`+0x58`.

One concrete `RerenderBusConfig` candidate remains excluded: its visible type
IDs are `3/4/5`, while the selected lookup requests ID `0`. Its tag mapping
and caller-supplied fallback cannot supply the ID-0 object's class or value.
The actual ID-0 concrete class and per-run selector remain `UNBOUND/NO`; do
not invent a default or infer one from the route, XML, or this unrelated
constructor.

## MVP use

Keep the selected object's typed `+0x38` getter and the existing conditional
dispatch in the candidate path. The value should remain an explicit runtime
input. This static slice does not require a user prompt and does not establish
which branch any authorized ADM run selected. The branch can affect audio, so
per-run audio equivalence remains open.

## Evidence limits

This supplement uses only the hash-bound Ghidra summary, two previously
authorized ADM metadata receipts, and historical DEE observations already
listed by the source package. No DEE run occurred in this slice. No raw binary,
decompilation, media, PCM, encoded payload, key, or license material is
included. Synthetic selector variants below are input-only expectations and
were not executed.
