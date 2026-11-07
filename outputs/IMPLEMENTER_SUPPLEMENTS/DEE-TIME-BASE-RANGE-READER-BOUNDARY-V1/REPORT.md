# DEE time_base range-to-reader boundary

## Typed field and host boundary

The plugin field is a `StringType` at `dee::proc::Priv+0x550`, with setter `FUN_18014f820` and getter `FUN_180152490`. Its constructor default `file_position` is not evidence of a selected job value. The host path stores a descriptor-vtable `+0x30` result at `StageRunner+0x270`, then calls generic SetProperty vtable `+0x8`; neither dynamic target is bound to the Atmos setter. The observed host keys are source/target, not `time_base`.

## Range path

For a value already resident at `Priv+0x550`, `FUN_18014fd90` is the first field-specific consumer. It checks `embedded_timecode`, handles start/end values, calls a range converter, validates bounds against source sample count and rejects end-before-start, then writes sample bounds at `Priv+0xf98` and `Priv+0xfb0` using seconds and sample rate.

The arithmetic is only partly closed. One summarized branch applies source-offset adjustment for `file_position` and skips it for `embedded_timecode`; detailed evidence conditions a `SUBSD XMM7,XMM6` on the comparison result, a stack-slot state, and a secondary string gate. Another subtraction's operands remain unresolved. Offset units, the multiplier in the boundary helper, and end-specific offset behavior remain `UNBOUND`.

## Reader boundary

The range record is statically passed into Reader input construction. Reader choice depends on per-source byte `+0x140` (`AtmosReader` or `AtmosReaderCC`). The sample bounds `+0xf98/+0xfb0` are not joined to Reader cursor `+0x28` or total `+0x20`, and the runtime Reader owner/branch is not selected.

Selected XML value, host property-to-setter join, runtime branch/owner/generation, and sample vectors remain `UNBOUND`/`NOT_RUN`. No DEE run or media/PCM/payload/key/license access occurred.

## Provenance

Source package `dee_time_base_xml_to_sample_range_reader_boundary_wash_v1` manifest SHA-256 `58a7cd433fcda8d8c1c4d63758908e21ee34f425f6f3c03356d4ee5d847a59e5`, seal SHA-256 `dc6e34572ffc09f8509061a7b35c1b2673262e52b9d52e1e634830f90b86d88c`; its normal and `python -O` checker passed.
