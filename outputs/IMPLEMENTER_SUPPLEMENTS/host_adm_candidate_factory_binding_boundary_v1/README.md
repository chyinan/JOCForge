# Host audio-input candidate binding audit

## Result

The hash-bound host static evidence identifies the R1 candidate as an internal `AtmosMezz` input-object factory path. Its factory method allocates the object and calls the constructor that installs the `AtmosMezz` input vtable. The candidate's supported-input-type provider returns `audio.atmos_mezz`.

This closes the candidate's internal factory-to-object-constructor edge and its own typed label. It does not close candidate-to-`audio_input` getter slot, nor bind the R138 `audio.adm` input owner to this candidate. The host's `audio_input` loader and registry insertion mechanism are generic; the concrete getter table/count and returned slot identities remain unavailable in the static evidence. The candidate factory pointer is an object-vtable method, which is not evidence that the same pointer is a category getter result.

The host also contains `audio.adm` in a separate type-list construction path. String presence or route-list membership does not establish that this internal `AtmosMezz` factory was selected for an `audio.adm` task.

## Minimum R141 join

Capture one same-task, same-parent-generation tuple containing:

1. loaded module identity token;
2. `audio_input` getter identity, returned table pointer and count;
3. selected table slot token and exact factory function-pointer token;
4. typed `audio.adm` input-owner token;
5. instantiated reader object, vtable and resolved `Run` target tokens.

Join those fields by owner and parent-generation tokens. Pointer equality or a matching label without that owner join does not close the selected-reader edge. Stop at this constructor-to-registry/input-type boundary; no downstream audio consumer is claimed here.

## Method boundary

Ghidra 12.2 PUBLIC, using the current hash-bound host program in read-only, no-analysis mode. No IDA, native execution, media payload read, or payload save was used.
