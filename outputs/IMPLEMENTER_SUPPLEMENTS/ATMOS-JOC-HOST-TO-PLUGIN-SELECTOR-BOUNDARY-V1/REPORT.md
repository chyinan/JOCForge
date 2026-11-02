# Host StageRunner to Atmos RenderBus selector boundary

Hash-bound Ghidra evidence connects the host StageRunner's generic plugin
factory to the selected Atmos plugin configuration. StageRunner calls the
selected config object's vtable `+0x30`; it stores the returned stage at
`Runner+0x270` and retains the config at `Runner+0x278`. Separate Run145/148
route evidence identifies `encode_to_atmos_ddp` as
`FilterDDPJocEncPlugin`/`FilterDDPJocEnc` in the Atmos module.

The host factory call does not explicitly carry a RenderBus vector or
selector. The Atmos renderer receives a caller-provided RenderBus vector and
selects type ID 0 for Speaker Monitoring, whose `vtable+0x38` result controls
the SpatialCoder-versus-PreProcessorBypass branch. The producer/adapter that
supplies that vector and ID-0 selector lies beyond the host edge traced here.
The targeted `.rdata` search found no literal route names, but that alone does
not exclude dynamically supplied configuration.

For implementation, preserve the typed selector and conditional branch; do
not infer it from route text, job XML, a user prompt, or a hardcoded default.
This boundary does not block a best-effort conditional MVP, but it does leave
the exact branch/audio behavior of prior runs unknown. Ghidra static only;
no binaries, decompilation, media, PCM, payload, key, or license were included.
