# Host provider initialization boundary for ADM physical N

Wash independently verified the source package manifest and its caller, identity, and ADM-N parent manifest bindings. This supplement contains only sanitized evidence and checker data; it omits raw Ghidra output, scripts, and decompilation.

The selected host caller initializes `FUN_1801828c0` field `+0x48` to zero and calls `FUN_180182330(&param_1[9], 0)`. That helper performs system initialization and registers DAMF, DAMF-create, ADM/BWF, IAB, and composite providers through the dereferenced handle. Its optional `set_filesys` branch requires a nonzero second argument; this caller passes zero, so that call is not taken. The same handle is then passed as parameter 3 to `FUN_1801831c0` for `master_open`.

This closes the local provider-handle setup path only. It does not expose a typed transfer of the separate ADM layout object's `+0x28` physical-N field into that opaque provider registry. The matching `+0x28` callback context is also a separate field and is not an identity join. Preserve the host-provider-handle to ADM-N owner/generation relationship as `UNBOUND`; no global absence, per-run acceptance, or product-readiness claim is made. Runtime remains `NOT_RUN`.

The reused ADM-N receipt identifies layout object `+0x28` U32 N with a static 1..128 gate. This static native admission does not prove a same-run connection to the host/provider handle or the full encoder path. No DEE job or dynamic capture ran; no media, PCM, payload, key, or license was accessed.
