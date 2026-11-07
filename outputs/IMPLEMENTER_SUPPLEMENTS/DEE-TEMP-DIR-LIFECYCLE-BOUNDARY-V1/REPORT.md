# misc/temp_dir lifecycle boundary

Reference-only Wash supplement for temp-directory configuration. It does not implement cleanup behavior. `IMPLEMENTATION_READY=NO` for the `clean_temp` effect.

The selected StageRunner trace binds `misc` to StageRunner `+0x138` and `temp_dir` to `+0x1a0`. Build resolves a storage descriptor at `+0x290/+0x298`, prepares a path object at `+0x2a0`, and stores the resolved path pointer at path-object `+0x68`. `FUN_14007c880` reads the nested `clean_temp` key and writes a byte to that same path object at `+0x90`.

The bounded filesystem-facing use is a `temp_dir_test` writeability probe: construct path, open with write mode, close the stream. This does not prove deletion of the probe or cleanup of job outputs. No consumer of path-object `+0x90` was joined to an output delete/retain action; storage-provider instance and job generation remain unobserved.