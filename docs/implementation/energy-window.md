# K-weighted energy window

Status: `LOCALLY_VERIFIED_COMPLETE_BLOCK_TRANSITION__LIFECYCLE_INPUT_QUEUE_DEFERRED`.

`EnergyWindowState` owns six independent two-stage, four-lane K-filter states, six normalized energy accumulators, six exponents, and the exact sample count retained in the current 960-sample window. Its cold constructor consumes the selected `KMatrixProvider`; state and accumulator invariants are checked before arithmetic.

`stage_energy_window_block` consumes one complete six-row, 1,024-sample block. It splits the block at 960-sample boundaries, retains K-filter state across each split and across windows, accumulates each lane with the approved R32 operations, performs the ordered six-lane reduction, and returns one or two immutable `EnergyRecord` values with the complete next state. It never mutates the supplied before-state.

The arithmetic kernel reproduces all three approved parameterized synthetic sequences: output hashes, two-stage next-state words, and power sums. Separate tests verify `64 -> 128` retained window samples across two complete blocks and fail-closed shape/nonfinite rejection.

The selected real K-matrix provider is used by the public block transition. The three parameterized cases deliberately use their specified synthetic matrix to discriminate arithmetic order; they do not prove complete production K-filter equivalence. This slice does not own the pending six-row PCM queue that assembles complete 1,024-sample blocks, and it does not implement frontend/OAR/DV32 composition, integrated/ungated dispatch, whole meter root, or final commit.
