# U04 geometry projection boundary: washed evidence v1

Status: EVIDENCE_ONLY_PARTIAL. This is a supplemental native observation, not a new production rule and not an implementation approval for this geometry path.

## Source binding

Source package: u04-spatial-input-lineage-wash-v1.
Source package MANIFEST.json SHA-256: 6fedddd07ce459c865fb002666f4329d63d7e02a232bf886d2e44d93a8f34765.
Source EVIDENCE.json SHA-256: 121bfcbb56a5a8c659f2c230797b246c4a3cdd2f896bab8b8f6542fbc760fcb3.
Identity: current PreparedJobInput XML 20669e276ac7b708b0a03b127d3f37331c8cbc5cfce9a0f4a2498739ecc36cb9; normalized config 02c68b5dffddc2557cc6b841268a4d4a0395377c5643607ff1f72b8dd44c24a0; authorized 38-channel BWF 7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1; crosswalk dda4c052c11b47e5f2d4245ae3d9e9736ebb0dc1fb793ebdb5bd0b4cbf7bbac2.

The source package manifest and every listed payload hash were checked. Its verifier passed under ordinary Python and python -O. Raw XML, binary float captures, verifier source, and any native binary/source material are not copied into this washed package.

## What this observation supports

In the same native run, the observed path connects the selected layout mask and geometry owner to interpolation descriptors, a 20x34 geometry matrix row writer, the U04 helper input slice, helper output, and the immediate consumer.

- The layout selected 11 local geometry points. Geometry indices 0..10 correspond to layout-mask bits [0,1,2,4,5,6,7,13,14,21,22].
- The first seven local matrix columns map to geometry points 0..6 and mask bits [0,1,2,4,5,6,7]; descriptor groups were [0,2,1], [3,4], [5,6] and companion groups [7,8], [9,10].
- The 20x34 row writer and helper input have same-run pointer identity. Row stride is 34 F32 values (136 bytes); helper consumption is a 20x7 slice. The maximum observed byte end is 2612 within the 2720-byte caller allocation.
- The first captured helper result is all positive zero. The second has 15 nonzero values; all 140 products are reproduced by the reported per-row F32 scale times input. Its first value reaches the direct consumer; the first accumulator contribution is 0x3f80b49c (about 1.00551176) after a 6.0 transform.

These are geometry/profile-local coordinates and values. They do not identify ADM global source IDs and do not establish the approved F03U band-major 24xN source_weight input. The source package records both joins as UNBOUND and the global IDs as NOT_CAPTURED. Keep the geometry projection and F03U source-weight provider as distinct inputs until an explicit producer and typed identity join is evidenced.

## Corrections and limits

- For the mixed integer/floating-point native call, formal floating argument 3 is in XMM2. The value 20 in R8 was seen only in a separate v10 call-1 entry; it was not captured for the v12 calls.
- A prior helper-return interpretation was corrected: the return is a raw mode echo, not a success status. The v12 return register was not captured.
- The full 20-term reduction was not captured. Full encoding did not complete in v12; model replay, Type10 composition, six-plane comparison, and Gate1 closure were not run.
- This package does not claim production source-weight identity, complete U04 projection, complete encoder output, or product acceptance.

Next useful reverse step: continue from the approved F03U caller contract and locate the actual 24xN source-weight producer/callsite, preserving global candidate IDs through its explicit consumer join. Do not use this geometry helper to fill that gap.