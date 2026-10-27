# F06NR source-frame input and F06ND handoff

Status: `INPUT_AND_TYPED_STATE_SHAPES_READY_NUMERIC_AFTER_STATE_NOT_RUN`.

This package makes deterministic synthetic input recipes and the existing F06NR/F06ND field and clock boundary available to the isolated implementation. It does not provide new numeric transform rules; those remain in the referenced approved specifications.

## Same-source input boundary

For codec source frame `s`, F06NR receives the same-generation F07-delayed primary rows as `F32[6][1536]` in external order `[L,R,C,LFE,Ls,Rs]`, and F07 `short_consumed` flags as `U8[6][5]` in block-major/internal-lane order `[L,C,R,Ls,Rs]`, each flag restricted to 0 or 1. The F07 flags already describe consumed mode; do not apply the transient delay again. Internal transform lanes map to external rows `[0,2,1,4,5]`.

The source clocks are distinct: global callback index `g=s+2`; JOC dispatch index `d=s+1`; F06NR `frame_index` before the call equals `s`; F06ND consumes the same source generation. The mismatch case in `GENERATION-REJECTION-VECTOR.json` is a contract rejection example, not runtime evidence.

## F06NR state and carrier shape

One F06NR owner contains `frame_index: U64`, five independent forward histories `F32[5][128]`, and five independent inverse carries `F32[5][128]`, all positive-zero at cold. The owner stages all five lanes and all six 256-sample blocks. It commits the full after-state once only after every lane/block succeeds; its `frame_index` becomes `s+1`. Numerical forward histories, inverse carries and nonzero carrier words are explicitly `NOT_RUN` in this package.

The producer order and row/flag mapping are in `MODEL-CONNECTIONS.md`. Its F06NI/F06NF/overlap operations produce a six-row F32 carrier `F32[6][1536]`. The whole LFE row 3 and samples 1280..1535 in each row are positive zero. For each of the five internal lanes, block 0 output is discarded after state update; inverse blocks 1..5 populate carrier blocks 0..4. Nonzero values are not included or asserted.

F06ND consumes that same-generation secondary carrier with the same delayed primary, same consumed flags and finalized programme dialnorm. This package does not claim that the full source-to-frame transaction or real carrier has been executed.

## Vector and evidence limits

`INPUT-ONLY-VECTORS.json` defines source 0, 1 and 2 input recipes using exactly representable synthetic F32 values and flags; their sample words are generated in memory, not stored. `SOURCE-VECTOR-HASHES.json` contains SHA-256 values over inputs and flags only. Those hashes do not identify DSP output, state after a transform or PCM from a real ADM file.

Only the package schemas, input hashes, source identities, state shapes, source/generation mapping and specified positive-zero carrier slots were checked for this addendum. F06NR numerical after-state, full nonzero carrier, selected native producer/caller join, real ADM execution, F06ND integrated execution and full-song replay remain `NOT_RUN`.
