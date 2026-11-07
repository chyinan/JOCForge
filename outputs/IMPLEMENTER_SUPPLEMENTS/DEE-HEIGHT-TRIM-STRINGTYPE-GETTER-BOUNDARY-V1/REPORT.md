# Height trim StringType getter boundary

Reference-only Wash supplement. It binds a type-level getter and stops before the selected field instance/callsite join. `IMPLEMENTATION_READY=NO`.

The schema registers `custom_trims:height_trim_5_1` as a StringType at `Priv+0x9e8` and initializes the object through `FUN_180138690`. The known StringType virtual `toString` getter is `FUN_180152490` at vtable slot `+0x30`; its body copies `param_1+0x08` string data into the output string. This proves a generic value getter exists, not that the height-trim instance is passed to it.

The bounded reference model has no code-call reference to the getter. The field-instance virtual callsite, selected provider/generation, numeric conversion, units, BSI/audio effect remain `UNBOUND`/`NOT_RUN`. Do not interpret declared numeric-looking strings as dB.