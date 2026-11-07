# Custom 5.1 trim XML boundary

## Disposition

Reference-only Wash evidence. The package binds the declared trim enums and constructor defaults, but it does not show XML token transfer, omission behavior, numeric conversion, or audio/BSI effects. `IMPLEMENTATION_READY=false` for trim behavior.

## Fields

| XML field | Declared choices | Typed member | Static constructor value |
|---|---|---|---|
| `custom_trims/surround_trim_5_1` | `auto`, `0`, `-3`, `-6`, `-9` | `StringType` at `FUN_180136220` param+`0x968` | `auto` |
| `custom_trims/height_trim_5_1` | `auto`, `-3`, `-6`, `-9`, `-12` | `StringType` at `FUN_180136220` param+`0x9e8` | `auto` |

The omission rule is `UNBOUND`; a constructor default does not prove that the selected XML job omitted the field or that `auto` survives XML processing.

For surround, `FUN_1800b5c60` is a U16 read/store candidate, but its receiver owner and typed join to the StringType member are `UNBOUND`; it is not established as the selected first consumer. For height, the scoped `+0x9e8` use reaches a cleanup-shaped helper, not a token reader. This is not a global no-consumer claim.

No runtime value assignment, numeric mapping/unit, BSI write, decoded PCM effect, or audible effect is established. Runtime is `NOT_RUN`.
