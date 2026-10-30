# F03C pow32 and F03U source-weight numeric semantics

Disposition: APPROVED_CONTRACT_RESTATEMENT_WITH_INPUT_ONLY_MODEL_VECTORS. This supplement carries the approved mathematical contract and finite set of discriminating test cases. It does not assert product-native parity, full encoder acceptance, or output correctness.

## Bound contract

Approved F03C.1 defines pow32(x,p) as mathematical real x raised to R32(p), then rounded to binary32 using round-to-nearest ties-to-even. The exponent for F03C.3 and F03U.4 is R32(0.2), word 0x3e4ccccd. Each R32 operation is a separate rounding point; signed zero and gradual underflow are retained; F32 routes prohibit FMA/reassociation. Valid feature inputs are finite, nonnegative F32, 24 bands, and 1..48 candidate columns. Negative finite values and nonfinite values are rejected before accepting a result.

F03U.4 operates on the original pre-selector feature snapshot, with candidate IDs in increasing order. For each candidate, start a positive-zero F32 accumulator; visit bands 0..23 and separately round each add to F32. Compute weights for all original candidates, not only selected winners.

The contract specifies the mathematical operation; it does not guarantee every platform libm is bitwise equivalent and adds no general native-vs-model bit-parity acceptance gate.

## Discriminating model vectors

The included vectors are synthetic, input-only binary32 words. They cover:

- 24 repeated copies of 0x3d0ac7c7: normative per-band power 0x3f021640 and serial R32 total 0x41432160. A local Windows UCRT candidate returned 0x3f02163f and total 0x4143215f; this is a host-runtime observation only, not product behavior.
- A fold-order case with powers [0x3f800000, 0x337ffffc, 0x337ffffc]: the required band-order R32 left fold returns 0x3f800000; regrouped or F64 accumulation then cast returns 0x3f800001.
- Maximum finite F32 and minimum positive subnormal: reported powers 0x4c420303 and 0x30930889; the 24-term serial total is 0x4c420303.
- Negative finite, positive infinity, and quiet NaN: reject before power evaluation.
- Negative zero: numerical zero; the approved clause does not separately specify the sign of the power result, so it remains a probe, not a product acceptance condition.

Expected words are model replay for the stated vectors. Rust standalone and UCRT are identified host probes; neither is the target DEE DLL nor the current product source. The reverse package checker recomputed the model values and passed in ordinary and optimized Python. Wash independently verified the source manifest and all member hashes, reran that checker in both modes, and matched its F03C.1/F03U.4 references to the existing approved caller-binding package.

## Next implementation check

Replay the included vectors through the current Rust source_weight entrypoint. Record the operation used for pow32 and the resulting F32 words. Compare against the normative mathematical model and serial R32 totals on these cases. A mismatch blocks that implementation path until corrected; it does not create a requirement for all-platform or all-domain bitwise native parity. Keep the 0/1 caller-order test and these general-value numeric tests separate.

## Not established

The target DLL general-value behavior was not run. The current product source was not inspected by the reverse researcher. No selector/gain/projection/codec execution, audio, AU, E-AC-3/JOC output, or playback was tested.