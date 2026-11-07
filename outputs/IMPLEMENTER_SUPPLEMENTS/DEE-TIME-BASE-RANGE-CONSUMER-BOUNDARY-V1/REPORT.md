# DEE time_base: typed range-consumer boundary

Reference-only Wash supplement; it does not authorize seeking or audio changes. `IMPLEMENTATION_READY=NO`.

The selected `encode_to_atmos_ddp` plugin schema declares `time_base` as `xs:string` with `file_position` and `embedded_timecode`; the constructor default is `file_position`, not an observed runtime value. Static construction places the StringType at `Priv+0x550`. The same Priv reaches `FUN_18014fd90`, whose first typed use reads that member and compares it with `embedded_timecode`.

For exact equality with `embedded_timecode`, the inspected range expression adds `(longlong)((double)iVar4 * (double)CONCAT44(uVar21,uVar6))` to a signed 64-bit accumulator. `iVar4` defaults to 48000 and may be replaced by parsed `sampling_rate`; the 64-bit offset is interpreted as a double. The non-equal branch only means “not equal”; it does not prove `file_position`. Full lexical rules, rounding/overflow behavior, subsequent seek, and output timing remain unverified.

The first unresolved edge is host XML value owner/generation to the selected DDP plugin instance's `Priv+0x550` value writer and a particular consumer invocation. Runtime value is `NOT_CAPTURED`; range/seek result is `NOT_RUN`.