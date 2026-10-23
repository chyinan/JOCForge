# F03S.3 geometric correction primitive

This direct slice consumes `target_xyz_u32[n][3]`, `source_xyz_u32[3]`, a scalar flag, and one U32 index-state value per target. It returns the F32 `L` matrix, symmetric system `H`, and nonnegative correction vector. All dot products, distances, penalties, matrix construction, Cholesky, forward/back substitution, and correction clamps use separate R32 operations in the order specified by F03S.3.

The local tests match both source vectors, including a one-target system and a two-target nonsymmetric-left / flag-mismatch / forward-backsolve case. Empty/mismatched shapes and non-finite words reject; singular or zero-diagonal systems return numeric failure. The component owns no state or clock.

This does not implement F03S.4 active energy update, F03N solver orchestration, outer source selection, geometry/response producers, selector/U04, parent transaction, real ADM, or product acceptance.
