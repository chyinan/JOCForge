# JOC capability matrix

> Generated from the public-spec closure rules on 2026-09-08. The CSV and JSON artifacts enumerate every cross-product cell; this document records the authority and the evidence boundary.

## Profile authority

| Semantic profile | Index | Source-derived inputs | Canonical carriage | Supported cells |
|---|---:|---|---|---:|
| 5.X | idx0 | L,R,C,Ls,Rs | 56 |
| Flat-7.X | idx1 | L,R,C,Ls,Rs,Lrs,Rrs | 62 |
| 5.X+2 | idx2 | L,R,C,Ls,Rs,Tfl,Tfr | 58 |
| 5.X Phase | idx3 | L,R,C,Ls,Rs | 55 |
| 5.X+2 Phase | idx4 | L,R,C,Ls,Rs,Tfl,Tfr | 58 |
| reserved-5 | idx5 |  | 0 |
| reserved-6 | idx6 |  | 0 |
| reserved-7 | idx7 |  | 0 |

idx0–idx4 follow ETSI TS 103 420 V1.2.1 Table 47/48. JOCForge uses Lrs/Rrs as the internal labels for the Table-47 Lb/Rb rear pair. idx3/idx4 are signaling-only phase variants; no extra PCM phase transform is applied.

## Status counts

| Status | Cells |
|---|---:|
| NOT_APPLICABLE_CONTAINER | 75285 |
| PUBLIC_AMBIGUOUS | 53424 |
| REJECTED_CAPACITY | 11586 |
| RESERVED | 90720 |
| SEMANTIC_CEILING | 756 |
| SPEC_LEGAL_NOT_IMPLEMENTED | 9860 |
| SUPPORTED_PASS | 289 |

## Evidence and boundaries

- `SUPPORTED_PASS` is resolved by the central orthogonal composition plan; canonical raw-elementary ETSI representatives are evidenced by `artifacts/multi-dependent-closure/validation.json` and `artifacts/orthogonal-closure/validation.json`.
- `REJECTED_CAPACITY` is the deterministic lower-rate result for the current paired I0/D0 writer budget.
- `PUBLIC_AMBIGUOUS` is reserved for mixed short partitions whose programme grouping/ownership is not publicly closed. Deployed `codecdatae=0` is an independent signaling profile and is not itself a raw-carriage ambiguity.
- `SEMANTIC_CEILING` is a public raw D6/D7 syntax cell beyond the canonical Table-47 ownership maximum: no additional non-empty, non-duplicated dependent slot exists for this plan. It is not an implementation-gap claim. `SPEC_LEGAL_NOT_IMPLEMENTED` remains for legal forms outside the generated writer evidence.
- Dependent LFE is represented by public `lfeon`/Table-E.1.4 LFE ownership, is a replacement of I0 LFE when I0 has LFE and a supplement otherwise, and remains outside the JOC matrix. Canonical dependent-LFE closure is evidenced by `artifacts/multi-dependent-closure/validation.json` and per-fixture ownership sidecars.
- `NOT_APPLICABLE` keeps CMAF's six-block, I0 plus optional D0, and container-specific signaling rules separate from raw elementary-stream support; legacy AC-3 short framing is also not applicable.
- The raw source-derived writer closes 6, 3+3, 2+2+2, and 1+1+1+1+1+1 for I0+D0 and generalized D0..Dn ownership where the central plan and structural writer are applicable. Independent and dependent LFE ownership remain outside the JOC matrix, and deployed signaling remains separate from carriage.
- The advanced CMAF writer closes the no-LFE, strict, one-I0+D0, six-block, 1024/1280/1408/1536/1664 kbps idx0..idx4 cells; remaining CMAF cells stay explicitly unimplemented or out of scope.
- legacy-ac3 is supported for idx1 across 1024/1088/1152/1216/1280 kbps and for idx0..idx4 at 1280 kbps, with strict EMDF, no LFE, six blocks, and I0+D0; other legacy-core cells remain fail-closed.
- idx5–idx7 are `RESERVED` and are never emitted. Raw D1+ cells with normative gates passed report FFmpeg interoperability separately as `UNSUPPORTED_DECODER_FEATURE`; FFmpeg is not a legality gate.

## Public references

- [ETSI TS 103 420 V1.2.1](https://www.etsi.org/deliver/etsi_ts/103400_103499/103420/01.02.01_60/ts_103420v010201p.pdf), Tables 47/48, clauses 8.1–8.3, and Annex E.
- [ETSI TS 102 366 V1.4.1](https://www.etsi.org/deliver/etsi_ts/102300_102399/102366/01.04.01_60/ts_102366v010401p.pdf), Table E.1.3 and clauses E.1.3/E.2.8.
- Local public OpenJOC evidence: D:/Programs/OpenJOC/crates/openjoc-joc/tests/stateful_decoder.rs, D:/Programs/OpenJOC/crates/openjoc-eac3/src/access_unit.rs, and D:/Programs/OpenJOC/docs/site/project/capabilities.md.

