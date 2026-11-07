# Host timecode-frame-rate conversion boundary

Reference-only Wash supplement. It binds sentinel/special-token branches and the first conversion helper call, but not numeric semantics. `IMPLEMENTATION_READY=NO` for non-default timecode behavior.

`FUN_14012ff20` compares `param_2` to `not_indicated`; if that property already exists it uses it, while a missing property takes an insertion path. The `29.97df` branch canonicalizes the stored rate string to `29.97` and writes a separate `drop_frame` property, but the flag literal is unresolved.

For non-sentinel rate strings, the first helper receiving the timecode string and selected rate is `FUN_1401359e0` at RVA `0x1306b3`, with its return treated as `double`. Offset parsing first calls `FUN_140054460` and is cast to `double`. The conversion/helper bodies are not expanded; units, accepted numeric syntax, precision, rounding, and rejection behavior remain `UNBOUND`. No full-enumeration or XML-owner conclusion is inferred.