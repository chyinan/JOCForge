# Host route to audio filter factory selection

## Result

`DLL_REGISTRATION_NAME_TO_FACTORY_INDEX_CLOSED; HOST_TASK_ROUTE_TO_FACTORY_INDEX_UNBOUND; HOST_PACKET_IDENTITY_AND_GENERATION_UNBOUND; NATIVE_OBSERVATION_NOT_RUN`.

The hash-matched DLL's three-entry `audio_filter` table has a static descriptor-to-index mapping:

| Factory index | DLL descriptor name |
|---:|---|
| 0 | `encode_to_atmos_ddp` |
| 1 | `atmos_mezz_to_atmos_ddp` |
| 2 | `pcm_to_atmos_ddp` |

Each factory entry passes its own initialized descriptor to the common filter-plugin constructor. Ghidra's static constructor references bind the descriptor names to their globals, and the returned audio-filter handle's ordered table binds those globals to indices 0, 1, and 2. The host's category loader enumerates and registers all three indices under `audio_filter`.

## Host route boundary

The hash-matched host contains the task/config key `encode_to_atmos_ddp` and separately loads the plugin category `audio_filter`. Static code in the bounded host route registry and category loader does not transfer that task key into a selected factory index. The DLL's index 0 has the same descriptor name, which identifies a registration candidate; string equality alone does not prove that the selected host task calls index 0.

Therefore the requested route-to-index result remains `UNBOUND`. Runtime task selection, the live parent/task token, and the host `AtmosMezz::Run`/`GetTargets` packet to plugin process-packet identity also remain unobserved.

## Evidence boundaries

- The three index-to-descriptor edges are Ghidra static evidence from the hash-matched DLL.
- The host category loader statically enumerates indices 0 through 2; it does not establish which index the task route selects.
- The host route key is statically registered in its task/config path. Runtime selection of that key was not observed.
- Native execution is `NOT_RUN`; host packet identity and parent/source generation are `UNBOUND`.
- No PCM, encoded payload, key, license, or authentication material was read or saved.

## Minimal next observation

Reuse the already-authorized task-owned ADM run if it remains available. At the route-to-factory dispatch boundary, record only the selected task-route identifier, factory-table index, invoked factory entry, and a typed parent/task-generation token. Confirm the factory-entry call belongs to the same task invocation. Do not read or save PCM or bitstream bytes. If the authorized run does not reach this boundary, preserve `NOT_REACHED` and `UNBOUND`.
