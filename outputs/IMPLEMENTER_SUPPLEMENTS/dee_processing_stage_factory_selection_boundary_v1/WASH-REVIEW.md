# DEE processing-stage factory selection boundary

This Wash addendum closes the host registry/config/factory dispatch boundary for the selected audio-filter pipeline. The host enumerates registry entries into stage-config objects. Build invokes each selected config object's factory virtual slot, retains both the config object and returned generic ProcessingStage object, then matches declared source/target capabilities and sets type properties.

The config name getter recognizes pcm_to_ddp among audio-filter names, but the name and capability matching do not prove a concrete plugin type. The registry callback, selected config object's dynamic vtable, concrete factory override, and returned stage vtable remain UNBOUND/NO. The scoped host path still does not show a source-row descriptor or DDP process call.

The next minimum edge is the callback stored in the specific pcm_to_ddp registry entry through the config vtable factory override to the concrete stage implementation. Only after that should the row-data adapter or descriptor offset-0 writer be traced. Ghidra static only; no DEE job, target attachment, PCM, or key access.
