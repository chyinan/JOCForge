# F03N rectangular NNLS solver

This direct slice implements the stateless F03N initial nonnegative least-squares solve for a 15-row, 1..=15-column F32 matrix and a 15-element F32 response. Inputs are U32 F32 bit words in row-major shape. Each finite F32 value is converted exactly to F64; the solver starts cold on every call and returns the full solution, R32 seed words, active-set permutation/diagnostics, dual values, residual norm, status, and the 3n secondary-iteration bound.

The implementation follows the supplied Householder, relative-rank, positivity, triangular-solve, feasibility-step, repeated last-pop, Givens, and iteration-limit rules. It does not call a generic NNLS library. F64 operations are staged separately; the final F64-to-F32 seed conversion uses explicit nearest-even bit rounding.

Nine actual F03N source-vector cases are tested: `U-OUT-NNLS-1`, `-3`, `-15`, `-REMOVE`, `-FRACTIONAL`, `-RANK`, `-NONPOSITIVE-TRIAL`, `-LAST-POP`, and `-ITERATION-LIMIT`. The adapter consumes only those selected F03N entries from the hash-pinned unit-vector asset; expected outputs never seed the solver.

The n=0 outer skip remains the caller's responsibility. F03S energy/correction/post-processing, selector, U04, context/response production, warm state, parent transaction, real ADM, and product acceptance remain separate and deferred.
