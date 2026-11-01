# D45 selector cell to bit accumulator boundary

## Result

The latest hash-bound Ghidra package closes a static path from selector U16 cells to a generic bit accumulator in both current-frame and predecessor-frame paths. The field name and absolute stream-relative bit range remain `UNBOUND`.

## Typed path

The lane-0/block-0 selector cell is a U16 at `owner+0x1bb22` (also represented as `owner+0x1bae8+0x3a`). Its geometry is `owner+0x1bb22 + 10*block + 2*lane`; the staged code domain is 0..3. `FUN_1802fa870` reads the code and writes difference-derived U16 symbols into a shared pointer-table entry at `owner+0x140f0+0x2c8+8*(5*block+lane)`.

The current-frame path `FUN_18033c670` and predecessor-frame path `FUN_18033b2f0` read that shared table. Both call `FUN_1802fd640` for one 4-bit value and for 7-bit symbol values. The generic writer masks each source short to the requested width, appends it to a 16-bit accumulator, emits a U16 when the accumulator fills, advances the output pointer, and carries remaining bits.

This binds selector-code-to-symbol-buffer-to-generic-bit-accumulator. It does not identify the semantic DDP/EC-3 field, the writer-state initializer/field order, or an absolute bit offset. Do not label the 4-bit value as a specific wire field without that join.

## Evidence boundary

Incoming package `force_d45_native_reachability_wash` manifest SHA-256: `3a525c19200e50424d4109013f898ca59b34a11d52ecc11fb21c6885e56cdb4a`; normal and optimized checkers passed. The relevant image is `dee_audio_filter_ddp_single_pass.dll`, SHA-256 `1767070831dc0d5925a1fc401684ae1447cef2d6bc59659f8a25f53c95f482e9`.

Existing washed references define the source/history-to-selector code rule and the earlier size-budget branch. This addendum adds only the exact selector read-XREF to bit accumulator. It includes no source, disassembly, binary, PCM, EC-3 payload, or key. No run captured runtime selector values; overall audio algorithm implementation readiness is not claimed.
