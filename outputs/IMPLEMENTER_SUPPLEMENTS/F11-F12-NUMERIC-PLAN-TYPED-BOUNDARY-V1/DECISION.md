# Typed F11-to-F12 receipt and candidate boundary

This addendum refines the already sealed F11 source 0/1/2 vectors. It does not rehash the 25-source resolution or repeat source-leaf validation.

The approved handoff object is `NumericPlanReceipt`. The implementation label `SelectedSourceNumericPlan` maps to an adapter projection of that receipt; the approved model does not define it as a new persistent owner. Join through `source` and the canonical `SharedAnalysisReceipt` digest, while keeping each owner's U64 before/after generation and hash distinct. Do not collapse the owner counters into a fabricated global generation number.

The three existing synthetic source rows show one complete local source transaction each: the F11 result and F12 best plan are nested in the same row and their plan digests agree. These rows do not serialize a typed `SharedAnalysisReceipt` digest, so the added typed map states the approved contract boundary and retains the existing local-execution grade. Symbolic digest fields are not captured IDs.

F11 runs once before the F12 candidate loop. Candidate q changes only F12 scratch. The saved source rows report 11 trials, zero candidate state commits, one source commit, and complete 4096-byte CRC-valid frames. The synthetic vectors do not include the global durable-append fault path. F11 per-source `H/F/M/residual` scratch remains `NOT_RECORDED` and is not added to the receipt.

Fresh source-to-frame execution, a runtime typed-receipt join, native execution, full-song replay, and product acceptance remain `NOT_RUN`.
