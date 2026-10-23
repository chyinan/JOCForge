# F06I synthetic integrated-meter state slice

This internal-only slice implements one selected F06I state transition for
explicit normalized-energy pairs. It is not a meter frontend, sample scheduler,
complete none-DRC meter, or programme measurement owner.

## State and local event timing

The fixed 48 kHz configuration derives `period=5` from the selected formula.
Cold/reset is the full state: 4×5 recent mantissas and exponents, 3,040
histogram counts and means, absolute count/mean/exponent, result, and the
period/counter fields. The canonical `METER_INTEGRATED_STATE_V1` serialization
is 24,512 bytes in the specified little-endian field order; it is not a native
ABI claim.

Each call consumes one caller-supplied normalized energy pair. The input is
written into recent group 3 before the counter advances. One event runs at
counter 20 and then every five accepted pairs; recent groups move after event
calculation at each period boundary, and the event's counter returns to 15.
The slice owns no F06W sample/window or callback clock and does not synthesize
energy pairs at EOF.

An event performs ordered recent-group merging, selected histogram binning,
global count halving only when the prior bucket count exceeds 32,767, histogram
mean update, the strict absolute-energy gate and continuous absolute mean, then
the relative threshold/result calculation. The event records its bucket,
halving/admission flags, threshold, begin and total. Full after-state hashes
are compared after group movement.

## Verification and exclusions

Ten synthetic sequences (345 input blocks) are recomputed from cold, with only
the explicit `state_patch` and reset indices allowed to alter that seed. All
complete-state SHA-256 values, result words, counters, histogram totals and
event projections are checked. Seven independent F06I.2 bin boundaries are
also checked. The five schedule cases in the same vector file are counted but
not executed; they belong to F06W.

Contract-derived boundary tests also reject negative/nonfinite mantissa,
unsupported input exponents, malformed state period/counter and nonfinite bin
input, while verifying that rejected attempts leave the caller's prior state
digest unchanged.

The 6,794-entry historical real-music energy fixture is not copied, loaded,
used as a provider, or used as a state seed. It remains a separate historical
attestation. F06W scheduling/windowing, F06U getter/fallback, real ADM and
meter frontend, K/FIR/true-peak, whole-meter finalize, parent transaction,
W04, and product acceptance remain deferred or `NOT_RUN`.

The exact source identities and internal-only distribution boundary are in
`integrated-meter-provenance.json`. Do not commit or distribute this slice on
the public branch.
