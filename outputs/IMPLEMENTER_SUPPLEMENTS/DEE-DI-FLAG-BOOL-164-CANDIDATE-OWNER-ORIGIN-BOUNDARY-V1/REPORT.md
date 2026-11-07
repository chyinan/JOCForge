# DI `+0x164` candidate owner-origin boundary

Wash independently verified the Ghidra 12.2 read-only single-function package, its 13-file checksum inventory, target image hash, and two parent receipts. This supplement omits the raw trace, scripts, and disassembly.

The inspected candidate reader is `FUN_1801bcf40` (`param_1+0x164`). Its inspected direct caller `FUN_1801febe0` walks the codec-parent child-record pointer array at `param_1+0x10cc`, selects `lVar6=*plVar8`, and supplies `*(undefined8 *)(lVar6+0xdf0)` as candidate `param_1`. In this caller slice, `lVar6` is the nearest typed owner; the `+0xdf0` pointer target is opaque, and no field write/constructor for it appears in the slice.

No typed pointer/owner/generation join connects this opaque target to the Atmos engine subcontext at `engine+0x1d0`. The Bool-to-result causality remains `UNBOUND`; this one-caller result is not a global absence claim and does not establish any audible effect. New Ghidra query, DEE, runtime capture, UI, audio/PCM, key/license, and implementation access are all `NOT_RUN`/not performed.
