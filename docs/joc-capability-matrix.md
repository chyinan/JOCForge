# JOC capability matrix

> Generated on 2026-09-09 by the central JOCForge composition resolver. The complete lattice is in the resolver-generated CSV/JSON artifacts.

## Central authority

- CompositionRequest -> CompositionResolver -> JocEncodingPlan | CapabilityClassification is the only semantic classifier.
- composition_capability_matrix() classifies profile, container, carriage, EMDF, partition, dependent count, LFE ownership, and bitrate without media encoding.
- This compatibility script preserves the historical artifact filenames by copying the same resolver output; it contains no independent legality or capacity rules.

## Profile summary

| Profile | Index | SUPPORTED_PASS cells |
|---|---|---:|
| idx0 | idx0 | 445 |
| idx1 | idx1 | 525 |
| idx2 | idx2 | 493 |
| idx3 | idx3 | 437 |
| idx4 | idx4 | 493 |

## Status counts

| Status | Cells |
|---|---:|
| SUPPORTED_PASS | 2393 |
| PUBLIC_AMBIGUOUS | 2990 |
| NOT_APPLICABLE | 110825 |
| SEMANTIC_CEILING | 6240 |
| REJECTED_CAPACITY | 10152 |
| INVALID | 7800 |
| RESERVED | 0 |
| SPEC_LEGAL_NOT_IMPLEMENTED | 0 |

## Evidence and boundaries

- PUBLIC_AMBIGUOUS is reserved for mixed short partitions whose programme grouping and ownership are not publicly closed.
- NOT_APPLICABLE keeps CMAF six-block I0 plus optional D0 and fixed legacy AC-3 framing separate from raw composition.
- SEMANTIC_CEILING is distinct from capacity rejection: raw D6/D7 IDs exist, but the canonical Table-47 ownership graph has no additional non-duplicated slot.
- SPEC_LEGAL_NOT_IMPLEMENTED is zero in the regenerated matrix; any future nonzero row is a release-blocking implementation or audit gap.
- FFmpeg interoperability is metadata, not normative classification.

See docs/orthogonal-composition-closure.md for the human-scale composition rules and targeted validation evidence.

