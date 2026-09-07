# JOC capability matrix

> Generated from the public-spec closure rules on 2026-09-07. The CSV and JSON artifacts enumerate every cross-product cell; this document records the authority and the evidence boundary.

## Profile authority

| Semantic profile | Index | Source-derived inputs | Canonical carriage | Supported cells |
|---|---:|---|---|---:|
| 5.X | idx0 | L,R,C,Ls,Rs | 16 |
| Flat-7.X | idx1 | L,R,C,Ls,Rs,Lrs,Rrs | 20 |
| 5.X+2 | idx2 | L,R,C,Ls,Rs,Tfl,Tfr | 16 |
| 5.X Phase | idx3 | L,R,C,Ls,Rs | 15 |
| 5.X+2 Phase | idx4 | L,R,C,Ls,Rs,Tfl,Tfr | 16 |
| reserved-5 | idx5 |  | 0 |
| reserved-6 | idx6 |  | 0 |
| reserved-7 | idx7 |  | 0 |

idx0鈥搃dx4 follow ETSI TS 103 420 V1.2.1 Table 47/48. JOCForge uses Lrs/Rrs as the internal labels for the Table-47 Lb/Rb rear pair. idx3/idx4 are signaling-only phase variants; no extra PCM phase transform is applied.

## Status counts

| Status | Cells |
|---|---:|
| OUT_OF_SCOPE_CONTAINER | 40110 |
| PUBLIC_AMBIGUOUS | 25536 |
| PUBLIC_LEGAL_NOT_IMPLEMENTED | 504 |
| REJECTED_CAPACITY | 6902 |
| RESERVED | 48384 |
| SPEC_LEGAL_NOT_IMPLEMENTED | 7505 |
| SUPPORTED_PASS | 83 |

## Evidence and boundaries

- `SUPPORTED_PASS` includes the canonical raw-elementary ETSI dependent carriage, strict EMDF, six-block cells evidenced by `artifacts/multi-dependent-closure/validation.json`; the I0+D0 profile closure remains evidenced by `artifacts/profile-closure/`.
- `REJECTED_CAPACITY` is the deterministic lower-rate result for the current paired I0/D0 writer budget.
- `PUBLIC_AMBIGUOUS` covers deployed codecdatae deviation and non-idx0 skip-field carriage where the public documents do not close the JOC-carrier meaning.
- `PUBLIC_LEGAL_NOT_IMPLEMENTED` is a legal raw E-AC-3 dependent count not yet closed by the canonical no-LFE Table-47 JOC ownership plan; it is not an invalidity claim. `SPEC_LEGAL_NOT_IMPLEMENTED` covers other legal forms not yet generated, including source-derived short blocks and LFE-present multi-dependent plans.
- `OUT_OF_SCOPE_CONTAINER` keeps CMAF's six-block, I0 plus optional D0, and container-specific signaling rules separate from raw elementary-stream support.
- The advanced CMAF writer closes the no-LFE, strict, one-I0+D0, six-block, 1024/1280/1408/1536/1664 kbps idx0..idx4 cells; remaining CMAF cells stay explicitly unimplemented or out of scope.
- legacy-ac3 is supported for idx1 across 1024/1088/1152/1216/1280 kbps and for idx0..idx4 at 1280 kbps, with strict EMDF, no LFE, six blocks, and I0+D0; other legacy-core cells remain fail-closed.
- idx5鈥搃dx7 are `RESERVED` and are never emitted. Raw D1+ cells with normative gates passed report FFmpeg interoperability separately as `UNSUPPORTED_DECODER_FEATURE`; FFmpeg is not a legality gate.

## Public references

- [ETSI TS 103 420 V1.2.1](https://www.etsi.org/deliver/etsi_ts/103400_103499/103420/01.02.01_60/ts_103420v010201p.pdf), Tables 47/48, clauses 8.1鈥?.3, and Annex E.
- [ETSI TS 102 366 V1.4.1](https://www.etsi.org/deliver/etsi_ts/102300_102399/102366/01.04.01_60/ts_102366v010401p.pdf), Table E.1.3 and clauses E.1.3/E.2.8.
- Local public OpenJOC evidence: D:/Programs/OpenJOC/crates/openjoc-joc/tests/stateful_decoder.rs, D:/Programs/OpenJOC/crates/openjoc-eac3/src/access_unit.rs, and D:/Programs/OpenJOC/docs/site/project/capabilities.md.

