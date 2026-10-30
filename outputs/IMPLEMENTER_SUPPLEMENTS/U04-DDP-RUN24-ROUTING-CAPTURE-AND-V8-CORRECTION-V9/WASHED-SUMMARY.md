# DDP run24 same-call routing capture and v8 attribution correction

Disposition: NATIVE_REFERENCE_ROUTE_CAPTURED; ADM/selector identity and current-job joins remain open. This update supersedes the prior washed v8 stream-object description.

## Source binding and verification

v9 source manifest SHA-256: 4fbb097f00ee01a2559eb45747d81b761677801b0951c5a4d7fdc7402919d6b5.
v9 EVIDENCE.json SHA-256: 5cfa77a40f51c27e9ac5a906d6cdfd0226a45926d9e9c33fc4b968f9236ba529.
NATIVE-ROUTING-CAPTURE.json SHA-256: deb407163796774f199ce55000865676cecc77fccfd04074ba3c3757130cddd4.
Superseded v8 source manifest SHA-256: 49fe7373af9ec66d7aebfab9641d18ba8b3d96e5b4168ffa7d50cad7bb92d1d5.
Both v9 checkers passed normal and optimized Python; Wash rechecked all listed source member hashes. No raw PCM, source, decompilation, address, module path, license or key is included.

## Run24 DDP route observation

One completed DEE reference invocation observed a 38-channel DDP frame context at one route-consumer call. A primary route group had ten numeric category codes 1..10 mapped to frame pointer slots 0..9. Twenty-eight additional records consumed their first u32 field as indices into the same frame input pointer table, slots 10..37. This binds the DDP local route tables to the 38-slot DDP frame pointer table in that call. Category code meanings remain unbound.

The reference output was 262144 bytes with SHA-256 077977564a8da534a52dee3feb1c791cc0825dd9cd7eb975575799b02450e686. No PCM sample values were read. The reference job XML was 3cf0ff7299c00ff70d09addf755661fafa38930f327a7d72427fa33d189b816e; this is not the current accepted PreparedJobInput XML 20669e276ac7b708b0a03b127d3f37331c8cbc5cfce9a0f4a2498739ecc36cb9. The output matched earlier reference output hash; it is not a JOCForge product output or full target-chain acceptance.

## Correction to v8

The prior v8 claim that AtmosMezz Run directly wrote the stream_object property is withdrawn. v9 reports that method handles metadata/temp-file setup but does not contain that direct property write. A candidate virtual target was not reached. AtmosMezz source-to-stream-object, storage provider-to-PCM pointer table, DDP frame channel-to-ADM UID/physical/global ID, and DDP frame channel-to-selector column all remain UNBOUND. No same-generation join to the separate selector reference was captured. The selector package uses the same BWF hash but a different job XML.

The latest run14 retry fixed misc/temp_dir but still failed DDP loading with Unknown filter before a frame hook; no run14 output was produced. That failure remains a NOT_CAPTURED attempt, not a routing result.

## Status

The native same-call DDP route-table to frame-pointer-table edge is now observed. The frame pointer slots are not ADM identities. Do not equate the 38 DDP slots to CHNA tracks/global IDs, the 48 selector columns, or counts by numeric coincidence. U04 numeric output, Type10, six-plane validation, Gate1, and full product output remain NOT_RUN.
