# temp_dir provider +0x58 boundary

The focused static callsite at RVA 0x7bb83 binds RCX to the object loaded from path_object+0x68, RDX to the assembled local path value, the indirect target to receiver vtable slot +0x58, and the returned AL to a conditional loop branch. This is callsite argument-flow evidence, not a complete method prototype.

StageRunner Build writes the object at +0x68 from StageRunner+0x290 and separately records a selected descriptor at +0x298. The +0x290 value comes from FUN_140041d50(local_3a8), which obtains a value through input vtable slot +0x18; its concrete class/vtable and the same-path join to +0x298 remain unbound. Therefore the +0x58 target and first file/path action remain UNBOUND.

One read-only 450-instruction function export was used to locate the callsite and included a prior +0x90 instruction. That repeated instruction was not reinterpreted or used for a new claim. No full callee body, runtime owner/generation, DEE/WinDbg run, or file deletion was examined.
