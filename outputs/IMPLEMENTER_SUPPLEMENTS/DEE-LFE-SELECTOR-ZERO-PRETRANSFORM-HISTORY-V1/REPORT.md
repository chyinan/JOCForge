# DEE LFE selector-zero pre-transform and history update

## Static algorithm slice

For the hash-bound DEE Atmos DDP build below, the selector-zero LFE path applies separate main and companion pre-transforms to each 256-sample F32 block, then a 64-iteration selector-zero recurrence. The two 128-F32 history halves are distinct and update in place per slot across blocks. This is a concrete algorithm rule for the captured layout-7 path, not a decoded-output parity result.

The input here is contiguous u[0..255] F32. The sanitized recurrence contract DEE-DDP-RECURRENCE-SELECTOR-SEMANTICS-WASH-V1 binds the preceding gather and selector-zero A/B tables. This supplement adds the L/R pre-transform tables and history equations.

## Main pre-transform

For i=0..63, let l0=L[2i], l1=L[2i+1], r0=R[127-2i], r1=R[126-2i]. Hm is the slot's main 128-F32 history half.

~~~text
W[128+2i]     = u[2i]   * r0 + u[255-2i] * l0
W[129+2i]     = Hm_old[64+i]
W[127-2i]     = u[2i+1] * r1 + u[254-2i] * l1
W[126-2i]     = Hm_old[63-i]
Hm_new[64+i]  = u[255-2i] * r0 - l0 * u[2i]
Hm_new[63-i]  = u[254-2i] * r1 - l1 * u[2i+1]
~~~

## Companion pre-transform

The companion uses its own Hc history half; main output is not fed into companion.

~~~text
W[128+2i]     = Hc_old[64+i]
W[129+2i]     = u[2i]   * r0 - u[255-2i] * l0
W[127-2i]     = Hc_old[63-i]
W[126-2i]     = u[2i+1] * r1 - u[254-2i] * l1
Hc_new[64+i]  = u[255-2i] * r0 + l0 * u[2i]
Hc_new[63-i]  = u[254-2i] * r1 + l1 * u[2i+1]
~~~

Each call writes every W[0..255] position. Across 64 iterations, history positions 64+i and 63-i cover each index 0..127 once. The same slot history is reused across six blocks in the three captured layout-7 observations; W is fully overwritten before selector-zero recurrence.

## Selector-zero recurrence stage

The next stage snapshots v0..v3 = W[4i..4i+3] before storing one four-F32 group. A and B are the 128-entry F32 tables in the already-washed DEE-DDP-RECURRENCE-SELECTOR-SEMANTICS-WASH-V1 package.

~~~text
W[4i]   = (A[2i]   * v0) * (-1) + B[2i]   * v1
W[4i+1] = (A[2i]   * v1) * (+1) + B[2i]   * v0
W[4i+2] = (A[2i+1] * v2) * (-1) + B[2i+1] * v3
W[4i+3] = (A[2i+1] * v3) * (+1) + B[2i+1] * v2
~~~

This covers 256 F32 values in 64 disjoint groups. The prior sanitized contract binds separate MULPS/ADDPS operations for this recurrence, old-value snapshot semantics, no FMA in this helper, and an uncaptured active MXCSR rounding mode. Do not extend that instruction-level claim to the pre-transform or later 16-point stage.

## History initialization and limits

A separate sanitized package binds zero history only when a new record is carved from its zeroed arena. It does not mean history resets per frame. Later call ordering in the three layout-7 captures was not revalidated, so a reused record carries its current history state.

The observations scope lane 5 to slot 5 and six blocks only for three reused layout-7 receipts. This does not generalize channel identity to arbitrary ADM layouts. The following 16-point stage is identified, but its complete equations are outside this supplement. The nonzero-selector branch, conditional 6x6 matrix branch, selected LFE wire-field join, and same-input PCM parity are also outside scope. The matrix branch is a separate static path and its runtime predicate was not captured.

No DEE run or PCM/EC3/key/license read occurred in this slice. This package contains sanitized equations, algorithmic coefficient tables, input-only vectors, and hash-bound evidence references only. It excludes native source, decompilation, scripts, logs, binaries, media, and credentials.
