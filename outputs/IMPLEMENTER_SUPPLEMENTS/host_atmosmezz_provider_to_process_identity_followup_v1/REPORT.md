# AtmosMezz provider to plugin process identity

## Result

`RUN_AND_GETTARGETS_RETURN_VALUES_NOT_SAME_AS_PROCESS_PACKET; STORAGE_PROVIDER_TO_PROCESS_PACKET_UNBOUND; SAME_TASK_GENERATION_NOT_OBSERVED`.

The host reader's `Run` return is a completion-status scalar, `GetTargets` returns target-label strings, and a nested provider callback value is wrapped into a host filename/message record. These direct return values are `NOT_SAME` as the DLL process packet. The host reader stores a `storage` provider object and calls that object from `Init`, `Run`, and `GetTargets`; the stored provider object's identity to the DLL process packet is not statically transferred or observed.

The DLL process consumer operates on a frame packet accepting a 1536-unit frame or a zero-size flush packet. The DLL also constructs a local per-source reader and obtains its frame before calling the process consumer. No typed transfer from the host provider field, target-label list, or reader-owned metadata to that DLL frame packet was recovered. The packet edge remains `UNBOUND`; no same-task activation or parent/source-generation token was captured.

## Existing-run applicability

The hash-bound R138 configuration uses route `encode_to_atmos_ddp` with input type `audio.adm`, so it does not exercise the AtmosMezz storage-provider path. Wash reports that R138 is complete with no live PID or WinDbg session, and R4's run08/09 are separate completed generic-audio tasks. Neither is reusable for this AtmosMezz-specific provider identity observation. No new run has been started; the next native attempt awaits Wash coordination.

## Minimal observation

If Wash coordinates one task-owned run with an `atmos_mezz` input and the target encode route, pre-arm a bounded scalar-only capture at the storage-property assignment, AtmosMezz provider callback, selected factory entry, and DLL process entry. Record only opaque object/pointer identity tokens, route id, factory index, packet frame-size scalar, and a typed parent/task-generation token. Keep PCM/bitstream memory unread. If initialization or the route misses these boundaries, record `NOT_REACHED` and do not retry.
