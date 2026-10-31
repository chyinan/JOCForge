# Encode route profile to audio-filter factory selection

## Result

`R138_ROUTE_AND_PROFILE_CONFIG_HASH_BOUND; DLL_DESCRIPTOR_TO_FACTORY_INDEX_CLOSED; HOST_ROUTE_TO_SELECTED_FACTORY_INDEX_UNBOUND`.

The hash-bound R138 profile selects route `encode_to_atmos_ddp` with input type `audio.adm` and the configured DDP profile. The host task-profile option lookup consumes the route key and returns profile/config option values. The host audio-filter loader separately enumerates the DLL's three factory indices at module load. The index argument at that registration callsite is the enumeration counter; the profile/config output does not flow into it.

The DLL registration crosswalk is statically closed: index 0 has descriptor `encode_to_atmos_ddp`, index 1 `atmos_mezz_to_atmos_ddp`, and index 2 `pcm_to_atmos_ddp`. The same-name index-0 entry is a registration candidate only. The R138 same-call reader-to-codec-to-writer record does not contain a selected factory index or a typed route-to-index transfer.

## Unclosed edge and next observation

`encode_to_atmos_ddp` route/profile output to the selected audio-filter factory index remains `UNBOUND`. The precise missing edge is the consumer that takes the route-specific profile/config object and supplies a selected index/entry to an actual factory invocation. Same-invocation/owner transfer and parent/task-generation are `NOT_OBSERVED`; native observation is `NOT_RUN`.

Reuse a task-owned active run only if Wash confirms the XML route/profile is the same. Otherwise wait for Wash to coordinate one isolated run after the current R4 observation completes. At the actual dispatch, capture only route key, profile/config owner token, selected index/factory, and typed parent/task-generation. Do not read or save PCM or bitstream data.
