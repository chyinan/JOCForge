# F13C Atmos storage Factory invocation boundary

## Resolved invocation chain

For the hash-identified Ghidra program, `FUN_1403fab60` calls a resolver that constructs an `atmos_storage_master_s` wrapper. The wrapper's `+0x8` virtual method is `FUN_14046bd90`; it loads a child object at `+0x30` and calls a thunk. The thunk dispatches the child's vtable `+0x8` slot to `FUN_1403c3980`. RTTI identifies that child as `AtmosMasterFileStorage::Factory`, not the separate `AtmosMasterFileStorageDesc` callback descriptor discussed in prior evidence.

`FUN_1403c3980` then reaches `FUN_1403f5270`, which constructs the Atmos storage callback wrapper. The target F13C/DDP-JOC writer is still not identified. No selected 5.1 stage or frame-writer edge is established.

## Disposition

Reference-only supplement; `implementation_ready=false`. This closes the indirect invoker and corrects the owner type for this path. It does not provide evidence of a F13C writer or audio effect. Keep that edge `UNBOUND`.

Ghidra static analysis only. No encoder, PCM, payload, or audio output was used; no license, HMAC, or key contents were read. The current installation binary was not rehashed; the recorded image digest is Ghidra project metadata.
