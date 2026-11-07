# Custom 5.1 trim value boundary

Reference-only Wash supplement. It does not implement trim or assign units. `IMPLEMENTATION_READY=NO`.

Both fields are declared as `xs:string`, and their `StringType` constructor members are statically bound with initial value `auto`. The declared non-auto tokens remain lexical strings:

| Field | Tokens | Static candidate |
|---|---|---|
| `surround_trim_5_1` | `0`, `-3`, `-6`, `-9` | `FUN_1800b5c60` reads U16 from `[RDI+0x968]` and stores U16 to `[RCX+0x240]`; `RDI` is not joined to the keyed StringType or selected provider. |
| `height_trim_5_1` | `-3`, `-6`, `-9`, `-12` | No value reader identified by the bounded exact `+0x9e8` operand query. The observed path is initialization/cleanup only. |

Omission materialization, runtime token assignment, numeric conversion, units, precision, selected provider, BSI writer, and audio/PCM effects remain `UNBOUND`/`NOT_RUN`. Do not label the values as dB or reuse another profile's trim behavior. The bounded query does not establish global absence of a reader.