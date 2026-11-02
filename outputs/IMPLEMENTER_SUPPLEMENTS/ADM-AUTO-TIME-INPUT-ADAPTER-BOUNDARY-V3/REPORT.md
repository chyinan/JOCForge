# ADM auto-time input-property adapter boundary

## Bound distinction

The host `dee::input::AtmosMezz` target-property request (`dee_audio_input_target`) reads normalized per-target rows from object `+0x130` through vtable slot `+0x10`, then serializes those rows into a host property-response string. The separate source-property request (`dee_audio_input_source`) reads a path vector at `+0xb8..+0xc0`; it is not the normalized time map.

The converter is a different image and object. `FilterConvertAtmosMezz+0xb8` points to a separately allocated `PrivContext`, whose `+0xa8` map is read by `FUN_18007fba0` for `offset` and `first_frame_of_action`. The selected static path contains no typed transfer from the serialized target response to that map, and no same input owner/generation join.

## MVP boundary

The default `file_position` path remains unblocked by this missing auto-time adapter. `offset=auto` / `ffoa=auto` is still not established; do not conflate the source path vector with the target-time map. This is a scoped static boundary, not a global-absence claim, and it contains no DEE/audio run or PCM/payload/key/license data.

R3 is continuing the adapter-edge investigation. Recheck the new package before changing implementation if a later receipt supersedes this boundary.
