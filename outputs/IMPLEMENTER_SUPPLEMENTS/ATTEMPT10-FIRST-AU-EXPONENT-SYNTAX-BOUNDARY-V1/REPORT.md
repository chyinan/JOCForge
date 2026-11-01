# Attempt-10 first-AU exponent syntax boundary

## Result

The previous hash-bound OpenJOC report for attempt-10 shows a first diagnostic at AU 0: decoded E-AC-3 exponent `-1` is outside the parser's `0..24` range. Source inspection shows the aggregate label `INVALID_JOC_CARRIAGE` can wrap E-AC-3 exponent-validation failures while the parser scans a skip-field carrier; it does not identify an ID14 semantic error by itself.

The exact AU0 lane/block and bit offset are still not bound to the reported file. A source-derived scalar vector demonstrates how one 7-bit grouped-exponent code can produce `-1`: with initial exponent 1, grouped code 12 expands to digits `[0,2,2]`; the first update is `1 + (0 - 2) = -1`. The parser raises before the following 2-bit `gain_range` read. This is a parser-side boundary vector only, not a frame and not proof of attempt-10 causality.

The same source audit found no selected-profile count/order discrepancy between the frozen model and current Rust source: both derive complexity `15 → 16` and write an 8-bit BSI value before block syntax. The current application `encode()` still returns `SourcePipelineUnavailable`, so its source path is not evidence for attempt-10. A cursor shift from BSI/carrier placement remains a possible but unproven inference; direct ID14 body semantics are not supported as the first diagnostic's layer.

## Use

Treat the `-1` path as a precise first-AU syntax-debug hypothesis. Compare the selected Rust writer's exponent fields and preceding cursor against approved DEE and the keyless prototype's non-secret algorithm, then produce a fresh file and test normal parsing. Do not copy OpenJOC as an encoder algorithm. Keep the exact source2/AU0 lane and attempt-10 runtime binding open until a run/output observation establishes them.

No output EC3/WAV bytes, PCM, private key, HMAC secret, or license material were opened or copied into this Wash supplement.
