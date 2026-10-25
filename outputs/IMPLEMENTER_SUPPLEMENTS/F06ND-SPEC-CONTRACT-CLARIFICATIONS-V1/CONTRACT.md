# F06ND approved clean-contract clarifications

This addendum answers interface questions by pointing to the frozen F06ND and F06 whole-composition clauses. It distinguishes explicit clean-model contracts from remaining gaps; it makes no DEE-native parity claim.

## Metadata override FIFO

The approved cold descriptor is four all-zero logical records, `metadata_delay=3072`, `append_slot=3`, and `frames_processed: U64 = 0`. Per source generation, the specified transition shifts `records[1:]` left, fills the final slot with a zero logical record, reads the new `record0`, consumes/resets it to zero, and increments `frames_processed`. The selected submission is opaque category 5, 32-bit, count 1; it is not EMDF ID1 or ID11. The selected category does not write the DRC control FIFO, so the override remains zero; category1 requires a separate authored-control producer and is rejected in this profile.

The source artifacts do not define a field-by-field logical-record schema, byte serialization, or nonzero category-5 payload. `append_slot=3` alone does not specify those missing fields. Keep them `UNBOUND`; do not invent a record layout.

## Source generation, statistics, scheduler, and whole-source commit

F06ND.1 defines source generations `g` as a contiguous sequence starting at zero. Primary and secondary frames, short flags, and programme dialnorm must be from the same `g`/programme. Statistics runs before the scheduler; the scheduler stores a U64 generation, starts at zero, and increments after success. Statistics state is 23 F32 values and does not store a generation. F06WH then carries the none-DRC outputs into the same source's BSI/ID1 path and stages the whole main-source state before its single source commit.

Therefore the clean semantic requirement is a shared caller-owned source generation `g` for both transitions, with scheduler state generation checked against that source progression. The package does not prescribe a Rust receipt/type name or add a persistent statistics generation. In the F06WH composition, statistics and scheduler results belong to the same staged source transaction; no extra independent commit is authorized.

F06ND says successful generation advances by one and maps overflow to `NUMERIC_FAILURE`. It does not specify a native DEE wrap rule. A checked increment returning the clean `NUMERIC_FAILURE` is a local realization; silent wrap is not specified as clean behavior and must not be claimed as native parity.

## Preflight/error order and remaining error outcomes

F06ND.7 explicitly lists the preflight order: required parameter supply/identity; profile/config domain; state type/dimensions; generation agreement; primary/secondary same-generation shape/finite checks; dialnorm domain; then computation. It maps missing parameters to `PARAMETER_UNAVAILABLE`, unsupported domain to `UNSUPPORTED_PROFILE`, shape/type/generation errors to `DRC_INPUT_ERROR`, and intermediate non-finite/overflow to `NUMERIC_FAILURE`.

Preserve that order. The text does not separately specify or vector-test simultaneous-invalid combinations; the ordered checks are the clean contract, while those combined outcomes remain untested. It also does not explicitly map raw primary/secondary NaN/Inf found during finite preflight to one of `DRC_INPUT_ERROR` or `NUMERIC_FAILURE`, nor a supplied-parameter identity mismatch to a distinct code. Keep those two classifications `UNBOUND`; do not infer them from intermediate failures or from other modules' error names.
