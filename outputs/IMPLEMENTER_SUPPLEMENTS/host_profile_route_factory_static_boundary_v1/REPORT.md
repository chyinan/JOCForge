# Host profile to factory static boundary

## Hash-bound scope

The selected R138 execution config is hash-bound to route `encode_to_atmos_ddp`, input `audio.adm`, and `time_base=embedded_timecode` (the time-base correction is attributed to Wash's verification of the hash-bound XML).

## Static role edges

Ghidra closes the route/profile option lookup and the registration of profile fields such as `start` and `time_base`. It also closes host setup from job configuration through StageRunner construction, plugin loading, and category factory-pointer registration. A separate StageRunner path constructs the XML schema.

These are configuration/setup/schema edges. The category registration loop enumerates available factory pointers; it does not identify the factory chosen for this R138 task. No typed transfer from the R138 profile owner or embedded-timecode consumer to a selected factory index/entry was recovered. A generic factory entry hit is non-gating for R138 until the task/profile owner and parent generation are joined.

## Status

`R138_PROFILE_TO_SELECTED_FACTORY=UNBOUND`; `ACTUAL_ROUTE_CONDITIONED_HOST_DISPATCH=NOT_IDENTIFIED`; `PARENT_TASK_GENERATION=NOT_OBSERVED`; `NATIVE_OBSERVATION=NOT_RUN`.

No raw decompiler text or Ghidra function symbols are included in this Wash-only addendum. No native job was started.
