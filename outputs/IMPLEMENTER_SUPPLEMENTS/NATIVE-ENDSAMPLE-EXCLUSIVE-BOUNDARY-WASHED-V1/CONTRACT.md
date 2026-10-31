# Native EndSample exclusive-boundary semantics

## Washed scope

This conclusion is limited to the hash-identified R121 native reader path. It does not establish behavior for every producer, job mode, or the full ADM-to-E-AC-3 JOC chain.

The hash-bound R121 evidence reports that the composite range fields map to StartSample and EndSample, in sample-boundary units rather than byte offsets. In this path:

- EndSample equal to zero is replaced by AtmosStorage.sample_count.
- EndSample greater than sample_count is rejected; equality is accepted.
- StartSample greater than EndSample is rejected.
- The reader loop consumes data only while reader_cursor is less than reader_extent.

The same-run R121 observation reports StartSample=0, EndSample=49999, reader_extent=49999, and 49,999 sample frames read per channel (32 * 1,536 + 847). The final callback remains 1,536 frames wide: 847 samples are read and 689 positions remain zero-padded. This runtime count is corroborative; the exclusive-boundary contract comes from the reported consumer checks and cursor comparison.

## Limits and open joins

The embedded-timecode-to-EndSample conversion still has a one-sample discrepancy: R121 reports 49,999 while the earlier frame-token interpretation expected 50,000. Exclusive EndSample semantics does not explain that upstream conversion.

The absolute sample-position origin is not established by reader-relative cursor values. R115-to-R121 execution identity/generation and sample-to-spatial-generation remain UNBOUND. Other producers and job modes remain outside this evidence.

No one-sample pair was run. No PCM or EC3 payload is included. The Ghidra report itself is not included in the source bundle and its bytes were not independently rehashed by Wash; see SOURCE-IDENTITIES.json for the recorded-hash discrepancy with the earlier Wash V2 package. Treat the conclusion as evidence-only and do not use it to infer global reachability or change codec algorithms.
