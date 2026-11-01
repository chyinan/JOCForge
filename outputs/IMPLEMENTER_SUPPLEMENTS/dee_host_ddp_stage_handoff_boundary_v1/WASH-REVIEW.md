# DEE host audio-filter stage handoff boundary

This Wash addendum independently checks the hash-bound host static evidence for the selected audio-filter path. It locates the path from source/target property setup through a generic stage-sequence interface. The interface exchanges stage-pass and status/progress methods; no source-row descriptor is passed at this handoff, and the scoped path contains no store to DDP descriptor offset 0 or typed call to the DDP process entry. A separate thread polls progress only.

A washed DDP-side package establishes that descriptor offset 0 is consumed as the source-row pointer table. That DDP consumer is not linked to the host stage object by this addendum. The concrete generic stage implementation/adapter remains the next boundary to trace. Keep descriptor writer, reader-to-DDP owner, and generation UNBOUND; absence in this scoped host path is not proof of absence elsewhere.

Ghidra static only; no DEE job, live attachment, PCM, row payload, or private-key access. This is a boundary package, not an audio algorithm rule.
