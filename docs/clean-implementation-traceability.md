# Clean implementation traceability

> Date: 2026-09-02
> Source: `D:\\Programs\\jocforge-sanitized-spec-01\\clean-handoff`
> Firewall: only the clean handoff, public specifications, generic mathematics, and the clean repository were used.

The C2 validation closure is recorded in `artifacts/validation-closure/` and
reproduced by `scripts/run-validation-closure.ps1`. The C3 temporal
stabilization is recorded in `artifacts/temporal-stabilization/` and
reproduced by `scripts/run-temporal-stabilization.ps1`. The prior temporal
matrix was insufficient because it counted source membership changes as coded
identity changes; the strengthened metric now reports both.

Disposition counts: `IMPLEMENTED=54`, `ALREADY_SATISFIED=7`, `TEST_ONLY=0`,
`OPEN_RESEARCH=10`, total `71`.

| Requirement | Disposition | Code / decision | Evidence |
| --- | --- | --- | --- |
| PUB-001 | IMPLEMENTED | `writer.rs` public JOC/EMDF serialization | OpenJOC strict inspect |
| PUB-002 | IMPLEMENTED | `profile.rs`, `writer.rs` frame/profile contract | core writer tests; OpenJOC |
| PUB-003 | IMPLEMENTED | `adm.rs` ADM/channel bindings, full-track timing, positions, and semantic LFE classification from `speakerLabel`/`frequency.lowPass` | semantic LFE and fail-closed ADM tests; DirectSpeakers inventory |
| PUB-004 | IMPLEMENTED | CLI rejects `Other`; parser rejects extra blocks, unsupported trajectories, and ambiguous programme membership | explicit parser/CLI errors and rejection tests |
| PUB-005 | IMPLEMENTED | `README.md`, design document | documented idx0/idx1/idx2 and ADM subset |
| PUB-006 | IMPLEMENTED | deterministic pure core and fixed shell ordering | reduction determinism test |
| PIPE-001 | IMPLEMENTED | `main.rs` expands every validated ADM channel binding and routes bounded LFE once to the core | duplicate CHNA rejection tests; Hanabi metrics sidecar and LFE singular-path evidence |
| PIPE-002 | IMPLEMENTED | `reduction.rs` accounting and diagnostics plus explicit LFE records | reduction tests; per-frame CSV |
| PIPE-003 | IMPLEMENTED | writer validates profile dimensions and frame sizes | OpenJOC/FFmpeg validation |
| PIPE-004 | IMPLEMENTED | typed errors at ADM/CHNA, reduction/factorization, and writer boundaries | duplicate/conflicting CHNA rejection tests and invalid input tests |
| PIPE-005 | IMPLEMENTED | CLI stdout and `.metrics.csv` record profile/rate/M/carriers | Hanabi metrics sidecars |
| PIPE-006 | IMPLEMENTED | pure reduction separated from CLI/writer | `reduction.rs` and `main.rs` boundary |
| RED-001 | IMPLEMENTED | `ReductionConfig` validates `1..=15` budget | reduction tests |
| RED-002 | IMPLEMENTED | `MAX_CODED_OBJECTS=15` | 20/28-object encodes |
| RED-003 | IMPLEMENTED | source ordinal and per-frame mapping retained; unsupported non-full timing is rejected before PCM | timing rejection tests; accounting CSV |
| RED-004 | IMPLEMENTED | positive weighted cluster contribution | no-drop accounting test |
| RED-005 | IMPLEMENTED | stable tie-breaking and fixed traversal | deterministic reduction test |
| RED-006 | IMPLEMENTED | signal/spatial diagnostics | `ReductionDiagnostics` |
| RED-007 | IMPLEMENTED | peak-limited cluster signals plus independently recomputed unexplained residual | leakage oracle test; closure threshold |
| RED-008 | IMPLEMENTED | signal/spatial/energy objective | design document and config |
| RED-009 | IMPLEMENTED | previous representative seed reuse and explicit continuity switch cost | `reduce_scene_with_previous`; temporal regression tests |
| CAR-001 | ALREADY_SATISFIED | existing `profile_spec` topology | profile tests; strict inspect |
| CAR-002 | IMPLEMENTED | source-derived 5/7 carrier wrapper | factorization tests; CLI |
| CAR-003 | IMPLEMENTED | reduced signals remain carrier inputs and measured matrix residual is thresholded | factorization residual oracle; closure JSON |
| CAR-004 | IMPLEMENTED | bounded reduction output and shared `0.90` peak normalization for reduced and LFE carriers | LFE peak/headroom sidecar rows and peak-limit unit/integration tests |
| CAR-005 | IMPLEMENTED | profile labels/order remain stable | profile tests and stdout |
| CAR-006 | IMPLEMENTED | stable seeds, continuity cost, and measured carrier/matrix coefficient jumps | sidecar jump metrics; temporal closure |
| CAR-007 | IMPLEMENTED | measurable signal/spatial/leakage/headroom objective | reduction diagnostics |
| CAR-008 | OPEN_RESEARCH | carrier dimension is currently profile-fixed | ORQ-003 remains open |
| MAT-001 | IMPLEMENTED | public coarse 96-symbol Huffman table and quantizer | writer arbitrary-symbol test; full-domain three-profile quantization stress |
| MAT-002 | IMPLEMENTED | bounded carrier scaling and quantized coefficients | factorization wrapper |
| MAT-003 | IMPLEMENTED | ridge-regularized least-squares objective | `analysis.rs` |
| MAT-004 | IMPLEMENTED | closure measures signal, spatial, unexplained leakage, factorization, and speaker-proxy error | quality gates and closure artifacts |
| MAT-005 | IMPLEMENTED | dynamic ridge regularization | near-singular analysis test |
| MAT-006 | IMPLEMENTED | deterministic coefficients with finite element-wise jump measurement | analysis jump test; sidecar and repeated encode |
| MAT-007 | IMPLEMENTED | full matrix dimensions and EMDF payload bounds | OpenJOC strict inspect; full-domain quantization stress |
| MAT-008 | IMPLEMENTED | regularization is part of generalized solver | analysis tests |
| MAT-009 | OPEN_RESEARCH | one-band full-programme baseline | ORQ-004 |
| MAT-010 | OPEN_RESEARCH | full matrix selected; sparse is deferred | ORQ-005 |
| TMP-001 | ALREADY_SATISFIED | CLI frame loop and sequence count state | existing framing path |
| TMP-002 | ALREADY_SATISFIED | 1536-sample public frame loop | OpenJOC AU census |
| TMP-003 | IMPLEMENTED | previous seeds, identity-keyed continuity, and measured matrix/carrier jumps | sparse identity and analysis tests; sidecar |
| TMP-004 | IMPLEMENTED | aggregate position per coded cluster; unsupported input trajectories rejected | OAMD position output; ADM timing/block tests |
| TMP-005 | IMPLEMENTED | `reduction_assignment_changes` and `reduction_identity_changes` with identity-keyed prior state | metric validity and sparse-index tests |
| TMP-006 | IMPLEMENTED | `ceil` frame count, zero-padded short final frame, and repeated CLI-process evidence | `short-final-process.json`; CLI integration |
| TMP-007 | IMPLEMENTED | unchanged source representatives reused between frames | previous-result API |
| TMP-008 | OPEN_RESEARCH | richer interpolation policy remains deferred after membership stabilization | ORQ-006; C3 gate |
| COM-001 | ALREADY_SATISFIED | profile channel maps and public EMDF payload IDs | profile/writer tests |
| COM-002 | IMPLEMENTED | profile-fixed compatibility layouts documented | design and README |
| COM-003 | IMPLEMENTED | OpenJOC speaker render plus quantitative clean speaker projection per profile | rendered WAV evidence; `speaker_render_error` threshold |
| COM-004 | IMPLEMENTED | strict render/decode plus independently measured spectral, band, tilt, transient, phase, coherence, polarity, quadrature, and boundary gates | `spectral-phase.json`; controlled profile OpenJOC/FFmpeg evidence; derived T-008 |
| COM-005 | IMPLEMENTED | compatibility is profile selection, not source mutation | CLI profile path |
| COM-006 | OPEN_RESEARCH | compatibility loss is not yet in reducer objective | ORQ-007 |
| COM-007 | ALREADY_SATISFIED | retained as reference-only observation | clean handoff |
| COM-008 | OPEN_RESEARCH | sequential objective selected for milestone | ORQ-007 |
| RATE-001 | IMPLEMENTED | fixed documented 48 kHz/1024k carrier support | parser and CLI errors |
| RATE-002 | IMPLEMENTED | deterministic I0/D0 budget table and actual-rate enforcement | bitrate sweep; rate tests |
| RATE-003 | IMPLEMENTED | max-15 budget and public payload bounds | writer/reducer checks |
| RATE-004 | IMPLEMENTED | explicit reduction or typed failure | reduction/writer errors |
| RATE-005 | IMPLEMENTED | final coded dynamic-object target never exceeds 15 | Hanabi output |
| RATE-006 | OPEN_RESEARCH | adaptive capacity is deferred | ORQ-002 |
| RATE-007 | ALREADY_SATISFIED | retained as reference-only observation | clean handoff |
| RATE-008 | OPEN_RESEARCH | capacity fitting policy is not yet continuous | ORQ-002 |
| OAMD-001 | IMPLEMENTED | aggregate coded-object membership and position | accounting CSV/OAMD |
| OAMD-002 | IMPLEMENTED | energy-weighted aggregate position for accepted static blocks; unsupported trajectory/timing is rejected | `aggregate_position`; ADM boundary tests |
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
