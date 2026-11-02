# Selected ID-0 RenderBus selector field

The selected object is located by looking up RenderBusConfig type ID `0`
(Speaker Monitoring), then forwarding that object to the preprocessor factory.
The factory reads its typed `vtable+0x38` result as the existing
SpatialCoderPlatform-versus-PreProcessorBypass selector.

A `RenderBusConfigBase` getter candidate at `+0x38` returns field `+0x58`.
The examined base-constructor path receives that field from its caller rather
than assigning a literal default. This narrows the contract to a caller-filled
config field; the selected type-ID-0 class and its value in any actual run
remain UNBOUND. The separate RerenderBusConfig candidate has type IDs 3/4/5,
so its selector values and fallback defaults do not apply to the selected
Speaker Monitoring entry.

Keep the value explicit and preserve the conditional dispatch; do not hardcode
zero or nonzero, and do not add a UI prompt based on this static slice. No UI
requirement is established here. The conditional rule is usable for MVP, but
the historical run's branch and exact audio behavior remain unknown.

Wash-only static evidence; no binary, decompilation, media, PCM, payload, key,
or license material is included.
