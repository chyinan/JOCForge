# F03S.2 outer energy/fallback slice

This direct function consumes an explicit 15×n F32 context matrix, 15-value response, and n-value seed. It computes the F32 projection, context/response energies, energy threshold, branch decision, and the fallback scale diagnostic using the selected F03S.2 constants.

When the active condition is true, this slice returns the original seed unchanged as a diagnostic boundary; it does not run F03S.3 geometry or F03S.4 active correction. When fallback is selected, it scales each seed value with a separate R32 multiplication and passes the result through the already-materialized F03S.5 floor/dominance postprocess.

The three selected source cases cover zero projection, an equal seed/context match that falls back, and exact energy-threshold equality that is active. Empty shape, non-finite input, and a non-finite R32 frontier reject without a result. F03S producers, NNLS selection, geometry/correction, U04, source state, parent transaction, and full ADM acceptance remain outside this direct slice.
