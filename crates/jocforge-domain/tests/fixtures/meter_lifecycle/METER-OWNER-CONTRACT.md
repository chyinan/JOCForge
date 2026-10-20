# Selected meter owner root

## Status

METER_LIFECYCLE_SEAM=READY_CANDIDATE_TEST_ONLY

PRODUCTION_ALLOWED=NO

REAL_ADM_FRONTEND_RENDER=NOT_RUN

REAL_FILTER_TRUE_PEAK_EXECUTION=NOT_RUN

WHOLE_AUDIO_METER_API=BLOCKED

NATIVE_RECEIPT_SCHEMA=NOT_OBSERVED

PRIVATE_KEY_AUTH=EXCLUDED

This root ends at immutable MeasurementReceipt. Main none-DRC is a separate
owner and is not part of the meter committed state.

## Authority and selected profile

The meter owner controls its meter frontend/OAR state, shared DV32 state,
valid-audio/window clock, K-filter/energy state, true/sample peak state,
integrated state, ungated state and immutable programme measurement.

Selected configuration is 48 kHz, 1536-sample callbacks, BS.1770-4,
dialogue intelligence off and speech threshold100. Meter normal callbacks are
followed by exactly three zero-drain callbacks. Complete 1024-sample blocks are
shared by peak and K-energy. Energy records are complete 960-sample windows.
Incomplete EOF tails are not padded into new records.

## Committed state fields and order

The state owner order is:

1. lifecycle: phase, failed, failure_stage and meter_callback;
2. meter_frontend_oar complete callback/cache/target/gain/record state;
3. shared_dv32 callback, eight parameters and history[6][32] F32;
4. window_k_energy startup/pending/processed/ring/window clock, K state
   [6][2][4], six mantissa and six exponent accumulators;
5. true_peak FIR state[6][4628], true maxima[6], sample maxima[6];
6. integrated period/counter/recent values/counts3040/bin means3040 and result;
7. ungated count/mean/exponent/phase/cursor/filled/energy/result;
8. programme_measurement, null until finalize.

Cold values and per-owner identities are pinned to
F06-METER-NONE-DRC-WHOLE-COLD-STATE-IDENTITIES.json. Repeat descriptors must
first expand into full indexed U32 arrays. Canonical state is the JSON object
mapping those eight owner names to expanded values, UTF-8, sorted keys,
comma/colon separators and finite values only. Wrapper metadata is excluded.
SHA-256 covers those exact bytes.

## Stage and clocks

Each meter callback validates callback index, provider identities, complete
input material, shapes and finite domain, then runs:

frontend -> OAR -> DV32 -> valid-audio scheduler -> peaks/K-energy ->
integrated/ungated -> proposal.

Peak and K-energy may run in parallel only on the same 1024-block identity.
Integrated and ungated receive each energy pair in the same order but own
distinct state. A successful callback proposes one complete after-state and
increments meter callback once.

Finalize runs only after the last callback and required drains. It reads true
maxima, sample maxima, integrated, ungated, DI-off constants, selects loudness,
derives dialnorm and proposes one immutable MeasurementReceipt. It performs no
FIR/K-filter flush or new audio work.

## MeasurementReceipt

Fields are programme/config/input identity, terminal meter state SHA and
counts; dialogue, speech percentage, ITU, ungated, true peak, sample peak,
selected loudness, gating kind and dialnorm. Its canonical digest is sorted
compact UTF-8 JSON over all fields except digest itself.

DI-off fixes dialogue=-120 and speech=-1. ITU above -15/16 is selected with
gating kind2; otherwise ungated with kind0. True peak does not select
loudness.

## Commit, fatal, reset and EOF

Production callbacks and finalize calculate on a clone. Success becomes one
owner proposal consumed by the global transaction. Fatal returns no
MeasurementReceipt, leaves numeric owners unchanged, records first failing
stage and retires the attempt. FAILED rejects further callbacks/finalize.

Full reset recreates every meter owner cold and removes the measurement.
Meter EOF uses exactly the configured three zero drains and no private flush.
Finalized meter state rejects more meter callbacks.

## Test-only seam

The seam validates the frozen no-PCM lifecycle trace: four callbacks plus
finalize. It validates ordered events, before/after hashes, callback counts,
owner hashes and final measurement fields. It does not execute ADM parsing,
OAR render, DV32, filters, true peak, integrated or ungated numerics.

Success returns validation receipts with production_allowed=false,
commit_count=0 and state_delta=null. Recorded model commits remain observations.
Wrong clock/count/provider/shape/finite/owner hash/finalize inputs fail.

Provider local reviews remain leaf evidence: F06TP FIR, F06KG K matrix, DV32,
integrated and peak providers are pinned, but they do not prove a complete
meter. Real ADM/render/filter/true-peak execution and production API remain
BLOCKED/NOT_RUN.
