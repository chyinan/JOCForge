# F12 ID12 count and frame-set parent boundary

## Result

Static Ghidra evidence shows the ID12 setter loop and frame-output loop use the same `piVar1` parent and pointer-table base, but independent count fields: setter count at `+0x4290`, output count at `+0x4294`, shared pointer table at `+0x4298`. Their code contains no static equality invariant. Thus unmatched tail entries are possible if the runtime counts differ; the package does not show that either tail occurs in a real invocation.

For indices within both loop bounds, both paths dereference the same table slot. The frame path passes `stream+0xb28` as the frame-set pointer; `FUN_140317600` stores it in `RSI` and forms the current record at `frame_set + RBX*0x38 + 0xd0`. This is static same-call parent/index/frame-set evidence, not a captured generation token or a final U5 mapping.

The earlier typed handoff remains corrected: `longlong*` arithmetic maps the source read to `stream+0x9c`, copied to nested context `+0x34` and then `+0x19140`. This addendum does not reanalyze that copy or R5's U5 writer.

## Evidence and limits

Source package `multi-adm-dialnorm-cross-input`, MANIFEST SHA-256 `439df32fce6b2dfe8f20026225536ef1e954784d1e99fd50d94e1a0b27b2ed33`, passed normal and optimized checkers over 77 payloads. The target image is `dee_ddp_encoder.exe`, SHA-256 `0cf23520799f52323b0994e5fcc16924da7b027a5b83a0cff3b4f06792c18532`.

- **CLOSED_STATIC:** same parent/table slot for common setter/output indices; frame-set pointer to frame record arithmetic.
- **UNBOUND:** actual setter/output counts in any run, tail presence, runtime generation token, and final U5 writer relation.
- **NOT_RUN:** no DEE run or media/payload read.

If a real run later requires closure, the minimum observation is one `FUN_140315db0` invocation: read both counts at the stated boundaries, record table-slot identities up to their maximum, and retain the shared stream's frame-set pointer plus frame ordinal. No frame or media bytes need to be read.
