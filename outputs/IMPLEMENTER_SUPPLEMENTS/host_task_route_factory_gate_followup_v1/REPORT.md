# Host task route gate to audio-filter factory index

## Result

`HOST_ROUTE_TO_PROFILE_CONFIG_LOOKUP_CLOSED; HOST_ROUTE_TO_FACTORY_INDEX_UNBOUND; PACKET_IDENTITY_AND_GENERATION_UNBOUND; NATIVE_OBSERVATION_NOT_RUN`.

The exact host task-profile option lookup consumes the route key `encode_to_atmos_ddp` and resolves configuration values for that route. Its result is profile/config data. The inspected callsite does not pass a factory index to the `audio_filter` category registry or invoke a selected factory. This route key gates route-specific configuration lookup; it is not evidence that the task selects a particular plugin factory.

The hash-matched DLL does have a closed registration crosswalk: `audio_filter` factory index 0 carries descriptor `encode_to_atmos_ddp`, index 1 carries `atmos_mezz_to_atmos_ddp`, and index 2 carries `pcm_to_atmos_ddp`. The literal match between the host task key and index 0's DLL descriptor is a candidate identity only. No typed transfer or route-conditioned branch from the host lookup to index 0 was recovered.

## Remaining edge

`encode_to_atmos_ddp` → selected `audio_filter` factory index remains `UNBOUND`. Runtime task selection and parent/task generation were not observed. The host `AtmosMezz::Run`/`GetTargets` packet to the plugin process packet remains `UNBOUND` as recorded in the prior Wash-only package.

## Minimal observation

Reuse an existing task-owned authorized ADM run if available. At the route-to-factory dispatch boundary, capture only the selected route identifier, selected table index, invoked factory, and a typed parent/task-generation token. Do not read or save PCM or bitstream data. If the run stops before this boundary, preserve `NOT_REACHED` and `UNBOUND`.
