# `time_base` job option tree to DDP setter boundary

Static Ghidra evidence follows an already parsed/accepted job_config object into the same StageRunner instance. Its `filter/audio` subtree is retained, then StageRunner builds the selected DDP plugin property tree and dispatches the nested `encode_to_atmos_ddp/time_base` key/value through the plugin's StringType property setter at `Priv+0x550`. Explicit accepted values override the descriptor's `file_position` default. This closes the static producer path from parsed option tree to setter.

The original XML bytes and a concrete parsed job configuration were not inspected in this pass. Whether omitted XML defaults are materialized upstream, whether a specific runtime job accepted the value, and its same-run owner/generation remain UNBOUND. E-203's nonzero comparator results therefore remain only “not equal to embedded_timecode”; they are not upgraded to `file_position`.

This static supplement contains no XML, raw native code, decompilation, PCM, payload or credentials. The source package supplied no concrete input-only job vector, so none is claimed here.
