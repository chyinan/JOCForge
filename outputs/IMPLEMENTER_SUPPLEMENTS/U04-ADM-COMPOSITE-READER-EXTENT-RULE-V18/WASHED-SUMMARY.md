# ADM composite source duration to reader extent rule

Disposition: SINGLE-ITEM DURATION RULE BOUND; ABSOLUTE CLOCK AND MULTI-ITEM DYNAMIC JOIN OPEN. This washes the v18 static composite-builder rule and separate run38/run39 observations.

## Provenance

Source v18 manifest SHA-256: bacae28bb9f0586656b5a5c842c12672e5f451396510a7fa3c1001ce060b5248.
Rule payload SHA-256: 2aa293fde05a320c6657982f5e92d6210041b38c9142504a5418b7e7341b2896.
Source checker passed normal and optimized Python; Wash rechecked all manifest members.

## Bound arithmetic and observations

Static per-source rule:
- duration_i = end_sample_i - start_sample_i
- composite_start_i = prior_composite_end + prepend_i
- composite_end_i = prior_composite_end + prepend_i + duration_i + append_i
- composite_duration = composite_end - composite_start

Run38 observed one 64-byte source record with prepend=0, start=4320000, end=4418000, append=0. Its main reader extent writer stored 98000. Run39 separately observed composite accessor fields start=0/end=98000 and result=98000. Both are reference runs using the authorized input and produced the known 262144-byte DEE reference output. The static writer and accessor rules connect these observations within their respective runs; run38 and run39 are not one runtime generation.

The prior 6522906 value is a different run37 per-source opened-storage sample-count getter used for end-sample validation; it is not the reader extent or a prior value of the run38 object.

## Remaining clock and input boundaries

For the captured one-item case, the composite duration to main-reader extent store is bound. The exact units or formula inside the accessor beyond end-start are not claimed; multi-item dynamic accumulation has not been exercised. The numerical equality end-start=98000 does not establish the typed mapping from record coordinates to absolute ADM sample position. Reader-relative cursor to ADM sample or host-frame time remains UNBOUND.

No relation is established to CHNA UID, physical track, global source ID, selector column, U04, fit generation, or spatial callback. Model comparison and product output remain NOT_RUN.
