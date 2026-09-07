# Raw multi-dependent semantic ceiling

> Generated on 2026-09-08 from ETSI TS 102 366 V1.4.1 E.1.3.1.2/E.1.3.1.7鈥?.8/E.2.8.2 and TS 103 420 V1.2.1 Table 47/48.

`SEMANTIC_CEILING` means the public D6/D7 ID is syntactically available, but the selected canonical Table-47 JOC ownership graph has no additional non-empty, non-duplicated dependent slot. It is distinct from `REJECTED_CAPACITY` and `SPEC_LEGAL_NOT_IMPLEMENTED`.

The canonical maximum is D0..D4 for idx0/idx3 and D0..D5 for idx1/idx2/idx4. One dependent LFE is a replacement or supplement on an existing full-band dependent and does not create an LFE-only slot.

## Ceiling rows

| Profile | LFE state | Topology | Cells | Verdict |
|---|---|---|---:|---|
| 5.X | absent | D0..D5 | 21 | SEMANTIC_CEILING |
| 5.X | absent | D0..D6 | 21 | SEMANTIC_CEILING |
| 5.X | absent | D0..D7 | 21 | SEMANTIC_CEILING |
| 5.X | independent | D0..D5 | 21 | SEMANTIC_CEILING |
| 5.X | independent | D0..D6 | 21 | SEMANTIC_CEILING |
| 5.X | independent | D0..D7 | 21 | SEMANTIC_CEILING |
| 5.X | dependent | D0..D5 | 21 | SEMANTIC_CEILING |
| 5.X | dependent | D0..D6 | 21 | SEMANTIC_CEILING |
| 5.X | dependent | D0..D7 | 21 | SEMANTIC_CEILING |
| Flat-7.X | absent | D0..D6 | 21 | SEMANTIC_CEILING |
| Flat-7.X | absent | D0..D7 | 21 | SEMANTIC_CEILING |
| Flat-7.X | independent | D0..D6 | 21 | SEMANTIC_CEILING |
| Flat-7.X | independent | D0..D7 | 21 | SEMANTIC_CEILING |
| Flat-7.X | dependent | D0..D6 | 21 | SEMANTIC_CEILING |
| Flat-7.X | dependent | D0..D7 | 21 | SEMANTIC_CEILING |
| 5.X+2 | absent | D0..D6 | 21 | SEMANTIC_CEILING |
| 5.X+2 | absent | D0..D7 | 21 | SEMANTIC_CEILING |
| 5.X+2 | independent | D0..D6 | 21 | SEMANTIC_CEILING |
| 5.X+2 | independent | D0..D7 | 21 | SEMANTIC_CEILING |
| 5.X+2 | dependent | D0..D6 | 21 | SEMANTIC_CEILING |
| 5.X+2 | dependent | D0..D7 | 21 | SEMANTIC_CEILING |
| 5.X Phase | absent | D0..D5 | 21 | SEMANTIC_CEILING |
| 5.X Phase | absent | D0..D6 | 21 | SEMANTIC_CEILING |
| 5.X Phase | absent | D0..D7 | 21 | SEMANTIC_CEILING |
| 5.X Phase | independent | D0..D5 | 21 | SEMANTIC_CEILING |
| 5.X Phase | independent | D0..D6 | 21 | SEMANTIC_CEILING |
| 5.X Phase | independent | D0..D7 | 21 | SEMANTIC_CEILING |
| 5.X Phase | dependent | D0..D5 | 21 | SEMANTIC_CEILING |
| 5.X Phase | dependent | D0..D6 | 21 | SEMANTIC_CEILING |
| 5.X Phase | dependent | D0..D7 | 21 | SEMANTIC_CEILING |
| 5.X+2 Phase | absent | D0..D6 | 21 | SEMANTIC_CEILING |
| 5.X+2 Phase | absent | D0..D7 | 21 | SEMANTIC_CEILING |
| 5.X+2 Phase | independent | D0..D6 | 21 | SEMANTIC_CEILING |
| 5.X+2 Phase | independent | D0..D7 | 21 | SEMANTIC_CEILING |
| 5.X+2 Phase | dependent | D0..D6 | 21 | SEMANTIC_CEILING |
| 5.X+2 Phase | dependent | D0..D7 | 21 | SEMANTIC_CEILING |

Total semantic-ceiling cells: 756.

See `artifacts/multi-dependent-semantic-ceiling.csv` and `.json` for the complete cross-product rows; the general matrix remains in `artifacts/joc-capability-matrix.csv` and `.json`.

