# MP4 SetProperty owner first break

Reference-only Wash addendum. It identifies one untyped function-pointer data slot; it does not bind an output writer or authorize MP4 execution. `IMPLEMENTATION_READY=NO`.

The reused output-format receipt binds candidate `FUN_1401496e0` to the `dee::output::Mp4::SetProperty` body. The exact data slot `0x1402f2760` contains qword `0x1401496e0`; Ghidra's references-to query for that slot returns no references, and the slot is `UNDEFINED` rather than a typed vtable entry.

This does not identify a caller/factory or connect the setter object to the separate `Mp4::Run → Mp4::InitPlugin` path. Same-instance muxer selection, writer creation, runtime output, and container behavior remain `UNBOUND`/`NOT_RUN`.