# DEE JOC per-block stride-selector source boundary

## Bounded result

In `dee_audio_filter_ddp_atmos.dll` SHA-256 `3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2`, `FUN_1800a3710` reads a block-local selector at `state + 0x140e8 + block * 10 + 0xd8`. The first four table values at RVA `0x846698` are `[0,1,2,4]`; selector 0 disables the short-delta branch and selectors 1/2/3 select strides 1/2/4. The API accepts 1536-sample blocks or zero for flush.

A narrow Ghidra scan of `FUN_1800942d0`, `FUN_180073ef0`, `FUN_1800b7930`, and `FUN_1800a3710` found no write to the selector field. A different `RSI+0xd8` reference in `FUN_180073ef0` writes to `RSI+0x192d4` and has no dataflow link to the selector. The current 1024 kbps / 5.1 selector source and value remain unbound.

## Implementable rule

Preserve the selector as an explicit per-block input and apply the observed table lookup. Do not derive it from bitrate or channel layout. If the current model lacks a selector producer, use the best-supported block/lane-plan value supplied by the model and verify the resulting JOC payload on authorized inputs; keep the source of the current profile's selector visible rather than inventing a constant.

This is a JOC payload selector boundary. It does not explain E-AC-3 core `exponent -1` parser failure and does not prove the short-value buffer alias or runtime route. The static table/read rule is high confidence; the writer/producer and active profile value are not observed.

## Exclusions

No DEE run, media/PCM read, private-key access, binary, or decompilation is included. The vector is input-only and unexecuted.
