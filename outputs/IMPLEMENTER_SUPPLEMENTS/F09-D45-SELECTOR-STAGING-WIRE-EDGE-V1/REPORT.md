# D45 selector staging-to-wire boundary

## Result

The selected Ghidra trace reaches `FUN_180307d90`, but does not reach a DDP or EC-3 bit writer. Keep the selector's wire field and bit range `UNBOUND`.

The evidence extends `F09-D45-FORCE-SELECTOR-BOUNDARY-V2`. That package already defines the static U16 history-to-code rule and its input-only vectors. This addendum records only the downstream boundary found in the newer Wash package.

## Closed edge

The selector staging base is `owner+0x1bae8`. The lane-0/block-0 cell is a U16 at `base+0x3a` (code domain 0..3). The traced path is:

`owner+0x1bae8 -> FUN_180305630 -> FUN_180307640 -> FUN_180306630`

`FUN_180306630` calls `FUN_1803343b0`, `FUN_18032bda0 -> FUN_18032c4b0`, and `FUN_180307d90`. The last function receives the staging base as `param_2`, but its recovered direct base read is `param_2+0x12`. That is `0x28` bytes before the lane-0/block-0 cell; it is not evidence that the cell was read. Its returned short is consumed in the candidate-size/budget path. No indirect call occurs in `FUN_180306630`.

The nearest separately named stuffer path is `FUN_180337980 -> FUN_180337e20`; no value or pointer edge from the selector cells to that writer is established. Do not infer a DDP/EC-3 field from these offsets or call names.

## Evidence boundary

The incoming Wash package `force_d45_native_reachability_wash` has manifest SHA-256 `d7942feb8eade85905562a76c5e4afe201dbb7a162094a21b69a02fc5faa6f42`. Its checker passed in normal and optimized Python modes and verified ten payloads. The relevant image is `dee_audio_filter_ddp_single_pass.dll`, SHA-256 `1767070831dc0d5925a1fc401684ae1447cef2d6bc59659f8a25f53c95f482e9`.

The prior supplement `F09-D45-FORCE-SELECTOR-BOUNDARY-V2` has manifest SHA-256 `6c3218735a9da7964cac061611779c61cf997ad011bef6b840a97497922e8933`. Its six-slot thresholds, U16 code mapping, control override, and staging geometry remain scoped static rules. They are not runtime captures and do not identify the wire representation.

No source, disassembly, binary, PCM, EC-3 payload, or key material is included here. The incoming package's DEE attempts did not capture this selector's runtime values or reach its stuffer boundary. This supplement does not make the overall audio algorithm implementation-ready.

## Next edge

Continue from a Ghidra read-XREF search for the exact lane-0/block-0 staging-cell displacement in the same hash-bound image. Trace the first actual cell consumer through any typed intermediate records to the bit accumulator and exact write width/bit range. Preserve `UNBOUND` unless that edge is observed.
