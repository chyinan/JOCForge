# Host route profile value to audio-filter index edge

## Result

`ROUTE_TO_PROFILE_CONFIG_OUTPUT_CLOSED; CONFIG_OUTPUT_TO_FACTORY_INDEX_UNBOUND_AT_PRECISE_CALLSITES; NATIVE_OBSERVATION_NOT_RUN`.

The selected host task key `encode_to_atmos_ddp` is consumed by a task-profile option lookup. That lookup returns profile/config option values to the option-application path. Separately, the host audio-filter category loader enumerates the DLL's count of three entries and registers integer indices 0, 1, and 2 under `audio_filter`.

The exact callsite where the profile/config lookup output is applied does not pass a factory index to the audio-filter index registration or a factory entry. The exact index-registration callsite receives the loop counter from the three-entry table enumeration, not the route-profile output. These two paths have no recovered same-invocation or same-owner transfer. The task route's selected factory index therefore remains `UNBOUND`; the registered DLL descriptor name at index 0 is a candidate only.

The route key does gate route-specific profile/config lookup. Static evidence does not show that this gate selects or requires a particular audio filter.

## Minimal observation

Reuse an existing task-owned authorized ADM run if available. At the actual route-to-factory dispatch, capture only the selected route key, the profile/config owner token, the index passed to the factory dispatcher, the invoked factory identity, and the same parent/task-generation token. If the path does not reach a dispatch, preserve `NOT_REACHED`/`UNBOUND`. Do not read or save PCM or bitstream data.
