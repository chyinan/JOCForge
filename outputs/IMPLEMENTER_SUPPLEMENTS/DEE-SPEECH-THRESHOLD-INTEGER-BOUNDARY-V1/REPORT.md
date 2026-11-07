# speech_threshold Integer transfer boundary

Reference-only Wash supplement; it does not establish a metering or audio effect. `IMPLEMENTATION_READY=NO`.

The descriptor registers `loudness:measure_only:speech_threshold` as an Integer with default 15 and inclusive bounds 0..100. Text parsing is base-10 integer parsing; float text rejects. The static setup reads a U32 at `EmpireLoudnessPrv+0x30`, rejects values above 100, and writes the unchanged value to Meter config `+0x48`.

During engine open, `FUN_1801a9620` copies config `+0x48` to engine `+0x48` before calling `FUN_1801af230` with the source config. That callee reads the source value and evaluates unsigned `(value-1U)<100`: 1..100 are copied unchanged to subcontext `+0x16c`; the other branch rewrites source config `+0x48` to literal 15 and does not write `+0x16c`. Because the engine copy precedes this fallback write, the final engine-state value after the branch is unbound.

The threshold-specific XML query owner/key, runtime generation, first post-open algorithm read of engine `+0x48`, and metering/audio effect remain `UNBOUND`/`NOT_RUN`. The reported CLI allowlist of 100 was not independently checked and is not a product-support claim.