# Selected coefficient search and generated score

The score provider validates the approved functional and generated-math byte identities before parsing four near-one coefficients, 1159 F64 table words and ten split scalars. The score kernel implements the selected near-one/general log branches and fixed-base10 power branch with explicit FMA sites and ordinary F64 operation grouping. It uses no platform log/pow fallback.

`search_selected_coefficient_cell` enumerates eligible single-channel stage0 candidates in channel order. Low bands add one missing channel per later stage; high bands perform the approved all-five single-channel reselection at stage1 and stop. Candidate and stage replacement use strict `<`, so equal values retain the first candidate. The F32 28/25 penalty is promoted to F64 and multiplied once per iteration. Sparse backup always retains the stage0 winner.

The six SYNTHETIC-FIT-CELL-BOUNDARIES-V1 cases execute 67 score calls. Local tests compare every stage's candidate count, winner, chosen support, raw score, penalized score, final stage, stored full row and stage0 sparse backup. They cover duplicate-score tie order, initially ineligible later additions, high-band reselection, negative coefficients, zero-target order and empty eligibility rejection.

Geometry eligibility and full 15x12 assembly are not yet connected. Therefore this result is a verified cell-search kernel, not a complete J05 coefficient producer or writer transaction.
