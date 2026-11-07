# Height trim key/query boundary

Reference-only Wash supplement. The registered key and generic query mechanism are separately bound, but there is no evidence the height key is actually queried or returned as the selected field instance. `IMPLEMENTATION_READY=NO`.

`FUN_180136220` registers `custom_trims:height_trim_5_1`, builds the descriptor in `FUN_18012beb0`, and initializes `StringType` at `param_1+0x9e8` through `FUN_180138690`. Reused DI receipts show `FUN_18014f9f0` can query dynamic linked-record names via `FUN_18014a060` and conditionally return an object on exact key equality.

No typed edge joins the registered height key to the dynamic query name or returned object to the `Priv+0x9e8` StringType / selected provider generation. Do not infer a query by this key, units, dB, BSI mapping, or audio effect.