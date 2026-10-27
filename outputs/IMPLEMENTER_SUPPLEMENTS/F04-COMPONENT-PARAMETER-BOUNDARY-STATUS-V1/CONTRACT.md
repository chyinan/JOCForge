# F04P / F04PG component-parameter boundary status

Classification: HASH_BOUND_STATUS_CROSSWALK. The selected F04P/F04PG formulas, ranges, cold state, recurrence and stage boundary are already defined in the approved package. This supplement records what is closed and the exact remaining identity/storage gaps; it does not add a new DSP rule or claim native execution.

## Closed model rules

The selected profile uses the approved fixed 48 kHz / 64-sample parameter generator, eight Q15 parameters and one shared parameter state over 15 component rows. Exact formulas, Q15 integers, binary32 projections, F04P recurrence, finite/range checks, cold values and full-reset behavior remain defined by the hash-bound F04PG/F04P specifications listed in SOURCE-CHAIN.json. Parameters are fixed for the selected profile, not generated per object or per row.

Each callback folds 24 ordered 64-sample block transitions into one complete ComponentStageProposal. The per-block F04P.4 commit is the local state fold for the next block; only the existing parent callback transaction installs the proposal. F04A.8 consumes the declared old/new fields in the approved order. This is a clean-model translation of the stage map, not a DEE-native transaction claim.

The relocated frozen Python model was rehashed at 6,733 bytes. It declares hold and blocks_processed as Python int. Its model validation bounds hold to 0..=0x7fffffff and requires blocks_processed >= 0; it increments blocks_processed once per 64-sample transition. The model constructor accepts only the scalar tuple sample_rate=48000, row_count=15, block_count=64. It does not accept an admitted configuration object or carry an install-call token. Python int semantics are not evidence of native width, signedness, overflow or ownership.

## Exact open items

1. Native owner/type for hold: UNBOUND. The model numeric bound does not establish signedness or native storage width.
2. Native owner/type for blocks_processed: UNBOUND. The model has no upper bound/overflow rule; do not infer U32/U64 from a Python counter or from runtime practicality.
3. Admitted profile instance to parameter-install owner/call identity: UNBOUND. The model constructor gate proves only its scalar selected tuple, not the native admitted-profile call chain.
4. Matching native DLL image: NOT_AVAILABLE_IN_RELOCATED_RESEARCH_ROOT on Wash independent file enumeration. No native disassembly or execution is claimed. Keep the three open items visible; do not silently choose native representations.

The input vectors are inputs only and have no expected outputs. They contain no audio, source code, disassembly or private key. The approved formula/provider rules remain usable independently; this status does not establish full F04 receiver execution or product acceptance.
