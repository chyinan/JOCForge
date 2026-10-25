# F04A membership producer/owner status

This V2 supplement records the newly hash-bound static row-owner and membership paths. It corrects any interpretation that the handler-gate flag is simply the persistent row-state flag or a typed `audio_diffuse_u32` field.

## Two distinct 56-byte row owners

- The handler input table comes from the frame descriptor's handler-gate table field. The selected handler reads a full U32 at relative row offset 4 and proceeds only when that U32 equals zero.
- The persistent/cache row-state table comes from a separate frame-descriptor field. Its rows are 56 bytes and are prepared by Control row state builder/formatter, then consumed by a persistent state dispatcher. The constructor places the two tables at distinct bases; row count and actual extent are not captured. No same-row pointer or same-generation identity joins them.
- In the branch that stages a raw 52-byte Control52 row beginning at temporary offset 4, the handler's U32 load at staged offset 4 covers raw Control52 bytes 0..3: a typed U8 at byte 0 and three bytes whose type/zero-extension is not established. The typed U32 in the raw Control52 row at offset 4 lands at staged offset 8; it is not the U32 consumed by this handler gate. Do not rename the full U32 gate read as `audio_diffuse_u32`.

## Persistent state and activity are separate

The persistent table's own row+4 flag is written by a row-state record formatter, a default/reset path, and a zero-activity update branch. The active dispatcher tests that persistent row+4 value and selects update versus reset. The current value across cache/copy/reuse branches is not observed. A separate row+0 activity result uses a strict `abs(product) > 2^-23` check and is not the handler gate input.

## Native membership dataflow

The handler gate is `full_u32_row_flag == 0` AND a fresh binary32 matrix row sum `>= 0.5`; it short-circuits the sum test for nonzero flags. The sum is accumulated left-to-right over the matrix count. Exactly 0.5 passes; negative values and NaN fail; positive infinity passes; no explicit finite guard is present. Candidate rows then pass through pairwise/state processing; the final U32 selected-row list is formed from nonzero row-status results and reaches native geometry construction and the positive-weight sample accumulator.

This static dataflow does not join the clean F04A membership predicate `audio_diffuse_u32 != 0 AND lane_i32 >= 0` to the gate flag, matrix sum, or selected-row list. It also does not prove that the accumulator is the clean F04A prefix writer. Preserve the native-to-clean membership and writer identities as `UNBOUND/NOT_PROVEN`.

No native run was performed. Live cache branch, current row values, row generation, and same-generation receipts remain unobserved.
