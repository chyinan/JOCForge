# U04 source_weight: actual caller binding

**Result:** the producer formula is already approved in F03U.4, not an unspecified synthetic caller input. Invoke it on the actual original candidate feature snapshot before projection. Gain-matrix production is outside this supplement.

| Field | Existing rule and actual caller evidence |
|---|---|
| Invocation | F03U.1: only selector CAPACITY_REACHED. The frozen CALLER uses the corresponding success status to assemble U04; EXHAUSTED, even with15 selected, skips. Skip does not create weights or reuse a cache. |
| N and row identity | N=original candidate_count1..48. candidate_ids[j] is increasing unique global source ID0..47; weight[j] belongs to that ID. It is not selected_count15, processing count, physical38 index, local sampler owner or active-target n. |
| Exact features | CALLER gathers original raw current feature48[b][candidate_ids[j]] into band-major24×N before selection. After selector returns it passes that same saved word list to U04. SEL serializes a separate selector descriptor and SCRATCH constructs independent mutable candidate features. Do not substitute residual, pruned or compressed selector scratch. |
| Transport | PACK uses each source's24 original F32 feature values, with exact F32→F64 promotion solely for fixture transport. There is no source_weight prediction/cache input field in this fixture. Product code consumes owned typed values, not research files. |
| Formula/type | For each j independently, start positive-zero F32 and visit bands0..23: power=pow32(F[b][j],0.2), then total=R32(total+power). Output finite nonnegative F32[N]. pow32 and every rounding/operation follow F03C.1; exponent R32(0.2) has bits3e4ccccd. No F64 reduction, reassociation or generic aggregate energy formula. |
| Clock/state | Input is current feature output for callbackg. Those feature values can embody F03F history/carry, but source_weight has no separate history or lag: recompute local scratch on each actual U04 call. It does not read raw PCM history, geometry/width, earlier weights or old target state. Any held ADM source snapshot on drain remains a different clock; current features still come from the current processing callback. |
| Consumer join | Pass weight[j] to the projection source row with the same candidate_ids[j], feature/position row and callback owner. It multiplies the per-source normalized gain as F03U.4 defines. Keep selected target ordering and the two matrix axes separate. No native receipt name or pointer identity is required. |

PYPURE's source-weight function and RSPURE's bounded serial-power primitive corroborate the arithmetic order. Their platform powf instances are not evidence of full-domain equivalence to F03C's mathematical pow32. The rest of the SDK weighted-center algorithm is a different consumer and is excluded. Do not introduce a new platform-powf guarantee or native gate.

Input-only cases distinguish16 candidates from15 selected, an unrelated processing set, original versus mutated scratch, and status skip. A zero feature column is explicitly only a primitive boundary case, since the actual nonnegative candidate gate requires raw band sum>=0.5.

All referenced texts were rehashed. No new native/DLL, numerical solve, gain/projection execution, actual PCM, expected audio or AU body was read/run. This closes the missing caller-domain interpretation with existing rules; it does not certify integrated Rust execution or revalidate U04 mathematics.
