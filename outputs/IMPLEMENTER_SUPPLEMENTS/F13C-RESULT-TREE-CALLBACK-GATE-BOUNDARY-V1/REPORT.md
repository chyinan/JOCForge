# F13C result-tree callback gate boundary

## Static finding

For the `AtmosMasterFileStorageDesc` registration, `FUN_1403f1dd0` stores the caller-supplied owner at `parent+0x6370`. The descriptor-backed owner `+0x10` context resolves in the imported Ghidra image to static `.rdata` object `0x141038a80`; its gate byte at `+0x1D0` is zero. The dispatcher `FUN_14045B6A0` therefore skips the indirect callback for this descriptor's event codes 2 and 3.

The two result adapters construct generic result payloads and call the dispatcher. The payload is not a typed `dmixmod` value. This closes the generic result-tree callback path for this descriptor only.

## Consumer boundary

The trace does not connect this result-tree owner or its generic payload to the selected DDP-JOC 5.1 stage or F13C frame writer. `FUN_1403c2d50` supplies one unresolved owner at callsite `0x1403c2eee`; `FUN_1403c3980` reaches another parent path whose vtable caller remains unresolved. The next static check is limited to those two owner inputs and whether either supplies a different context with a nonzero gate and a typed writer edge.

The imported image digest comes from historical Ghidra program metadata; the current installation binary was not rehashed. No encoder, audio data, payload, license, HMAC, or key content was read or generated.

## Disposition

Reference only; `implementation_ready=false`. Do not infer that `dmixmod` has no audio effect globally, that the UI `Stereo` label is its native default, or that this output-side result callback represents the selected 5.1 writer. Keep the F13C writer and preferred-downmix mapping `UNBOUND`.
