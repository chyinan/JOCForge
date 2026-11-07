# temp_dir first delegate boundary

Reference-only Wash supplement. It identifies the first delegated call on one branch, not a file-system operation or cleanup effect. `IMPLEMENTATION_READY=NO`.

On the fallthrough path where byte `[param_1+0x160]` is zero, `FUN_140066230` reaches an indirect vtable call at `0x1400662b3` before any identified file-system/cleanup API. The vptr is read from the object in R8; slot `+0x18` is invoked. The target method and operation remain unknown. The alternate branch at `0x1400662a7` was not inspected.

The observed call inputs do not include path-object `clean_temp+0x90` as an explicit argument, and the `param_1+0x160` condition is not joined to that path object. No relation to create/delete/retain behavior is established.