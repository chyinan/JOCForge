# Candidate argument-2 arena producer boundary

Wash rechecked the sealed Ghidra 12.2 static package, all declared payload hashes, the manifest/seal binding, and its three cited parent manifest/seal pairs. This supplement omits the source package's raw Ghidra export and query script.

The candidate qword store is `FUN_1800d24e0` RVA `0xd252b`, destination `[R8+0xb18]`. Its inspected caller `FUN_18009edb0` receives argument 2 as `param_2+0x348`. The direct producer `FUN_1800729a0` passes argument 2 as `puVar16+0x9a4`; Ghidra types `puVar16` as `undefined2*`, yielding byte offset `0x1348`. In the inspected function, `puVar16` is an incoming cursor aligned to 32 bytes and stored back through `param_1`. The resulting candidate address is therefore statically `puVar16+0x1690` on this path.

The nearest parent call `FUN_18006da40` supplies argument 1 from a child-table entry `+0xdf0` and argument 2 from local `local_140`. These affine/address facts bind the candidate arena path, not its semantic owner or generation. The plan-B `local_68` descriptor path separately reads `[RDI+0xb1c]`; no typed pointer/generation join is established between that descriptor and the candidate arena. Do not promote `[R8+0xb18]` to the third-U32 first writer. First writer, semantic owner, and approved carrier transfer remain `UNBOUND`.

The keyless runtime/capture gate remains `NO_GO`; capture/runtime are `NOT_RUN`. No global absence, runtime behavior, or product-readiness claim is made. No media, PCM, payload, key/license, or implementation source is included.
