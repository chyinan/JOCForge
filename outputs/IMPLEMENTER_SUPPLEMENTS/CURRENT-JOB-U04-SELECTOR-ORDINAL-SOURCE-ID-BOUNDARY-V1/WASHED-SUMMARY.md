# U04 selector ordinal and source identity boundary

Disposition: EVIDENCE_ONLY_PARTIAL. This package adds no production rule, algorithm, expected output, or implementation approval. It supplements the already approved F03U source-weight caller contract and the separate U04 geometry-projection evidence.

## Source binding

Source package: spatial-selector-identity-wash-v1.
Source manifest SHA-256: 31b7e8d9e1d43f6a2f61f0171a9e99c73d41914d9b79418f69764cee7fbff1ff.
Source EVIDENCE.json SHA-256: 3fe9e525cd1e8c768af1facdf6120f0cae370095e3a0fd51c0931c47c1267802.
Base PreparedJobInput XML identity: 20669e276ac7b708b0a03b127d3f37331c8cbc5cfce9a0f4a2498739ecc36cb9.
The attempted execution XML is a distinct identity because it changes schema version and temporary-directory declarations. The evidence reports codec and loudness values preserved, but that attempt exited before measurement and produced no reference output.

The source manifest and all three payload hashes were independently checked. The supplied checker passed with ordinary Python and python -O. No raw feature payload, audio, source listing, unredacted report, address, binary path, license, or key is included here.

## What is supported

A prior same-job native wrapper entry observed candidate_count=48, target_count=15, and a band-major 24x48 F32 feature matrix with column stride 48. Static producer analysis binds the local candidate loop ordinal to its feature-matrix column.

A bounded static source-record path copies and transforms per-candidate geometry/control rows between distinct record layouts. It did not establish any field in those records as a global ADM source ID. The prior same-generation U04 observation independently binds the 20x7 helper input columns to geometry-point ordinals 0..6 and layout-mask bits [0,1,2,4,5,6,7]. It does not connect those columns to the 24xN source-weight columns.

Therefore preserve all these as separate identity domains until an explicit typed join is observed: ADM global source ID, feature-matrix column, local candidate ordinal, selected ordinal, geometry-point ordinal, layout-mask bit, U04 input column, and 24xN source-weight column. Do not join them by equal values, shape, order, or numeric coincidence.

## Unresolved and next action

The selector gate entry and actual selected-ordinal list were not captured. The continuation attempt failed before measurement during audio component initialization; it is not a native gate observation. Model replay, native gate comparison, Type10, six-plane validation, and Gate1 remain NOT_RUN or NOT_EVALUATED; Gate1 is not closed.

Continue static analysis from the feature descriptor owner to the source identity producer and its same-record consumer. Separately restore the known native launch context and capture one gate input plus selected ordinals and their downstream consumer. If the launch still fails, preserve that failure and continue the static typed-identity trace; do not promote the failed attempt to evidence of a gate result.