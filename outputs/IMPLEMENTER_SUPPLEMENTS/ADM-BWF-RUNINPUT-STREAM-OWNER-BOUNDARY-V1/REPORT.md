# ADM BWF RunInput stream-object ownership boundary

**Status:** host `RunInput` stream-object handoff to a DDP-stage auxiliary setter is bound statically; the selected RunInput receiver identity, AtmosMezz reader owner, and converter `PrivContext+0xa8` join remain unbound.

## Bound host path

At the hash-bound host callsite, `StageRunner::RunAudioTask` calls `FUN_14004b920` using the stage pointer from `Runner+0x270` and requests `dee_audio_input_target`. `RunInput` asks the same receiver's vtable `+0x10` for a target list, resolves each `.stream_object` using that receiver's vtable `+0x20`, and stores the returned opaque pointer in `StageRunner+0x1158`. The first host consumer calls the same stage receiver's vtable `+0x18` with the target key and stream-object pointer.

For the hash-bound DDP `FilterDDPJocEnc` candidate, vtable `+0x18` stores only `dee_misc_container` and `dee_logger` at backend `+0xf20/+0xf28`. Vtable `+0x20` looks up a pointer in a DDP-stage-local map at `FilterDDPJocEnc+0xa8`. These are DDP stage fields, not the converter's `PrivContext+0xa8`; equal offsets across different images/objects do not establish identity.

The separate AtmosMezz endpoint serializes normalized properties from `dee::input::AtmosMezz+0x130`; the converter allocates a `PrivContext` at `+0xb8` and its `setRange` path reads a map at `+0xa8`. No same-run typed owner/generation edge joins the host `.stream_object` to that AtmosMezz object or converter map. The selected RunInput receiver's exact runtime stage instance/factory identity is also not captured.

## Boundary and next observation

The remaining relation is a same-run typed identity/generation join from the `.stream_object` returned by the selected stage to the AtmosMezz reader owner or converter `PrivContext+0xb8/+0xa8`. The included auto-versus-explicit offset/FFOA matched-control is a plan only; it was **NOT_RUN**. It records pointer tokens and owner/generation metadata only, with no PCM or encoded payload reads.

## Scope and limits

The source package checker passed normally and under `python -O`; its five payloads, manifest, and seal were verified by Wash. The package reports hashes for the host, DDP, and converter images and references Ghidra evidence files by hash, but those raw evidence files and binaries are not included in the package and were not independently rehashed here. No DEE run, media/payload/PCM read, key/license read, or implementation contact was performed for this static addendum. This does not block using the already-established host input route, but it does not authorize an invented owner join.
