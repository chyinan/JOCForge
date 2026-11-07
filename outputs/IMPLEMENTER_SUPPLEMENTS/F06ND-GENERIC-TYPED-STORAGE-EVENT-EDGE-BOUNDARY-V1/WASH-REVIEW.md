# F06ND generic typed storage-event edge

One generic `writeData<int>` path submits an `InputControlEvent` vector to an event helper, which conditionally forwards matching entries to its first per-event consumer. The template `int` identifies the storage/sample-buffer type; it is not proof that the event belongs to selected F06ND Category5.

The event-vector producer, owner, and generation remain unjoined to the selected Category5 source (`NO_SELECTED_JOIN_ESTABLISHED`). DRC/BSI output effect is `UNBOUND/NO`; runtime is `NOT_RUN`. This is a scoped result, not a global absence claim. No PCM path or event-record layout is reconstructed.

No source, decompilation, binary, PCM, payload, key, authentication material, local absolute path, or raw address is included.
