# Six fit-cell boundary vectors

Internal test-only companion to SYNTHETIC-QMF-FIT-CHECKPOINTS-V1. This small input-first set covers tie order, initial eligibility, high-band reselection, signed full fits and first-stage sparse backup. It does not introduce an algorithm or a new owner.

For each case, construct five carrier rows of48 complex slots and64 bins. All imaginary components and all bins other than INPUT.band are +0. Carrier0 real is the supplied amplitude; for carrier c1..4 use amplitude times (-1)^bit(slot,c-1). If duplicate01 is true, carrier1 uses carrier0's values; this is content duplication, not alias ownership. Round components F32. At each slot/bin, build target by starting at +0 and, in channel order0..4, updating value=R32(value+R32(carrier_real*target_mix[channel])). Imaginary target is +0. Weights are explicitly one at this fit-entry boundary.

Use the approved fit/quantizer/score network from TARGET-QMF-FIT-INTEGRATION-V1. Six cases distinguish:

- duplicate_tie: equal initial scores retain the first candidate under strict comparison.
- low_band_eligibility_seed: eligibility gates initial seeds; approved later search can add initially ineligible channels.
- high_band_reselection: band18/mode3 may replace the final single-channel choice while retaining stage0 backup.
- negative_and_backup: retain signed full coefficients; sparse backup is the first-stage result, not the final full coefficient at that channel.
- zero_target_order: first eligible channel wins a tie, not always channel0.
- empty_eligibility: reject; do not invent zero support or a zero fit. The delivered normalized fixture label is not a required native/product error string.

INPUT-ONLY.json and the approved mathematical provider are inputs. EXPECTED.json contains assertion-only Gram/cross, initial candidate solve/normalize/stored/trial/score terms and all model fit stages. Never use expected to select a candidate or drive production. Generic {f64le:...} containers preserve model values; F32 stages retain their exact F32 numerical values inside those containers, and score stages retain F64.

Coordinator verified nine research payload size/hash bindings and independently reran normal and optimized model recomputation: six cases and67 explicit score calls passed in both modes, with expected opened only after computation. The mathematical provider is the existing approved provider, not a new table authorization. No generic libm fallback, native execution or real audio was used. Model recomputation is not independent/native numerical agreement. No research source or import path is delivered. Whole-song acceptance remains separate.
