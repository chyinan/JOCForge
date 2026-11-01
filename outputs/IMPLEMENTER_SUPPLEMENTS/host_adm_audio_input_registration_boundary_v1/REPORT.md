# ADM/BWF audio-input registration boundary

## Hash-bound scope

The R138 config selects `input_type=audio.adm` and `route_key=encode_to_atmos_ddp`; its corrected `time_base` is `embedded_timecode`. Current hash-bound runtime identities include the host, Atmos audio-filter plugin, and DACT storage/parser library.

## Registration producer

Ghidra closes the host's generic `audio_input` registration mechanism: the plugin manager resolves the category getter, receives a function-pointer list and count, enumerates the list, and appends pointers to the category registry. This proves generic registration production and storage. It does not identify which module supplies a concrete ADM/BWF reader entry or bind a registry slot to R138's `audio.adm` value.

The current DACT image contains ADM/BWF storage/parser components, but the inspected host/DACT images do not yield a typed DACT ADM factory-to-`audio_input` registry transfer. No separate current-runtime audio-input plugin image was identified; a built-in host factory remains possible. The concrete producer image/ABI is therefore still unbound.

## R138 dispatch boundary

The input-object executor has a virtual `Run` call on its selected reader instance. This is the smallest one-shot point to identify the concrete reader object and vtable target. It is downstream of factory selection. Bind the reader to R138 only if the same wrapper/task owner exposes the typed `audio.adm` input value and a parent-generation token; compare the resolved Run target against the exact host registration table entry under that same task.

The first reader-output handoff into the Atmos/codec path remains `UNBOUND`. If a single coordinated observation is used, capture only pointer/owner tokens and scalar factory/frame values at the category getter, reader Run invocation, and first processing consumer. Do not read PCM or bitstream memory; stop after one attempt.

No authorized audio sample was used in this static Ghidra turn. Wash/R4 is coordinating the consolidated observation; no new native run was started.
