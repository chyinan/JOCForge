# DDP auxiliary and AtmosMezz reader ownership boundary

**Status:** DDP factory/backend auxiliary owners and the host AtmosMezz object's internal reader/serializer path are separately bound; their same-run owner/generation join and the converter setter remain unbound.

## DDP stage-owned auxiliary path

The selected `encode_to_atmos_ddp` factory allocates a `FilterDDPJocEnc` stage and a fresh `dee::proc::Priv` backend. The stage-local map at `FilterDDPJocEnc+0xa8` is initialized and queried by the DDP stage; the value writer for that map is unbound in this scoped export. A different DDP setter stores `dee_misc_container` and `dee_logger` at backend `+0xf20/+0xf28`. The bounded `+0xf20` consumer passes the misc-container to a MediaInfo/WAV metadata probe; `+0xf28` reaches DDP debug logging. These are DDP-owned auxiliary edges, not an AtmosMezz/converter owner transfer.

## Host AtmosMezz path and converter boundary

In the host image, `dee::input::AtmosMezz` is constructed separately; its property setter handles input fields, its `Run` method writes normalized sampling/timecode/offset/FFOA rows to `this+0x130`, and the `dee_audio_input_target` response callback serializes rows from that same typed field. The converter is a third owner: its plugin-created `PrivContext` at plugin `+0xb8` has a consumer of an offset/first-frame map at `PrivContext+0xa8`. The host response-to-converter setter remains unbound.

The DDP stage map `+0xa8`, the converter `PrivContext+0xa8`, and the AtmosMezz `this+0x130` are different objects/offsets in different images. No typed pointer/call/copy establishes a same-run identity or generation between them. Do not infer an owner join from the equal `+0xa8` offset or from DDP auxiliary field names.

This boundary is scoped to the listed static paths; it does not claim that the native images contain no other route. No runtime observation, DEE run, media/PCM/payload/key read, or implementation contact occurred.

## Verification scope

Wash reran the source package checker normally and under `python -O`; both passed its package and source-hash checks. The package contains no media, PCM, binary, Ghidra project, or raw disassembly; its source-identity file records hashes for the referenced images and Ghidra artifacts.
