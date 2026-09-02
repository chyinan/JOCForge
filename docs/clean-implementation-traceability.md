# Clean implementation traceability

> Date: 2026-09-02
> Source: `D:\\Programs\\jocforge-sanitized-spec-01\\clean-handoff`
> Firewall: only the clean handoff, public specifications, generic mathematics, and the clean repository were used.

Disposition counts: `IMPLEMENTED=50`, `ALREADY_SATISFIED=7`, `TEST_ONLY=4`,
`OPEN_RESEARCH=10`, total `71`.

| Requirement | Disposition | Code / decision | Evidence |
| --- | --- | --- | --- |
| PUB-001 | IMPLEMENTED | `writer.rs` public JOC/EMDF serialization | OpenJOC strict inspect |
| PUB-002 | IMPLEMENTED | `profile.rs`, `writer.rs` frame/profile contract | core writer tests; OpenJOC |
| PUB-003 | IMPLEMENTED | `adm.rs` narrow ADM/BW64 subset | ADM parser tests; documented subset |
| PUB-004 | IMPLEMENTED | CLI rejects `Other`; DirectSpeakers policy is explicit | CLI path and error contract |
| PUB-005 | IMPLEMENTED | `README.md`, design document | documented idx0/idx1/idx2 and ADM subset |
| PUB-006 | IMPLEMENTED | deterministic pure core and fixed shell ordering | reduction determinism test |
| PIPE-001 | IMPLEMENTED | `main.rs` includes Objects and DirectSpeakers sources | metrics sidecar has every source |
| PIPE-002 | IMPLEMENTED | `reduction.rs` accounting and diagnostics | reduction tests; per-frame CSV |
| PIPE-003 | IMPLEMENTED | writer validates profile dimensions and frame sizes | OpenJOC/FFmpeg validation |
| PIPE-004 | IMPLEMENTED | typed errors at reduction/factorization/writer boundaries | invalid input tests |
| PIPE-005 | IMPLEMENTED | CLI stdout and `.metrics.csv` record profile/rate/M/carriers | Hanabi metrics sidecars |
| PIPE-006 | IMPLEMENTED | pure reduction separated from CLI/writer | `reduction.rs` and `main.rs` boundary |
| RED-001 | IMPLEMENTED | `ReductionConfig` validates `1..=15` budget | reduction tests |
| RED-002 | IMPLEMENTED | `MAX_CODED_OBJECTS=15` | 20/28-object encodes |
| RED-003 | IMPLEMENTED | source ordinal and per-frame mapping retained | accounting CSV |
| RED-004 | IMPLEMENTED | positive weighted cluster contribution | no-drop accounting test |
| RED-005 | IMPLEMENTED | stable tie-breaking and fixed traversal | deterministic reduction test |
| RED-006 | IMPLEMENTED | signal/spatial diagnostics | `ReductionDiagnostics` |
| RED-007 | IMPLEMENTED | peak-limited cluster signals and headroom metric | reduction test |
| RED-008 | IMPLEMENTED | signal/spatial/energy objective | design document and config |
| RED-009 | IMPLEMENTED | previous representative seed reuse | `reduce_scene_with_previous` |
| CAR-001 | ALREADY_SATISFIED | existing `profile_spec` topology | profile tests; strict inspect |
| CAR-002 | IMPLEMENTED | source-derived 5/7 carrier wrapper | factorization tests; CLI |
| CAR-003 | IMPLEMENTED | reduced signals remain carrier inputs | per-frame factorization |
| CAR-004 | IMPLEMENTED | bounded reduction output and carrier peak normalization | analysis/reduction diagnostics |
| CAR-005 | IMPLEMENTED | profile labels/order remain stable | profile tests and stdout |
| CAR-006 | IMPLEMENTED | frame-local source carrier path plus stable seeds | temporal policy |
| CAR-007 | IMPLEMENTED | measurable signal/spatial/leakage/headroom objective | reduction diagnostics |
| CAR-008 | OPEN_RESEARCH | carrier dimension is currently profile-fixed | ORQ-003 remains open |
| MAT-001 | IMPLEMENTED | public coarse 96-symbol Huffman table and quantizer | writer arbitrary-symbol test; OpenJOC |
| MAT-002 | IMPLEMENTED | bounded carrier scaling and quantized coefficients | factorization wrapper |
| MAT-003 | IMPLEMENTED | ridge-regularized least-squares objective | `analysis.rs` |
| MAT-004 | TEST_ONLY | external render/speaker comparisons | OpenJOC render evidence; numeric budgets remain follow-up |
| MAT-005 | IMPLEMENTED | dynamic ridge regularization | near-singular analysis test |
| MAT-006 | IMPLEMENTED | deterministic per-frame coefficient construction and seed reuse | repeated encode |
| MAT-007 | IMPLEMENTED | full matrix dimensions and EMDF payload bounds | OpenJOC strict inspect |
| MAT-008 | IMPLEMENTED | regularization is part of generalized solver | analysis tests |
| MAT-009 | OPEN_RESEARCH | one-band full-programme baseline | ORQ-004 |
| MAT-010 | OPEN_RESEARCH | full matrix selected; sparse is deferred | ORQ-005 |
| TMP-001 | ALREADY_SATISFIED | CLI frame loop and sequence count state | existing framing path |
| TMP-002 | ALREADY_SATISFIED | 1536-sample public frame loop | OpenJOC AU census |
| TMP-003 | IMPLEMENTED | previous seed representatives and deterministic ties | temporal policy |
| TMP-004 | IMPLEMENTED | aggregate position per coded cluster | OAMD position output |
| TMP-005 | IMPLEMENTED | `reduction_assignment_changes` | core API |
| TMP-006 | IMPLEMENTED | `ceil` frame count with zero-padded short final frame | Hanabi 13,888,071-sample run |
| TMP-007 | IMPLEMENTED | unchanged source representatives reused between frames | previous-result API |
| TMP-008 | OPEN_RESEARCH | richer transition/interpolation policy | ORQ-006 |
| COM-001 | ALREADY_SATISFIED | profile channel maps and public EMDF payload IDs | profile/writer tests |
| COM-002 | IMPLEMENTED | profile-fixed compatibility layouts documented | design and README |
| COM-003 | TEST_ONLY | OpenJOC speaker render is exercised externally | rendered WAV evidence |
| COM-004 | TEST_ONLY | render/decode path available; numeric coloration thresholds deferred | external logs |
| COM-005 | IMPLEMENTED | compatibility is profile selection, not source mutation | CLI profile path |
| COM-006 | OPEN_RESEARCH | compatibility loss is not yet in reducer objective | ORQ-007 |
| COM-007 | ALREADY_SATISFIED | retained as reference-only observation | clean handoff |
| COM-008 | OPEN_RESEARCH | sequential objective selected for milestone | ORQ-007 |
| RATE-001 | IMPLEMENTED | fixed documented 48 kHz/1024k carrier support | parser and CLI errors |
| RATE-002 | TEST_ONLY | total AU budget is enforced; detailed rate sweep remains script work | 1664 kbps outputs |
| RATE-003 | IMPLEMENTED | max-15 budget and public payload bounds | writer/reducer checks |
| RATE-004 | IMPLEMENTED | explicit reduction or typed failure | reduction/writer errors |
| RATE-005 | IMPLEMENTED | final coded dynamic-object target never exceeds 15 | Hanabi output |
| RATE-006 | OPEN_RESEARCH | adaptive capacity is deferred | ORQ-002 |
| RATE-007 | ALREADY_SATISFIED | retained as reference-only observation | clean handoff |
| RATE-008 | OPEN_RESEARCH | capacity fitting policy is not yet continuous | ORQ-002 |
| OAMD-001 | IMPLEMENTED | aggregate coded-object membership and position | accounting CSV/OAMD |
| OAMD-002 | IMPLEMENTED | energy-weighted aggregate position | `aggregate_position` |
| OAMD-003 | IMPLEMENTED | no dominant-source metadata substitution | reduction result |
| OAMD-004 | ALREADY_SATISFIED | existing public OAMD syntax writer | OpenJOC strict |
| OAMD-005 | IMPLEMENTED | deterministic position and seed policy | repeated frames |
| OAMD-006 | IMPLEMENTED | omitted source remains in coded signal and accounting | no-drop test |
| OAMD-007 | OPEN_RESEARCH | aggregate policy selected; alternatives not benchmarked | ORQ-007 |
| OAMD-008 | OPEN_RESEARCH | non-LFE DirectSpeakers/bed expansion remains narrow | explicit CLI policy |

## Clean algorithm summary

The implemented reduction chooses an energy seed and then farthest-point seeds
using normalized signal signatures, spatial distance, and RMS-energy distance.
Every source is assigned to one selected seed with deterministic tie-breaking.
Cluster signals use positive RMS-derived weights and are peak-limited; cluster
positions use the same energy family. The public writer emits the actual active
coded-object count, full one-band matrices, and coarse 96-symbol Huffman
coefficients after quantization. Generalized coefficient analysis uses a
64-sample deterministic view and a project-owned ridge fallback, while carrier
audio uses the complete 1536-sample source window.

This milestone does not claim production perceptual quality, frequency-local
optimization, sparse/full selection parity, or complete non-LFE bed semantics.
