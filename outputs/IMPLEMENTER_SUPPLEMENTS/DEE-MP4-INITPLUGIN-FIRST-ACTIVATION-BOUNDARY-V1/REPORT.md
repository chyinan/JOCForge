# MP4 InitPlugin first activation boundary

Reference-only Wash supplement. It traces `output_format` to the first plugin activation call, not to a concrete muxer or runtime output. `IMPLEMENTATION_READY=NO` for MP4/DASH output behavior.

`Mp4::Run` passes its `param_1` unchanged into `Mp4::InitPlugin`. InitPlugin reads output-format property data at `param_1+0x1d8`, builds a local property array with 0x18-byte entries, and passes its header to an indirect interface slot `+0x18` through plugin pointer `param_1+0x260` at `0x140148c7d`. The receiver comes from `(*(longlong **)(param_1+0x260))[1]`.

This binds the property array to the first activation call only. The indirect target/ABI, selected muxer, and same-object connection from SetProperty member `+0x3b` to this Run/InitPlugin instance remain unbound. No container output or audio effect is inferred.