# AssignTempFile owner/path delegate boundary

Reference-only Wash supplement. The wrapper's first delegated call is bound, but the callee's file operation remains open. `IMPLEMENTATION_READY=NO` for cleanup semantics.

`FUN_140067610` prepares `local_a8` from `param_4` and `local_88` from `param_2`, then calls `FUN_140066230(param_1, local_88, param_3, local_a8)` at `0x1400676d5`. The callee body was not inspected. Therefore creation, deletion, or queue registration is not established.

The prior temp-path receipt binds the caller-side owner argument to `*(path_object+0x70)` and records generated temporary path arguments. That does not bind a runtime job/provider identity. The `clean_temp` byte at path object `+0x90` remains unjoined to an output delete/retain consumer.