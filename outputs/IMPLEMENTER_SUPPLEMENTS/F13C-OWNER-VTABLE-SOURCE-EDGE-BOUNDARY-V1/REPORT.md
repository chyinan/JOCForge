# F13C owner source and vtable edge boundary

## Corrected owner path

The supplied callsite `0x1403c2eee` is inside `FUN_1403c2e50` (`0x1403c2e50..0x1403c2f9e`), not `FUN_1403c2d50`. At that callsite, `FUN_1403c2e50` calls `FUN_1403c0ad0` with the owner argument from `*(param_1+0x8)`. The method is in the `0x140bca558` vtable at slot `+0x20`.

`FUN_1403c3980` occupies slot `+0x8` in the same table and reaches `FUN_1403f5270`, which constructs the `FUN_1403f1dd0` storage-callback wrapper. Ghidra found no direct code caller for `FUN_1403c3980`; the indirect invocation site is unresolved. Neither path reaches a selected DDP-JOC 5.1 stage or F13C writer in this trace.

## Disposition

Reference-only supplement; `implementation_ready=false`. It corrects the caller attribution and narrows the two owner-source edges, but does not resolve the vtable invoker, selected stage, or writer. No audio rule or absence-of-use claim is promoted. The imported binary hash is from historical Ghidra metadata; the current installation was not rehashed.

No encoder, PCM, payload, license, HMAC, or key content was accessed. Ghidra static analysis only.
