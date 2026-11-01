# Reader metadata probe and current E-drive setup pointer lifetime

## Schema-only host observation

A hash-bound `dee.exe --schema` invocation had no task profile, input audio, route selection, or job XML. It reached startup metadata registration, the audio-filter load-time registration site, and a generic schema-registry consumer. The observed metadata callbacks included AC-4 input and AC-3 output descriptors. They did not identify a selected R138 `audio_input` reader. The schema's `encode_to_atmos_ddp` declaration is not a selected route observation.

## Current E-drive converter image

The current converter image hash is `bdb502fe147ad825be717b6b1ac450a103ba937ea6782d931da8ddb2e3cc8d80`, distinct from the earlier D-drive image `9972be4f952ca5f9121fa83ef941b29d4c1e6baf0ff1f708baaaedffa1bd76a8`. The local type-list candidate includes `audio.adm`, but no typed edge connects it to the host audio-input selection.

In the selector-3 setup branch, `FUN_1802851A0` initially assigns `param4[2]` to `this+0x50`; on the successful branch it then allocates a new `count*6` table, overwrites `this+0x50`, and copies the records. Thus the input pointer is transiently aliased before allocation, while the post-allocation table is distinct. This updates the lifetime detail of the earlier static copy-chain supplement; it is not an R138 selected-reader or DDP packet identity join.

## Exact remaining edge

R138 `audio.adm` reader/factory selection remains `UNBOUND`. The selected host reader, converter setup pointers, DDP worker, and packet normalizer were not observed in one activation. The included metadata-only observation plan is `NOT_RUN`; it is a research plan, not a runtime receipt. No PCM, payload, row/table bytes, key/license contents, or encoded output were read by this analysis.
