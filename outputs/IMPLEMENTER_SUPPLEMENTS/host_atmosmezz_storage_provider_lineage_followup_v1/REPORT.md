# AtmosMezz storage-provider field lineage

## Result

`HOST_STORAGE_PROPERTY_TO_RUN_AND_GETTARGETS_PROVIDER_FIELD_CLOSED; PROVIDER_TO_PLUGIN_PROCESS_PACKET_IDENTITY_AND_GENERATION_UNBOUND; NATIVE_OBSERVATION_NOT_RUN`.

A host property-dispatch method stores the value supplied for property key `storage` in the AtmosMezz reader's storage-provider member. It stores `dee_misc_container` into a different member. The host reader's `Init`, `Run`, and `GetTargets` methods load the same storage-provider member and invoke its interface methods. This closes a static reader-owned provider-field path.

`Run` uses provider callbacks to process each target, writes reader-owned metadata entries, and returns a status scalar. `GetTargets` uses a provider callback and returns target-label strings. Neither method returns the plugin process packet. The provider-field path does not establish that the stored object, a callback result, or a metadata record is the same object received by the DLL process owner.

## Remaining identity boundary

No typed edge from the host storage provider or AtmosMezz reader to the plugin process packet was recovered. Same-object identity, task invocation identity, and parent/source generation remain `UNBOUND`; native observation remains `NOT_RUN`.

## Minimal observation

Reuse an existing task-owned authorized ADM run if available. Record only the reader's storage-provider identity token when the property is assigned and at the Run/GetTargets provider call, then record the process-packet identity token and typed parent/task-generation at the filter dispatch boundary. Compare tokens within the same task invocation. Do not read or save PCM or bitstream data.
