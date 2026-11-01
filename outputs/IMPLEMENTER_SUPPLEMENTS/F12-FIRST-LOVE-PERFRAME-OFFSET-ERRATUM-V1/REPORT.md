# F12 First Love per-frame offset correction

## Correction scope

This erratum corrects one statement in Wash package `F12-FIRST-LOVE-NATIVE-SCALAR-ENCODER-CONFIG-BOUNDARY-V1` (manifest SHA-256 `e3bfcb7bde5fb217071c605c122cd4bfeeabaa8763df013e3c37e5289d047fcc`). That package said the per-frame `param_4[10]` write in `FUN_180178DC0` targeted the same byte offset `+0x3C` as the scalar-bearing `EmpireEnc` field. That is incorrect because the function's `param_1` is typed `int*`.

In `FUN_180178DC0`, expression `param_1 + 0x3C` advances by `0x3C` four-byte elements, so the destination is byte offset `+0xF0`. The per-frame value comes from `param_4[10]`. It is not the scalar-bearing byte offset `+0x3C` established on the configure-time `EmpireEnc` object. The semantic meaning of `param_4[10]` and its relation to final BSI DialNorm remain unbound.

## Unchanged evidence

The First Love measure-only observation remains scalar `-21.010569`, with no unit stated in the native log and no input SHA recomputed in that continuation. The static aggregate-to-encoder-config raw float copy remains bound. The final BSI DialNorm field, plugin unit/scale, and rounding/quantization remain unproven. No new measurement or payload read occurred for this erratum.
