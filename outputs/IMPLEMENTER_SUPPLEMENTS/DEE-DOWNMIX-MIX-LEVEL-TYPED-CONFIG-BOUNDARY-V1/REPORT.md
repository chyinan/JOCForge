# Four downmix mix-level typed-config boundary

Reference-only Wash supplement. It binds lexical tokens to an internal typed `MixLevelValue` U32 and per-field storage candidates; it does not establish BSI values, rendered downmix, or audio effect. `IMPLEMENTATION_READY=NO` for effect changes.

The four fields remain independent: Lo/Ro center `Priv+0x7D0`, Lo/Ro surround `+0x828`, Lt/Rt center `+0x880`, and Lt/Rt surround `+0x8D8`. The plugin maps lexical values to internal codes: `+3→0`, `+1.5→1`, `0→2`, `-1.5→3`, `-3→4`, `-4.5→5`, `-6→6`, `-inf→7`. Center accepts all eight; surround accepts the five codes greater than 2. Each object statically defaults to lexical `-3` / internal code 4.

The first type-specific conversion is `FUN_1801700d0` at MixLevel vtable `+0x28`; it maps the lexical token and dispatches U32 to `+0x50`. `FUN_18014f6c0` stores U32 at member `+0x50` and sets flag byte `+0x08`. No runtime property event, selected owner/generation, native BSI consumer, or rendered audio effect is bound. DME's host-key bridge remains a separate open edge.