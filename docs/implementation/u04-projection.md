# F03U U04 projection direct slices

This module provides two stateless F03U primitives over caller-supplied values: the selector-status admission gate and the final U04 membership normalization/projection. The gate calls refinement only for `CAPACITY_REACHED`; `CANDIDATES_EXHAUSTED` preserves initial positions even when the selected count happens to be 15.

The projection consumes source XYZ, selected target XYZ/attributes, source weights, and an explicit `gain[N][T]` matrix. It applies the source-axis reciprocal normalization first, the target-row reciprocal normalization second, then accumulates projected XYZ by source ordinal. Target attributes with low bit 1 preserve the exact initial target XYZ words. It does not compute gain, source weights, geometry, target membership, or an NNLS solution.

The three projection and three admission vectors are selected directly from the hash-pinned unit-vector asset. Full geometry refinement, target installation, U04 producer, parent transaction, real ADM, and product acceptance remain deferred.
