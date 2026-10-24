# 未闭合 Functional ID 状态

本报告逐项对照本地实现映射和批准包的完整性矩阵。父 Functional ID 只有明确完成时才算闭合；局部 direct slice 通过，不会替代父 ID 的完成状态。

- Functional ID 数量：104。
- root package MANIFEST SHA-256：`c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a`。
- SPEC-COMPLETENESS-MATRIX SHA-256：`2da96a83e431c33f2d4d1cbe028eff06c1fa29741817ca04bf3fa56da66c3c9e`。
- 本地映射文件：`docs/implementation/IMPLEMENTATION-MAP.json`。
- `测试`列只列出本地 direct slice 的定向测试；矩阵中的 unit-vector ID 是洗白包证据，不表示本地已执行该功能。

## STRC

- 功能范围：`status and identity reconciliation ledger`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`CLOSED_BY_FINAL_LEDGER_REBUILD`；问题：`Q12`；依赖：无。
- 规格身份：`test-vectors/STATUS-RECONCILIATION-LOCAL-REVIEW-STATUS.json` SHA-256 `55c78588b78cb9c579dcd5061368c7dec1fc6a0cd2977b197306a4b313100a0b`。
- 矩阵 aspect 状态：`PARTIAL_OR_NOT_EVALUATED`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`R6_TARGETED_PASS; SR6_LEDGER_001_CLOSED_BY_FINAL_LEDGER_REBUILD; FINAL_MANIFEST_AND_EXPOSURE_ZERO_STALE_ZERO_MISSING`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## GTC

- 功能范围：`selected-profile top-level meter/callback/source/queue/output transaction and fail-stop restart contract`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PASS_STATIC_CONTRACT_G01_G03_CLOSED`；问题：`Q14`；依赖：无。
- 规格身份：`specs/00-global-transaction-contract.md` SHA-256 `5415b7b2fb60f50351cd26691e7ef0fc5bd758e501d305d0a714b90c714cd105`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：`GLOBAL-TRANSACTION-INPUT-VECTORS:14_TOP_LEVEL_CASES+7_ANALYSIS_FAULTS+4_QUEUE_FAULTS+2_DURABLE_FAULTS+3_SCHEMA_REGRESSIONS`, `CROSS-STAGE-CONTRACT-JOIN-VECTORS:J01_J04_9_CASES`。
- 洗白包覆盖说明：`R2_LOCAL_STATIC_PASS; G01_G03_CLOSED_LOCAL_STATIC; G04_G11_WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## XSTJ

- 功能范围：`selected cross-stage carrier/analysis/priming/publication J01-J04 join contract`。
- 本地实现状态：`SCOPED_CLAUSES_LOCALLY_VERIFIED`。
- 洗白规格状态：`PASS_STATIC_CONTRACT_J01_J04`；问题：`Q14`；依赖：无。
- 规格身份：`specs/00-cross-stage-contract-joins.md` SHA-256 `3840bc0cbaa6938c4a129790f39ba103d435fd484866de24a604cb43ff59afdb`。
- 矩阵 aspect 状态：`DEFINED_LOCAL_STATIC_R2_REVIEW_PASS_WHOLE_EXECUTION_NOT_RUN`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：`CROSS-STAGE-CONTRACT-JOIN-VECTORS:J01_J04_9_CASES`。
- 洗白包覆盖说明：`R2_LOCAL_STATIC_PASS_9_CASES_61_ASSERTIONS_3_CONTINUOUS_QUEUE_MAPS; J01_J04_CLOSED_LOCAL_STATIC; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/timeline_clock_contract.rs`。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F01

- 功能范围：`ADM/config`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARTIAL`；问题：`Q01`, `Q02`；依赖：`D11`。
- 规格身份：`specs/01-input-and-clock.md` SHA-256 `6bd78ebe23536c3a55ec54f2e13c2e8b98c2ee245ae5a886e47bc9580520b93a`。
- 矩阵 aspect 状态：`PARTIAL_OR_NOT_EVALUATED`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：`U-RIFF-VALID`, `U-RIFF-TRAILING`, `U-RIFF-ODD-NO-FINAL-PAD`, `U-RIFF-DUP-METADATA`, `U-RIFF-DUP-DATA`, `U-RIFF-BAD-FMT`, `U-RIFF-HOP-OVERFLOW`, `U-RIFF-BOOL-INDEX`, `U-RIFF-RF64`, `U-RIFF-SIZE-MISMATCH`, `U-ADM-TIME`, `U-ADM-GRAPH-VALID`, `U-ADM-GRAPH-MISSING-LINK`, `U-ADM-GRAPH-DUP-TRACK`, `U-ADM-GRAPH-DOCTYPE`。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F01R

- 功能范围：`RIFF/PCM24 reader`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q01`, `Q02`；依赖：无。
- 规格身份：`specs/01-input-and-clock.md` SHA-256 `6bd78ebe23536c3a55ec54f2e13c2e8b98c2ee245ae5a886e47bc9580520b93a`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：`U-RIFF-VALID`, `U-RIFF-TRAILING`, `U-RIFF-ODD-NO-FINAL-PAD`, `U-RIFF-DUP-METADATA`, `U-RIFF-DUP-DATA`, `U-RIFF-BAD-FMT`, `U-RIFF-HOP-OVERFLOW`, `U-RIFF-BOOL-INDEX`, `U-RIFF-RF64`, `U-RIFF-SIZE-MISMATCH`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-io/tests/riff_pcm24.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `selected-riff-pcm24-reader-and-hop`，状态 `LOCALLY_VERIFIED_TEN_MODEL_RIFF_PCM24_CASES__WHOLE_F01_AND_ADM_DEFERRED`：Reads one selected-profile RIFF/WAVE PCM24 byte slice, checks the washed container/chunk/format bounds, preserves chunk order and metadata payload bytes, exposes an optional unique requested metadata payload, and converts one valid hop into 38 planar F32 word rows. It does not perform file I/O, ADM XML/link validation, RF64/BW64 or extensible format, F01G graph resolution, or native malformed-input qualification.
    - 向量说明：All ten `riff-pcm24-read` unit cases are replayed: three accepted files and seven model-reader rejections.
    - 向量说明：Accepted cases compare metadata projection, chunk IDs, data offset, selected payload result, and every requested planar F32 U32 word.
    - 向量说明：Boundary cases cover trailing bytes, an odd final chunk without padding, duplicate requested metadata, duplicate data, bad fmt, hop overrun, RF64 and RIFF size mismatch.
    - 向量说明：The bool first_sample case is rejected at the JSON-to-typed-usize test adapter boundary; the production parser API accepts usize and has no language-level bool input.
    - 向量说明：The package explicitly marks native malformed-input conformance NOT_RUN and axml/chna checks as existence-only, not ADM semantic validation.

## F01G

- 功能范围：`CHNA/AXML/timestamp/object semantics`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARTIAL`；问题：`Q01`, `Q03`；依赖：无。
- 规格身份：`specs/01-adm-graph-and-time.md` SHA-256 `c3abb599305392cdad7b43a1d8d45b1c1ae0466ff899f56caa3590d010a11f14`；`specs/01-bed-routing-and-source-geometry.md` SHA-256 `26e4220818d50227f75d5bc00e99840609d6a2023619e01328cb768bed4511b2`；`specs/01-selected-geometry-layout-and-bed-routing.md` SHA-256 `8ea5ff070ebfb77ecfe86a7e87ce846e05de2a1b3575bf834b5ead7414a51830`。
- 矩阵 aspect 状态：`PARTIAL_OR_NOT_EVALUATED`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：`U-ADM-TIME`, `U-ADM-GRAPH-VALID`, `U-ADM-GRAPH-MISSING-LINK`, `U-ADM-GRAPH-DUP-TRACK`, `U-ADM-GRAPH-DOCTYPE`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/adm_graph.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `selected-real-axml-chna-host-identity-and-source-join`，状态 `LOCALLY_VERIFIED_CANDIDATE_GRAPH_EVENTS_ACTUAL_DBMD_AND_224_BOUNDARY_WORDS__SOURCE_BINDING_UNRESOLVED`：Parses caller-supplied candidate AXML/CHNA identity payloads and the same candidate's actual 580-byte DBMD payload; admits 28 single-block zero-origin object events against N=13,888,071; joins all 38 parser-produced DBMD track records by physical host; and produces a candidate ordered 28-row control52 set. Candidate control52 words match 224 same-package native downstream source-field inputs. It does not read WAV/data/PCM and does not produce complete installed48 numeric state. This source is not proven to be the F01E selected-job source, and no production encode path calls this candidate parser/control API.
    - 向量说明：Candidate AXML/CHNA bytes produce all 38 expected graph identity rows, ten beds and 28 object hosts; candidate object/block timestamps admit 28 single zero-origin Cartesian events covering N=13,888,071.
    - 向量说明：The first native load contributes 38 UID lookups and 38 bed/object descriptor host-key joins.
    - 向量说明：Five UID queries cover repeated hit, missing and empty inputs; `Option` output is a local API projection, not native sentinel modeling.
    - 向量说明：The actual same-candidate DBMD payload is parsed by the existing DBMD parser; all 38 raw fields and enum/flag projections match the hash-bound parser output, then 28 object rows join DBMD fields by physical host.
    - 向量说明：Candidate event→control52 formulas match 224 same-package native downstream control-boundary inputs across eight calls; these are source-consumer inputs, not independent AXML event-producer outputs.
    - 向量说明：F01E's separate validation-only input-adm.wav is 743,696,526 bytes/SHA-256 7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1 with N=6,522,906; this candidate metadata receipt reports hanabi.wav, 1,583,325,474 bytes and N=13,888,071. F01E host10 Z is 0.0 versus candidate AXML Z=0.121. No same-source trim/range binding exists; do not combine these sources.
    - 向量说明：The source receipt confirms zero PCM read. Native malformed XML policy, source-field0, skip+4 transition, prefix geometry and full installed48/control38 remain open.

## F01B

- 功能范围：`bed label routing/prefix layout/fallback/source geometry`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARAMETERIZED_RULES_DEFINED`；问题：`Q01`, `Q03`, `Q14`；依赖：`D10-G`, `D10-LX`, `D10-BP`, `D10-BR`。
- 规格身份：`specs/01-bed-routing-and-source-geometry.md` SHA-256 `26e4220818d50227f75d5bc00e99840609d6a2023619e01328cb768bed4511b2`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PARAMETER_PROVIDER_DELIVERED_LOCAL_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/bed_source_points.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `candidate-normalized-bed-source-points`，状态 `LOCALLY_VERIFIED_NINE_BED_ROLE_POINTS_AND_72_SOURCE_BOUNDARY_VALUES__SOURCE_BINDING_UNRESOLVED`：Generates the nine selected normalized bed-role source points for bed IDs 0,1,2,4,5,6,7,8,9 with exact F32 words and fixed attributes. Bed ID3 is the direct LFE and produces no regular bed point. This candidate value producer is not connected to full source installation or control38.
    - 向量说明：Nine point formulas are keyed by parsed bed_id; direct LFE bed_id3 is omitted from the regular path.
    - 向量说明：Each computed point matches all eight same-candidate source-field boundary inputs for its physical bed host, 72 inputs total; these are downstream consumer inputs, not independent AXML coordinate producer outputs.
    - 向量说明：No source/programme binding to the F01E main job or full source installation is claimed.

## F01GL

- 功能范围：`selected layout expansion/P11/P15/bed semantic routing/normalized points/48-source installation`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARAMETERIZED_RULES_DEFINED`；问题：`Q01`, `Q03`, `Q14`；依赖：`D10-G-FRONT-SIDE-Y`。
- 规格身份：`specs/01-selected-geometry-layout-and-bed-routing.md` SHA-256 `8ea5ff070ebfb77ecfe86a7e87ce846e05de2a1b3575bf834b5ead7414a51830`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PARAMETER_PROVIDER_DELIVERED_LOCAL_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/virtual_prefix_geometry.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `candidate-virtual-prefix-q15-geometry`，状态 `LOCALLY_VERIFIED_11_Q15_POINTS_33_XYZ_WORDS_143_METADATA_WORDS_AND_8_CONSUMER_VALUE_JOINS__SOURCE_BINDING_UNRESOLVED`：Generates only the washed selected eleven-point Q15 recipe, one-time F32 XYZ words, fixed five attribute components, packed attribute and 13 defined metadata words per row. It models neither full layout expansion nor table identity aliasing/runtime writes, and is not connected to a 48-source owner.
    - 向量说明：Eleven candidate Q15 points are recomputed and compared after generation: 33 XYZ words and 143 defined metadata words.
    - 向量说明：Eight washed same-candidate consumer calls match all 264 runtime XYZ words and 440 attribute components by value; this implementation does not claim pointer/table alias or runtime consumer execution.
    - 向量说明：No source binding to the F01E main job, full 48-source state, source field0 or skip-at-field4 transition is claimed.

## F01C

- 功能范围：`head XML/frontend clock/layout normalization subset`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q01`, `Q02`, `Q03`；依赖：无。
- 规格身份：`specs/01-frontend-config-normalization.md` SHA-256 `0dae9b61901e10d002dad26a2addc102f319049900f611722606c7bbc57686b9`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F01E

- 功能范围：`selected fixed job XML/DBMD/bed-channel identity/object event/control52/EOF-reset producer`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q01`, `Q02`, `Q03`；依赖：无。
- 规格身份：`specs/01-frontend-config-dbmd-events.md` SHA-256 `afd9efc879ed9a4878bd967c1370275074efd2226e11e67eb86838f4168f61b7`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-application/tests/prepared_job_input.rs`, `crates/jocforge-domain/tests/job_configuration.rs`, `crates/jocforge-domain/tests/object_control.rs`, `crates/jocforge-domain/tests/primary_metadata_control.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `selected-job-xml-profile-normalization`，状态 `LOCALLY_VERIFIED_11_JOB_XML_CASES__PROFILE_GATE_CONSUMED__GRAPH_AND_EVENTS_OPEN`：Parses the selected UTF-8 job XML, validates the selected WAVE descriptor/source-sample count, normalizes F01E.1 fields, and resolves the result through the existing selected production-profile gate. File paths are returned as values only; this function performs no file I/O. It does not parse ADM/CHNA, DBMD or events, produce control38, or wire encode.
    - 向量说明：Eleven complete job XML cases: ADM and atmos_mezz selection/defaults, ADM precedence, duplicate field, malformed XML, DOCTYPE, namespaced root, missing DRC, nonzero start, alternate rate, and wrong PCM profile.
    - 向量说明：Three accepted cases match the fixed normalized profile and reference values; eight rejected cases match their local CONFIG_* error category with no state commit.
    - 向量说明：The parser input is a real hash-bound selected job XML vector. It is not paired with a synthetic ADM graph and is not claimed as whole F01E or end-to-end execution.
  - `selected-job-resource-and-adm-event-admission`，状态 `LOCALLY_VERIFIED_CONFIG_GRAPH_ADMISSION_AND_IMMUTABLE_RESOURCE_CUSTODY__METERING_AND_MAIN_NOT_CONNECTED`：Coordinates the existing selected job normalization and selected ADM graph event validator before retaining a caller-supplied immutable resource snapshot and its actual SHA-256. It holds the admitted graph/events only after structural admission. It does not parse RIFF itself, run metering, enter READY, execute callbacks or produce output.
    - 向量说明：three accepted and eight rejected selected job configurations retain their existing outcomes
    - 向量说明：accepted preparations contain 28 admitted events and 38 physical tracks
    - 向量说明：a source extent beyond the admitted event coverage rejects before the graph is held
    - 向量说明：the retained Arc bytes share caller storage and their SHA-256 is recomputed from actual bytes
  - `selected-dbmd-authored-mode-and-track-field-producer`，状态 `LOCALLY_VERIFIED_4_AUTHORED_MODES_6_TRACK_FIELDS_10_DBMD_REJECTIONS__F01G_GRAPH_JOIN_OPEN`：Parses an already supplied selected DBMD payload: section framing/checksum/terminator, required section9/section10, authored mode and physical-order section10 per-track fields. The field accessor joins by physical track index, not pack/object ordinal. It does not open the RIFF file, parse F01G ADM graph bytes, attach track fields to host routes, emit control38, or claim whole F01E.
    - 向量说明：Four authored-mode cases verify warm_trim_mode 0..3, authored byte warm+1, and surroundEx only for authored byte 3.
    - 向量说明：Six packed track cases verify low2 flag, enum48 and enum52 extraction/sentinels by physical track index.
    - 向量说明：Ten malformed DBMD cases verify length-header rejection, checksum, duplicate section, truncated header/body, terminator tail, authored mode, section10 magic and track table; errors return no partial parse result.
    - 向量说明：F01G XML/CHNA graph parsing and the physical host join remain separate required edges; this DBMD producer is not whole F01E or an end-to-end path.
  - `selected-object-event-control52-row-production`，状态 `LOCALLY_VERIFIED_28_CONTROL52_VALUE_ROWS__F01E_SOURCE_AND_EVENT_SET_BINDING_OPEN`：Implements the pure F01E.8/.9 row-value formula and tests 28 input rows from the hash-bound F01E vector. The separate ordered-set API requires parser-produced DBMD track fields and the bed/host identity join; its same-source exercise is mapped under the F01G candidate slice and is not source-bound to the F01E main job. It owns no callback hold, reset, commit, or programme state.
    - 向量说明：All 28 actual_fixture_object_controls event-value rows are computed and compared against their 364 expected control52 words; these F01E vectors are not source-bound to the separate F01G hanabi candidate.
    - 向量说明：Three event_cases independently cover diffuse, spread, and sentinel row formulas.
    - 向量说明：Local row-value rejects cover nonzero origin, too-short duration, and invalid host domain.
    - 向量说明：Candidate-source DBMD/host/order/set integration is tested separately under F01G. F01E source binding, callback hold/reset, and control38 atomic publication remain open.
  - `primary-source-bound-control38-value-production`，状态 `LOCALLY_VERIFIED_R4_PRIMARY_METADATA_BOUND_38_ROWS_364_OBJECT_WORDS_BED_TAIL_UNPRODUCED__NO_MAIN_ENCODE`：Binds the Wash R4 path-free job projection, frozen binding and metadata-only source receipt to AXML/CHNA/DBMD hashes, then recomputes 38 physical-host rows, 364 object control words and 38 DBMD rows. Bed bytes 24..51 remain unproduced. Raw job XML is unavailable; full PCM identity is NOT_REHASHED. No application::encode connection or end-to-end run is claimed.
    - 向量说明：38 primary metadata/control rows
    - 向量说明：364 object words compared after production
    - 向量说明：38 DBMD rows compared
    - 向量说明：Bed tail remains unproduced
    - 向量说明：Original job identity RESEARCH_DECLARATION_ONLY; PCM identity NOT_REHASHED; no main encode or native end-to-end claim

## F02

- 功能范围：`sample/callback/EOF`。
- 本地实现状态：`SCOPED_CLAUSES_LOCALLY_VERIFIED`。
- 洗白规格状态：`PARTIAL`；问题：`Q03`, `Q04`；依赖：无。
- 规格身份：`specs/01-input-and-clock.md` SHA-256 `6bd78ebe23536c3a55ec54f2e13c2e8b98c2ee245ae5a886e47bc9580520b93a`。
- 矩阵 aspect 状态：`PARTIAL_OR_NOT_EVALUATED`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：`U-TIME-1`, `U-TIME-1535`, `U-TIME-1536`, `U-TIME-1537`, `U-TIME-6522906`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/timeline_clock_contract.rs`。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F03

- 功能范围：`spatial control`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARTIAL`；问题：`Q03`；依赖：`D10`, `D11`。
- 规格身份：`specs/03-spatial-and-audio.md` SHA-256 `1ae3ecc8058eb06f84e51ddef03e804e0e6f0678ba91df3aa7640f6aa1decbcb`。
- 矩阵 aspect 状态：`PARTIAL_OR_NOT_EVALUATED`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：`U-FRONT-GATE`, `U-FRONT-INSTALL-EMPTY`, `U-FRONT-INSTALL-RECT`, `U-FRONT-INSTALL-TIES`, `U-FRONT-INSTALL-ATTR`, `U-OUT-NNLS-1`, `U-OUT-NNLS-3`, `U-OUT-NNLS-15`, `U-OUT-NNLS-REMOVE`, `U-OUT-ENERGY-ZERO`, `U-OUT-ENERGY-MATCH`, `U-OUT-POST-FLOOR`, `U-OUT-POST-DOMINANCE-EQUAL`, `U-OUT-POST-TIE`, `U-OUT-CORRECTION-ONE`, `U-OUT-NNLS-FRACTIONAL`, `U-OUT-EMPTY`, `U-OUT-ACTIVE-UPDATE`, `U-OUT-NNLS-RANK`, `U-OUT-NNLS-NONPOSITIVE-TRIAL`, `U-OUT-NNLS-LAST-POP`, `U-OUT-NNLS-ITERATION-LIMIT`, `U-OUT-ITERATION-CONSUMED`, `U-OUT-ENERGY-EQUAL`, `U-OUT-CORRECTION-TWO`, `U-OUT-REJECT-NONFINITE`, `U-OUT-REJECT-SINGULAR`, `U-OUT-REJECT-NEGATIVE-D`, `U-SEL-SINGLE`, `U-SEL-COMPLEMENT`, `U-SEL-PREVIOUS-ID`, `U-SEL-CAPACITY`, `U-SEL-CHOICE-BOUNDARY`, `U-SEL-CHOICE-BELOW`, `U-SEL-CHOICE-SECONDARY-TIE`, `U-SEL-AFFINITY-WIDTH`, `U-PROJ-AXES`, `U-PROJ-PROTECTED`, `U-PROJ-ZERO`, `U-PROJ-ADMIT-CAPACITY_REACHED-15`, `U-PROJ-ADMIT-CANDIDATES_EXHAUSTED-3`, `U-PROJ-ADMIT-CANDIDATES_EXHAUSTED-15`, `U-FEAT-FFT-ZERO`, `U-FEAT-FFT-IMPULSE`, `U-FEAT-STATE-CONTINUOUS`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F03G

- 功能范围：`source gate/target installation`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`SELECTED_PROFILE_LOCAL_R1_REVIEW_PASS`；问题：`Q02`, `Q03`；依赖：无。
- 规格身份：`specs/03-gate-and-target-installation.md` SHA-256 `aeb6791967413e71518cfb1c5853c52cd8e6e7e7174f60afa611cc0e0fa8d9dc`。
- 矩阵 aspect 状态：`DEFINED_LOCAL_R1_REVIEW_PASS_WHOLE_EXECUTION_NOT_RUN`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：`U-FRONT-GATE`, `U-FRONT-INSTALL-EMPTY`, `U-FRONT-INSTALL-RECT`, `U-FRONT-INSTALL-TIES`, `U-FRONT-INSTALL-ATTR`。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/selected_target_attribute_relations.rs`, `crates/jocforge-domain/tests/target_slot_install.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `target-slot-install`，状态 `LOCALLY_VERIFIED_F03G_2_4_COST_GREEDY_MAPPING_AND_STATE_PROJECTION`：Explicit prior 15-slot state and selected XYZ/ID/attribute proposal to original row-major cost words, exact n-pass greedy tie order, and complete next-state projection. Empty selection preserves old state. Constructor, source gate, selector, projection/fit producers, marker/history, parent commit, and product integration are excluded.
    - 向量说明：U-FRONT-INSTALL-EMPTY
    - 向量说明：U-FRONT-INSTALL-RECT
    - 向量说明：U-FRONT-INSTALL-TIES
    - 向量说明：U-FRONT-INSTALL-ATTR

## F03C

- 功能范围：`selector affinity/support/coverage/compaction`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q03`, `Q14`；依赖：无。
- 规格身份：`specs/03-selector-affinity-and-selection.md` SHA-256 `ce95853e007f9282f3d1dce0e43cffb132ed079b5f1cd2fe3e6ce10901d51697`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：`U-SEL-SINGLE`, `U-SEL-COMPLEMENT`, `U-SEL-PREVIOUS-ID`, `U-SEL-CAPACITY`, `U-SEL-CHOICE-BOUNDARY`, `U-SEL-CHOICE-BELOW`, `U-SEL-CHOICE-SECONDARY-TIE`, `U-SEL-AFFINITY-WIDTH`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/candidate_selector.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `candidate-selector`，状态 `LOCALLY_VERIFIED_COLD_CANDIDATE_SELECTION__FEATURE_AND_TARGET_PRODUCERS_DEFERRED`：Cold candidate selector over explicit feature/position/response/width/attribute/previous-ID inputs; affinity/complement, fixed Q/T/W, previous-ID boost, slot gate, tie resolution, feature/coverage/support updates, survivor compression, and selected output diagnostics.
    - 向量说明：U-SEL-SINGLE
    - 向量说明：U-SEL-COMPLEMENT
    - 向量说明：U-SEL-PREVIOUS-ID
    - 向量说明：U-SEL-CAPACITY
  - `selector-score-choice`，状态 `LOCALLY_VERIFIED_PRIMARY_WINDOW_AND_SECONDARY_TIE_PRIMITIVE`：Stateless primary maximum, 0.01 F32 tie window, and earliest secondary-maximum selection over explicit same-length primary/secondary score arrays.
    - 向量说明：U-SEL-CHOICE-BOUNDARY
    - 向量说明：U-SEL-CHOICE-BELOW
    - 向量说明：U-SEL-CHOICE-SECONDARY-TIE
  - `selector-pairwise-affinity`，状态 `LOCALLY_VERIFIED_ORDERED_WIDTH_ASYMMETRIC_AFFINITY_PRIMITIVE`：Stateless ordered pairwise affinity from explicit positions, attributes, and widths; preserves candidate-column width asymmetry and selected trigonometric route.
    - 向量说明：U-SEL-AFFINITY-WIDTH

## F03U

- 功能范围：`U04 admission/geometric membership/target-row projection`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARTIAL_DEGENERATE_AND_DOMAIN_CHANGE_FRONTIERS`；问题：`Q03`, `Q14`；依赖：无。
- 规格身份：`specs/03-u04-refinement-and-projection.md` SHA-256 `ca235e2f373d95d5c628ca61c651d01805bb9cb14a7c3457d9a77b1d87941879`。
- 矩阵 aspect 状态：`PARTIAL_OR_NOT_EVALUATED`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：`U-PROJ-AXES`, `U-PROJ-PROTECTED`, `U-PROJ-ZERO`, `U-PROJ-ADMIT-CAPACITY_REACHED-15`, `U-PROJ-ADMIT-CANDIDATES_EXHAUSTED-3`, `U-PROJ-ADMIT-CANDIDATES_EXHAUSTED-15`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/u04_projection.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `u04-membership-projection`，状态 `LOCALLY_VERIFIED_TWO_LAYER_U04_PROJECTION__GEOMETRY_AND_INSTALL_DEFERRED`：Stateless F03U.4/.5 projection from explicit source XYZ/weights, target XYZ/attributes and gain[N][T]; two reciprocal normalization layers, F32 projection, and exact protected-anchor preservation. Gain/weights/geometry are caller-supplied.
    - 向量说明：U-PROJ-AXES
    - 向量说明：U-PROJ-PROTECTED
    - 向量说明：U-PROJ-ZERO
  - `u04-refinement-admission-gate`，状态 `LOCALLY_VERIFIED_SELECTOR_STATUS_ADMISSION_GATE__REFINEMENT_DEFERRED`：Stateless F03U.1 caller gate: only CAPACITY_REACHED triggers refinement; CANDIDATES_EXHAUSTED preserves initial positions even with count 15. Does not run refinement.
    - 向量说明：U-PROJ-ADMIT-CAPACITY_REACHED-15
    - 向量说明：U-PROJ-ADMIT-CANDIDATES_EXHAUSTED-3
    - 向量说明：U-PROJ-ADMIT-CANDIDATES_EXHAUSTED-15

## F03F

- 功能范围：`4096 FFT/feature weighting/grouping/state`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q03`, `Q14`；依赖：无。
- 规格身份：`specs/03-feature-processor-and-state.md` SHA-256 `dbd1cf259d2e40013910868d31e38909678d3af8add9d80bd59d15e8cc6e8edd`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：`U-FEAT-FFT-ZERO`, `U-FEAT-FFT-IMPULSE`, `U-FEAT-STATE-CONTINUOUS`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F03FC

- 功能范围：`feature configuration mathematical generation`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q03`；依赖：无。
- 规格身份：`specs/03-feature-configuration-generation.md` SHA-256 `fe6d6a361d3af53b5289bdafda0a883ac3c5a5e8bd8446797f61e2cd95c6bcc6`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F03R

- 功能范围：`selected source routing/activity/feature flags`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q01`, `Q03`, `Q04`, `Q14`；依赖：无。
- 规格身份：`specs/03-source-routing-and-feature-flags.md` SHA-256 `ef0a7bd95edc2adc63b07e752879b6ddefb817962b7b6fa44c77b4acb5c51f19`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F03P

- 功能范围：`post-audio width-zero point response/target context/math angles`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARAMETERIZED_RULES_DEFINED`；问题：`Q02`, `Q03`, `Q04`, `Q13`, `Q14`；依赖：`D10-P15`。
- 规格身份：`specs/03-point-response-and-target-context.md` SHA-256 `8fb727170fb41df5354a17429fc00411010f010cb32dc86e0fdcc81bddaf4964`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PARAMETER_PROVIDER_DELIVERED_LOCAL_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F03N

- 功能范围：`rectangular current NNLS initial fit`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q03`, `Q14`；依赖：无。
- 规格身份：`specs/03-rectangular-nnls-initial-fit.md` SHA-256 `7cfb51e8905f739e60bebcd30899887bda52cc493a5bab0c976ca645548b09fc`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：`U-OUT-NNLS-1`, `U-OUT-NNLS-3`, `U-OUT-NNLS-15`, `U-OUT-NNLS-REMOVE`, `U-OUT-NNLS-FRACTIONAL`, `U-OUT-NNLS-RANK`, `U-OUT-NNLS-NONPOSITIVE-TRIAL`, `U-OUT-NNLS-LAST-POP`, `U-OUT-NNLS-ITERATION-LIMIT`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/src/spatial/rectangular_nnls/tests.rs`, `crates/jocforge-domain/tests/rectangular_nnls.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `rectangular-nnls-solver`，状态 `LOCALLY_VERIFIED_COLD_RECTANGULAR_NNLS__OUTER_AND_PRODUCERS_DEFERRED`：Stateless 15-row, 1..=15-column F32 matrix plus F32 response to F64 nonnegative solution, R32 seed, active-set diagnostics, and SOLVED/ITERATION_LIMIT status. No F03S outer solver, selector, U04, context/response producer, warm state, parent transaction, or n=0 outer skip integration.
    - 向量说明：9 exact F03N cases: U-OUT-NNLS-1, U-OUT-NNLS-3, U-OUT-NNLS-15, U-OUT-NNLS-REMOVE, U-OUT-NNLS-FRACTIONAL, U-OUT-NNLS-RANK, U-OUT-NNLS-NONPOSITIVE-TRIAL, U-OUT-NNLS-LAST-POP, U-OUT-NNLS-ITERATION-LIMIT
    - 向量说明：Full F64 solution bytes, R32 seed words, passive/inactive/permutation, accepted/rejected/removals order, secondary iteration count, and status

## F03S

- 功能范围：`outer energy/geometric correction/postprocess`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARTIAL_DEGENERATE_FRONTIER`；问题：`Q03`, `Q14`；依赖：无。
- 规格身份：`specs/03-outer-energy-correction.md` SHA-256 `7ca47864c9fe7573b6cc6edfafdbd2049c5e74aa73f9b240c4c15bf4a427cdb2`。
- 矩阵 aspect 状态：`PARTIAL_OR_NOT_EVALUATED`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：`U-OUT-ENERGY-ZERO`, `U-OUT-ENERGY-MATCH`, `U-OUT-POST-FLOOR`, `U-OUT-POST-DOMINANCE-EQUAL`, `U-OUT-POST-TIE`, `U-OUT-CORRECTION-ONE`, `U-OUT-EMPTY`, `U-OUT-ACTIVE-UPDATE`, `U-OUT-ITERATION-CONSUMED`, `U-OUT-ENERGY-EQUAL`, `U-OUT-CORRECTION-TWO`, `U-OUT-REJECT-NONFINITE`, `U-OUT-REJECT-SINGULAR`, `U-OUT-REJECT-NEGATIVE-D`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/outer_active_update.rs`, `crates/jocforge-domain/tests/outer_empty_matrix.rs`, `crates/jocforge-domain/tests/outer_energy_fallback.rs`, `crates/jocforge-domain/tests/outer_geometric_correction.rs`, `crates/jocforge-domain/tests/outer_postprocess.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `outer-coefficient-postprocess`，状态 `LOCALLY_VERIFIED_F03S_5_FLOOR_DOMINANCE_AND_TIE_RULES__OUTER_DEFERRED`：Stateless F03S.5 strict floor, sequential R32 total, dominance comparison, earliest-maximum tie, and one-hot cap. Does not run F03S.1-.4 or any solver/geometry/outer pipeline.
    - 向量说明：U-OUT-POST-FLOOR
    - 向量说明：U-OUT-POST-DOMINANCE-EQUAL
    - 向量说明：U-OUT-POST-TIE
    - 向量说明：empty-vector and non-finite local rejection boundaries
  - `outer-energy-fallback`，状态 `LOCALLY_VERIFIED_F03S_2_BRANCH_AND_FALLBACK_DIAGNOSTICS__ACTIVE_ROUTE_DEFERRED`：Stateless F03S.2 projection/energy branch, threshold and fallback scale. Active cases return the input seed unchanged for branch diagnostics only; fallback scales the seed and calls the existing F03S.5 postprocess. No F03S.3/.4 active correction.
    - 向量说明：U-OUT-ENERGY-ZERO
    - 向量说明：U-OUT-ENERGY-MATCH
    - 向量说明：U-OUT-ENERGY-EQUAL
    - 向量说明：empty-shape/non-finite/numeric-overflow local rejections
  - `outer-geometric-correction`，状态 `LOCALLY_VERIFIED_F03S_3_F32_GEOMETRY_SYSTEM_AND_CORRECTION__ACTIVE_UPDATE_DEFERRED`：Stateless F03S.3 target/source XYZ geometry, distance/flag diagonal penalty, B/L/H construction, lower F32 Cholesky, ordered forward/back substitution, and nonnegative correction. No F03S.4 active update or outer pipeline.
    - 向量说明：U-OUT-CORRECTION-ONE
    - 向量说明：U-OUT-CORRECTION-TWO
    - 向量说明：empty/mismatched shape and non-finite local rejection boundaries
  - `outer-active-update`，状态 `LOCALLY_VERIFIED_F03S_4_DISCRIMINANT_STEP_AND_OUTPUT__OUTER_DEFERRED`：Stateless F03S.4 active quadratic-energy update from explicit context/response/seed/correction inputs. Returns discriminant, step, scaled seed, and output; does not generate correction, choose branch, run NNLS, or apply floor/prune.
    - 向量说明：U-OUT-ACTIVE-UPDATE
    - 向量说明：U-OUT-REJECT-NEGATIVE-D
    - 向量说明：empty-shape/non-finite local rejection boundaries
  - `outer-empty-matrix-skip`，状态 `LOCALLY_VERIFIED_N_ZERO_EMPTY_MATRIX_SKIP__NONEMPTY_OUTER_DEFERRED`：F03S.1 n=0 branch only: validate the explicit source-call order and return a zero-row/48-column matrix without reading response/XYZ or invoking NNLS. Nonempty target matrix production is excluded.
    - 向量说明：U-OUT-EMPTY: source_call_order [0,2], matrix [], shape [0,48], solver_calls 0, empty_targets_skipped true
    - 向量说明：nonempty-target, duplicate-source-call, and out-of-range source-call local rejects

## F03FD

- 功能范围：`U04/outer degenerate-domain fail-stop frontiers and exact-real SPD scope`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q03`, `Q14`；依赖：无。
- 规格身份：`specs/03-degenerate-frontier-and-fail-stop.md` SHA-256 `b657fb3f4b322a41aee8cb7573191d67a56db71ca2d24e27e0b0d4914e3e2c83`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F03W

- 功能范围：`selected ADM/config/events to installed48 whole callback composition/source ownership/stage order/writer-set/drains/reset/fail-stop`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q03`, `Q04`, `Q14`；依赖：无。
- 规格身份：`specs/03-adm-installed48-whole.md` SHA-256 `a494e653e584c8453464aa74ae08bf453cc615e1ccafd98ab4ede238de1f7dd9`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PROVIDER_MANIFEST_DELIVERED_R2_LOCAL_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：`ADM-INSTALLED48-WHOLE-INPUT-VECTORS:R2_7_CALLBACKS+182_RECOMPUTABLE_LEAVES+20_WRITER_ENTRIES+5040_MATRIX_WORDS+112_READER56+672_TARGET_WORDS+10_FAMILY_STATE+NONCOLD_FAILURE_RESET`。
- 洗白包覆盖说明：`R2_LOCAL_7_CALLBACK_SYNTHETIC_NUMERIC_RECIPES_PASS_182_LEAVES_10_STATE_FAMILIES; HISTORICAL_4251_RESUME_ATTESTATION_ONLY; REAL_ADM_FRESH_COLD_WHOLE_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/installed48_activity.rs`, `crates/jocforge-domain/tests/installed48_source_map.rs`, `crates/jocforge-domain/tests/selected_p11_geometry.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `selected-primary-installed48-source-map-and-activity`，状态 `LOCALLY_VERIFIED_SELECTED_ACTIVITY_AND_ALL_POSTROUTE_VARIANTS`：Bind the approved selected primary metadata/control projection to the generated P11 prefix and 37 sorted non-LFE physical rows; produce callback-local source activity, prefix field0/field4 writers, and stateful full-frame activity history for 4,247 normal callbacks and four cached drains. This is a narrow model slice, not a connected production path or native consumer acceptance.
    - 向量说明：4,251 callback rows: phase, host-frame, source sample cursor, actual/advertised counts, quiet flags, writer indices and field0/field4 masks
    - 向量说明：5,695 field transition receipts derived from recomputed masks
    - 向量说明：96 selected post-route variants; V2 specifies active width>=R32(0.2) clears Z at byte offset32 after the normalizer disables field52
    - 向量说明：Selected validation-WAVE replay was in-memory only; primary-WAVE PCM identity remains NOT_REHASHED

## F04

- 功能范围：`audio/matrix/Type10`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARTIAL`；问题：`Q04`；依赖：`D10`, `D11`。
- 规格身份：`specs/03-spatial-and-audio.md` SHA-256 `1ae3ecc8058eb06f84e51ddef03e804e0e6f0678ba91df3aa7640f6aa1decbcb`。
- 矩阵 aspect 状态：`PARTIAL_OR_NOT_EVALUATED`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：`U-FRONT-PARAMETER64`, `U-FRONT-WINDOW`, `U-FRONT-ACTIVITY-ZERO-ENDPOINT`, `U-FRONT-ACTIVITY-EQUAL`, `U-FRONT-ACTIVITY-ABOVE`, `U-FRONT-ACTIVITY-MUTED`, `U-FRONT-MATRIX`, `U-FRONT-OVERLAP`, `U-FRONT-FILTER64`, `U-RECV-RING`, `U-RECV-TEMPORAL-3`, `U-RECV-TEMPORAL-7`, `U-RECV-ACC-RAMP`, `U-RECV-ACC-TINY`, `U-RECV-ACC-SNAP`, `U-RECV-TRANSFORM`, `U-QMF-FFT-ANALYSIS`, `U-QMF-FFT-SYNTHESIS`, `U-QMF-REORDER`, `U-QMF-SYNTHESIS`, `U-QMF-SURROUND-RESET`, `U-QMF-DESCRIPTOR-CONTINUOUS`, `U-QMF-DESCRIPTOR-EQUAL`, `U-QMF-DESCRIPTOR-LAST`, `U-QMF-PROTOTYPE-OK`, `U-QMF-PROTOTYPE-LENGTH`, `U-QMF-PROTOTYPE-FINITE`, `U-QMF-PROTOTYPE-IDENTITY`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F04C

- 功能范围：`installed48 consumer composition through matrix/component/LFE/Type10/point-target/receiver/fitting-QMF with reset and four cached drains`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PASS_CONSUMER_SIDE_COMPOSITION`；问题：`Q04`, `Q14`；依赖：无。
- 规格身份：`specs/04-whole-spatial-chain-composition.md` SHA-256 `92325b12128023001e1f3fd4696dcec68bbd47a46acdb7886781b411a10c5d59`。
- 矩阵 aspect 状态：`CONSUMER_SIDE_COMPOSITION_DEFINED_AND_SYNTHETIC_CONTINUOUS_PASS`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `CONSUMER_SIDE_CONTINUOUS_SYNTHETIC_PASS_UPSTREAM_PRODUCERS_AND_PRODUCT_NOT_RUN`：acceptance; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：`F04-WHOLE-SPATIAL-CHAIN-VERIFICATION:2_NORMAL+4_CACHED_DRAIN+RESET+11_REJECTIONS`。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F04A

- 功能范围：`window/regular/prefix/matrix/component`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARTIAL`；问题：`Q02`, `Q04`；依赖：`D11`。
- 规格身份：`specs/04-window-matrix-and-component-audio.md` SHA-256 `350937eadb5da2432ed60723305cdc8de80a139fef803ec3a2aa5a3990ed6334`。
- 矩阵 aspect 状态：`PARTIAL_OR_NOT_EVALUATED`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：`U-FRONT-WINDOW`, `U-FRONT-ACTIVITY-ZERO-ENDPOINT`, `U-FRONT-ACTIVITY-EQUAL`, `U-FRONT-ACTIVITY-ABOVE`, `U-FRONT-ACTIVITY-MUTED`, `U-FRONT-MATRIX`, `U-FRONT-OVERLAP`, `U-FRONT-FILTER64`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F04F

- 功能范围：`four-lane diffusion filter/constructor/lifecycle`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q02`, `Q04`, `Q14`；依赖：无。
- 规格身份：`specs/04-diffusion-filter-and-lifecycle.md` SHA-256 `4b662c3721ba10ad9dafd193bf13e1bc2df571ab4cab580e46a8a6b39e9a3594`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/src/spatial/diffusion_filter/tests.rs`, `crates/jocforge-domain/src/spatial/diffusion_tail/tests.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `single-tail-diffusion-filter-block-preview`，状态 `LOCALLY_VERIFIED_SELECTED_F04F_BLOCK_PREVIEW__TAIL_LIFECYCLE_DEFERRED`：Consume one identity-validated selected 48 kHz diffusion parameter set, one immutable per-tail F04F before-state, and one finite block of 0..=4096 F32 samples. Produce four planar lanes and a complete proposed after-state using the F04F.2-.4 block transition. No callback/source/AU clock, per-object tail map, prefix routing, cross-owner commit, or production encode connection is included.
    - 向量说明：zero_coefficient_delay: four lane outputs, ring, stage banks, envelopes, and input smoother after-state
    - 向量说明：cold_fade_length_6 and empty_clears_fade: first-block half-length fade and zero-length phase transition
    - 向量说明：warm_split_5_7: bitwise output and complete after-state equal to unsplit execution
    - 向量说明：four 48 kHz selected-parameter callbacks: generated input recipe, all four lane SHA-256 values, and complete DIFFUSION_STATE_V1 after-state SHA-256
  - `single-tail-active-inactive-callback-lifecycle`，状态 `LOCALLY_VERIFIED_SYNTHETIC_F04F_5_TAIL_TRANSITIONS__PRIMARY_CALLBACK_JOIN_DEFERRED`：For one caller-selected tail, remove state immediately when inactive; when active, cold-create from previous raw PCM plus current raw PCM and split the four outputs into previous/current halves; warm-process current PCM only and use previous-filter outputs cached by the same tail. Return a proposal and next per-tail state. Do not route the 37-object map, create a callback transaction, persist raw PCM, or claim primary-source output identity.
    - 向量说明：cold_active: previous_PCM||current_PCM and exact 1536/1536 output split checked against the block preview
    - 向量说明：warm_active: previous half comes from the same-instance cache; current half comes from one 1536-sample block preview
    - 向量说明：inactive removal followed by cold reactivation does not carry the removed tail's state
    - 向量说明：warm state with missing cache returns STATE_INCOMPLETE without output or a next-tail proposal
    - 向量说明：Inputs are synthetic and establish local composition only; primary-source lifecycle/output receipts are NOT_COMPUTED

## F04FM

- 功能范围：`Hadamard4 coefficients/prefix pair-cycle generation`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q02`, `Q04`, `Q14`；依赖：无。
- 规格身份：`specs/04-diffusion-mathematical-generation.md` SHA-256 `eeba9137918ffb647e3b0a68ca5b210aadd0b0815fed3eec7ae199a08057484d`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F04FCM

- 功能范围：`fixed48k diffusion constructor mathematical config`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q02`, `Q04`, `Q14`；依赖：无。
- 规格身份：`specs/04-diffusion-constructor-generation.md` SHA-256 `e1a420ea0d4284c0cfadcf6962d34ff8cc0f877cee0d0503a6c0e2669c2978b9`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F04G

- 功能范围：`11-point prefix spread cells/boundaries/normalization/gain`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARAMETERIZED_RULES_DEFINED`；问题：`Q02`, `Q03`, `Q04`, `Q14`；依赖：`D10-P11`, `D10-W`, `D10-WK`, `D10-PC`, `D10-SC`。
- 规格身份：`specs/04-prefix-spread-and-gain.md` SHA-256 `ca5b069d16f0a0837ad4501036c74bb6e8ae3d6acfaf8bf3961ca4869923a68b`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PARAMETER_PROVIDER_DELIVERED_LOCAL_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/src/spatial/prefix_membership/prefix_membership_tests.rs`, `crates/jocforge-domain/tests/installed48_source_map.rs`, `crates/jocforge-domain/tests/selected_p11_geometry.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `selected-p11-prefix-membership-producer`，状态 `LOCALLY_VERIFIED_SYNTHETIC_AND_SEALED_SELECTED_MEMBERSHIP_VECTORS`：Implement selected P11 geometry membership for the approved selected profile, including mapped-width, power, edge-weight and route-writer contribution production. The only public constructor is bound to selected P11; this does not complete F04G, install a generic basis API, or connect F04 production.
    - 向量说明：nine declared gain cases and 163,832 primitive F32 words
    - 向量说明：selected P11 basis: 11 points, 33 F32 words
    - 向量说明：primary active-width routing includes the V2 threshold/normalizer clarification for source22 variant44

## F04GA

- 功能范围：`selected prefix spread supplied24 functional values and complete caller/provider contract`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q02`, `Q03`, `Q04`, `Q14`；依赖：无。
- 规格身份：`specs/04-spread-authorized-parameters.md` SHA-256 `4c8b966929818a9a0ea8cace90830bd5e7fa9ce8fdb716931078f97ccee83013`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F04P

- 功能范围：`component attenuation/envelope parameter state`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q04`；依赖：无。
- 规格身份：`specs/04-component-parameter-state.md` SHA-256 `3e9b777a36ae722aeccd1054d778484e6a96e44d16f2534159dfe49afc7d1861`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：`U-FRONT-PARAMETER64`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F04PG

- 功能范围：`component Q15 coefficient/threshold mathematical generation`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q02`, `Q04`, `Q14`；依赖：无。
- 规格身份：`specs/04-component-parameter-generation.md` SHA-256 `2564879656a5b0e73975d99ef7a55217440131c5d4e2d6b4dc8117adecef76d8`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F04R

- 功能范围：`receiver cold state/routing/interpolation/delays`。
- 本地实现状态：`LOCALLY_VERIFIED_F04R_DIRECT_RECEIVER_ONLY__WHOLE_RECEIVER_NOT_RUN`。
- 洗白规格状态：`PARTIAL`；问题：`Q04`, `Q08`；依赖：`D11`。
- 规格身份：`specs/04-receiver-state-and-routing.md` SHA-256 `74b9b01537eb76a37582535b84426138ee8c8ecdca3486f9f43d0504c7c26306`。
- 矩阵 aspect 状态：`PARTIAL_OR_NOT_EVALUATED`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：`U-RECV-RING`, `U-RECV-TEMPORAL-3`, `U-RECV-TEMPORAL-7`, `U-RECV-ACC-RAMP`, `U-RECV-ACC-TINY`, `U-RECV-ACC-SNAP`, `U-RECV-TRANSFORM`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/src/spatial/spatial_audio_state/spatial_audio_receiver/tests.rs`。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F04D

- 功能范围：`descriptor selection and gain ramp`。
- 本地实现状态：`LOCALLY_VERIFIED_F04D_DIRECT_STAGE_ONLY__WHOLE_RECEIVER_NOT_RUN`。
- 洗白规格状态：`PARAMETERIZED_RULES_DEFINED`；问题：`Q04`；依赖：`D07-P`, `D11-QH`。
- 规格身份：`specs/04-descriptor-control.md` SHA-256 `da42e76828eb9cf4c882a72af65b5b26f3ca29d0437643ba5c79ed8546618b7e`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PARAMETER_PROVIDER_DELIVERED_LOCAL_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：`U-QMF-DESCRIPTOR-CONTINUOUS`, `U-QMF-DESCRIPTOR-EQUAL`, `U-QMF-DESCRIPTOR-LAST`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/src/spatial/spatial_audio_state/descriptor_control/tests.rs`。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F04Q

- 功能范围：`QMF fold/phase/FIR/synthesis state`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARAMETERIZED_RULES_DEFINED`；问题：`Q04`, `Q08`；依赖：`D11-QA`, `D11-QS`, `D11-QBR`, `D11-QBI`。
- 规格身份：`specs/04-qmf-analysis-and-synthesis.md` SHA-256 `13a2907c8e60bd81f948f33b9ad07f2d7dc0552b0b12f53224d27b80af610edf`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PARAMETER_PROVIDER_DELIVERED_LOCAL_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：`U-QMF-FFT-ANALYSIS`, `U-QMF-FFT-SYNTHESIS`, `U-QMF-REORDER`, `U-QMF-SYNTHESIS`, `U-QMF-SURROUND-RESET`, `U-QMF-PROTOTYPE-OK`, `U-QMF-PROTOTYPE-LENGTH`, `U-QMF-PROTOTYPE-FINITE`, `U-QMF-PROTOTYPE-IDENTITY`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F04RB

- 功能范围：`selected receiver-QMF-JOC fitting generation/state/ownership/clock/reset whole binding`。
- 本地实现状态：`LOCALLY_VERIFIED_MODE0_FITTING_DIRECT_KATS__WHOLE_RECEIVER_NOT_RUN`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q04`, `Q08`, `Q14`；依赖：无。
- 规格身份：`specs/04-qmf-receiver-whole-binding.md` SHA-256 `104d604cf876cc522a85b21f352bd0a09fc20c3171ddcd71da981c8076ea3a75`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/src/spatial/spatial_audio_state/fitting_mode0/tests.rs`, `crates/jocforge-domain/src/spatial/spatial_audio_state/spatial_audio_receiver/tests.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `actual-receiver-to-joc-qmf-projection`，状态 `LOCALLY_VERIFIED_ACTUAL_RECEIVER_TO_QMF_PROJECTION__PARENT_J05_RECEIPT_NOT_CONNECTED`：Retain receiver preconditioned rows1..15 and consume the same execution's s09, descriptor and analysis histories in the existing fitting-QMF stage; expose immutable target object/packed and carrier QMF views without a second receiver execution
    - 向量说明：cold_normal_g2 actual receiver proposal feeds the fitting-QMF stage
    - 向量说明：all 46080 target complex cells agree between 15x48x64 object and 5x48x192 packed accessors
    - 向量说明：out-of-domain packed group and carrier indices reject

## F04QF

- 功能范围：`mathematical QMF FFT and rotation`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q04`, `Q08`；依赖：无。
- 规格身份：`specs/04-qmf-mathematical-fft.md` SHA-256 `e915152a13098a18c4028ef5f3e72ba417341cdf87b17dc76645e73395319823`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：`U-QMF-FFT-ANALYSIS`, `U-QMF-FFT-SYNTHESIS`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F04QP

- 功能范围：`QMF public prototype identity/provider`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARAMETERIZED_RULES_DEFINED`；问题：`Q08`；依赖：`D11-QA`, `D11-QS`。
- 规格身份：`specs/04-qmf-prototype-source-and-loading.md` SHA-256 `8255e0d95eb91db0c5ecf6b120bab4b7f60796d121bf9249bb51db72bc91811f`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PARAMETER_PROVIDER_DELIVERED_LOCAL_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：`U-QMF-PROTOTYPE-OK`, `U-QMF-PROTOTYPE-LENGTH`, `U-QMF-PROTOTYPE-FINITE`, `U-QMF-PROTOTYPE-IDENTITY`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F05

- 功能范围：`OAMD/JOC`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARTIAL`；问题：`Q08`；依赖：`D11`。
- 规格身份：`specs/05-joc-and-object-metadata.md` SHA-256 `7210b128689a8e7d5bf47738e085761000bce72282e37d0057dc38994d6b36af`。
- 矩阵 aspect 状态：`PARTIAL_OR_NOT_EVALUATED`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：`U-JOC-QUANT-0`, `U-JOC-QUANT-1`, `U-JOC-QUANT-2`, `U-JOC-QUANT-3`, `U-JOC-WIRE-MODES`, `U-JOC-CAPACITY`, `U-JOC-RETRY-INPLACE`, `U-JOC-SEQUENCE-WRAP`, `U-OAMD-XYZ`, `U-OAMD-ESCAPED`, `U-OAMD-ROWS`, `U-OAMD-COUNT-31`, `U-OAMD-COUNT-32`, `U-OAMD-COUNT-159`, `U-OAMD-TIME-TABLE`, `U-OAMD-TIME-EXPLICIT`, `U-OAMD-TRIM-ALL-SKIP`, `U-OAMD-TRIM-ALL-DEFAULT`, `U-OAMD-TRIM-MASK`, `U-OAMD-DBMD-DEFAULT`, `U-OAMD-DBMD-CHECKSUM`, `U-OAMD-DBMD-NONDEFAULT`, `U-OAMD-ROWS-READ`, `U-OAMD-TIME-TABLE-READ`, `U-OAMD-TIME-EXPLICIT-READ`, `U-MH-FIT`, `U-MH-HISTORY`, `U-MH-MARKER-ACTIVATE`, `U-MH-MARKER-OLD100`, `U-MH-MARKER-OLDGREATER`, `U-MH-MARKER-NEWGREATER`, `U-MH-MARKER-QUIET`, `U-MH-MARKER-UNCHANGED`, `U-MH-MARKER-DISTANCE-EQUAL`, `U-MH-MARKER-DISTANCE-ABOVE`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/src/joc/coefficient_fitting.rs`, `crates/jocforge-domain/src/joc/coefficient_geometry.rs`, `crates/jocforge-domain/src/joc/coefficient_normal_equations.rs`, `crates/jocforge-domain/src/joc/coefficient_quantization.rs`, `crates/jocforge-domain/src/joc/coefficient_representation.rs`, `crates/jocforge-domain/src/joc/coefficient_search.rs`, `crates/jocforge-domain/src/joc/coefficient_solver.rs`, `crates/jocforge-domain/src/joc/fit_score.rs`, `crates/jocforge-domain/src/joc/temporal_weights.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `selected-coefficient-fit-retry-and-payload`，状态 `LOCALLY_VERIFIED_THREE_GENERATION_FULL_FIT_SELECTED_PAYLOADS_AND_HELPER_CAPACITY_RETRY__PARENT_COMMIT_DEFERRED`：Consumes object/carrier QMF windows and receiver control projections; produces all selected fit representations, initial ID14 body and deterministic fine/coarse/sparse/lower-band retry proposals without rerunning producers. Parent J05 admission, queue/commit and real-programme execution remain separate; 80-byte capacity is helper-only while product default stays372.
    - 向量说明：generation0 detects slot0 and computes 241 cached energies
    - 向量说明：generation1 detects slot12 and computes all 360 energies
    - 向量说明：generation2 detects no slot and computes all 360 energies
    - 向量说明：all segment fields, 15x16 energy history words, last-change values and 48 weights compare by exact bits
    - 向量说明：three generations x 180 cells compare every 5x5 Gram and 5-item cross word, for 540 exact equations
    - 向量说明：six fit-cell boundaries execute 67 diagnostic/search score calls and verify strict ties, initially ineligible later additions, high-band reselection, signed coefficients, penalties and stage0 sparse backup
    - 向量说明：three full generations compare every geometry and fit output, 6249 fit-stage score calls, exact selected 132/172/173-byte payloads and helper-only 80-byte retry attempt sequences

## F05Q

- 功能范围：`JOC coefficient quantization/representation/wire/retry`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARAMETERIZED_RULES_DEFINED`；问题：`Q08`, `Q14`；依赖：`D11`。
- 规格身份：`specs/05-joc-quantization-and-wire.md` SHA-256 `91fde215d0a6d5d1e820a2e29b467afd2b1fc78a5fa8f87300cc42a346152143`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PARAMETER_PROVIDER_DELIVERED_LOCAL_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：`U-JOC-QUANT-0`, `U-JOC-QUANT-1`, `U-JOC-QUANT-2`, `U-JOC-QUANT-3`, `U-JOC-WIRE-MODES`, `U-JOC-CAPACITY`, `U-JOC-RETRY-INPLACE`。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/huffman_codebooks.rs`, `crates/jocforge-domain/tests/id14_header_framing.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `quantized-huffman-body-writer`，状态 `LOCALLY_VERIFIED_BODY_BITS__SECTION_PRODUCER_AND_PARENT_DEFERRED`：Already-quantized coefficient and selected-channel inputs to exact F05Q Huffman body bits, including codebook selection, reset rules, position deltas, MSB-first writes, padding, and local capacity checks. Excludes fit/quantizer, retry, header/body composition, parent commit, and whole JOC integration; the selected ID14 header/body writer is tracked separately.
    - 向量说明：4 F05Q consumer cases: modes 0..3, table mapping, per-point/channel reset, sparse 50/100 previous and exact padded body bytes
    - 向量说明：12 capacity cases: exact byte fit, one-byte-short, and coefficient scratch overflow
    - 向量说明：5 local zero-commit failure paths across writer, decoder and provider installation
  - `id14-header-body-writer`，状态 `LOCALLY_VERIFIED_SELECTED_ID14_WRITER__UPSTREAM_AND_PARENT_DEFERRED`：Selected ID14 writer controls, descriptor bit fields, body splice, optional extension trailer, padding, writer return code, status word, output length, and attempt-buffer partial writes. This is not the full upstream section producer or whole ID14 transaction.
    - 向量说明：14 success and 10 failure writer cases with separate return/status assertions
    - 向量说明：4 exact expanded descriptor templates: one, 15, and 64 records in object order
    - 向量说明：3 live writer header bit sequences match the captured payload positions
    - 向量说明：header-only, header/body splice, extension trailer, final padding and capacity boundaries
    - 向量说明：failure initial output buffers are hash-bound 0xAA sentinels; partial writes and unchanged output length are checked separately

## F05QH

- 功能范围：`six selected public JOC Huffman codebooks/symbol-codeword identity/MSB bit traversal/decode rejection`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q08`, `Q14`；依赖：无。
- 规格身份：`specs/05-joc-huffman-codebooks.md` SHA-256 `1b593718504078e9fba296a120bd7bc7d180a8c77b4887a060d5ae1f3fa77857`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/huffman_codebooks.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `huffman-codebook-table-provider`，状态 `LOCALLY_VERIFIED_SIX_CODEBOOKS_AND_LOCAL_BITSTREAM_PRIMITIVES__OPENJOC_REPLAY_NOT_RUN`：Six hash-pinned codebooks, structural validation, MSB-first table-level encoding/decoding, and local transactional writer/reader primitives. F05Q consumer mapping is tracked as a separate direct slice; no F05Q section framing, parent commit, or full JOC integration.
    - 向量说明：588 single-symbol roundtrips across six tables
    - 向量说明：18 ascending, descending, and boundary stream cases
    - 向量说明：18 invalid symbol/type cases, including all six table domains
    - 向量说明：6 truncated-codeword cases
    - 向量说明：Table validation rejects duplicate/prefix-conflicting codewords before use

## F05J

- 功能范围：`JOC selected-profile dispatch/sequence/commit`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARTIAL`；问题：`Q08`, `Q14`；依赖：`D11`。
- 规格身份：`specs/05-joc-dispatch-and-commit.md` SHA-256 `fc549d4bdeb598b801987dc153764835b61a2e56bdd6e734b127460527397c9e`。
- 矩阵 aspect 状态：`PARTIAL_OR_NOT_EVALUATED`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：`U-JOC-SEQUENCE-WRAP`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F05JC

- 功能范围：`selected regular JOC live QMF/temporal/geometry/180-cell fit/representation retry/commit chain`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARAMETERIZED_RULES_DEFINED`；问题：`Q08`, `Q14`；依赖：`D11-JN`, `D11-JP`, `D11-JL`。
- 规格身份：`specs/05-joc-selected-producer-and-commit.md` SHA-256 `92d7be8c41adf94056c81d98514b66318b3a38b006db3426bf56919effd9f012`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PARAMETER_PROVIDER_DELIVERED_LOCAL_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F05JL

- 功能范围：`selected live JOC QMF-to-ID14 coordinator/staged transaction/sequence/writer/FIFO clock/reset/EOF`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q08`, `Q14`；依赖：无。
- 规格身份：`specs/05-live-joc-coordinator.md` SHA-256 `49c3e07298262e2d5867d936eab0ddf6d746a17a9ce844358a0966d1b5d4d779`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PROVIDER_MANIFEST_DELIVERED_LOCAL_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：`LIVE-JOC-COORDINATOR-INPUT-VECTORS:4_CONTINUOUS+2_FATAL_RESET+WRAP+4249_CLOCK`。
- 洗白包覆盖说明：`R3_LOCAL_FRESH_REVIEW_PASS_1524_CHECKS_60_RECEIPTS_112_ACTUAL_MATERIAL_114_CANONICAL_PROJECTIONS_10_STATE_FAMILIES_39_DIGESTS_6_FATAL_ZERO_COMMIT; HISTORICAL_4249_JOIN_ATTESTATION_ONLY; REAL_ADM_AND_LISTENING_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/src/joc/synthetic_coordinator.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `synthetic-coordinator-receipt-validation`，状态 `LOCALLY_VERIFIED_INPUT_ONLY_SYNTHETIC_RECEIPT_VALIDATOR__NO_PRODUCERS_OR_STATE_COMMIT`：Test-only validator for six caller-provided synthetic stage receipts and the final ID14 body. It validates canonical material, provider bindings, shapes, generation/owner tokens, upstream chain and body identity; it executes no producer and commits no JOC state.
    - 向量说明：one six-stage receipt chain success and 267-byte ID14 body
    - 向量说明：11 fail-closed mutations
    - 向量说明：15/15 independent checks; expected_used_as_seed=false

## F05JS

- 功能范围：`selected JOC exact score/log/pow kernel and generated D192/tables/scalars`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARAMETERIZED_RULES_DEFINED`；问题：`Q08`, `Q14`；依赖：`D11-JN`。
- 规格身份：`specs/05-joc-score-kernel.md` SHA-256 `7c802e8d51b4051ac10c984410c1798eb9179a38e623d904f702858bef9c59b3`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PARAMETER_PROVIDER_DELIVERED_LOCAL_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F05O

- 功能范围：`single-frame OAMD rows/timing/sections`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARAMETERIZED_RULES_DEFINED`；问题：`Q03`, `Q08`, `Q14`；依赖：`D11-XY`, `D11-Z`。
- 规格身份：`specs/05-oamd-row-and-timing-wire.md` SHA-256 `b49694e3026938c1b5f91ef95fd63cc2eea03e25161367ba26095177ff452b11`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PARAMETER_PROVIDER_DELIVERED_LOCAL_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：`U-OAMD-XYZ`, `U-OAMD-ESCAPED`, `U-OAMD-ROWS`, `U-OAMD-COUNT-31`, `U-OAMD-COUNT-32`, `U-OAMD-COUNT-159`, `U-OAMD-TIME-TABLE`, `U-OAMD-TIME-EXPLICIT`, `U-OAMD-TRIM-ALL-SKIP`, `U-OAMD-TRIM-ALL-DEFAULT`, `U-OAMD-TRIM-MASK`, `U-OAMD-DBMD-DEFAULT`, `U-OAMD-DBMD-CHECKSUM`, `U-OAMD-DBMD-NONDEFAULT`, `U-OAMD-ROWS-READ`, `U-OAMD-TIME-TABLE-READ`, `U-OAMD-TIME-EXPLICIT-READ`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/oamd_body_assembler.rs`, `crates/jocforge-domain/tests/oamd_body_read.rs`, `crates/jocforge-domain/tests/oamd_rowdata.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `oamd-rowdata-body-encoder`，状态 `LOCALLY_VERIFIED_ENCODER_ONLY__F05T_TRIM_AND_F05L_LIVE_PRODUCER_DEFERRED`：Stateless F05O integer XYZ wire-code conversion, escaped integer encode/decode, rowdata payload encoding, and one-rowdata-section OAMD body encoding. This does not complete the F05O functional ID.
    - 向量说明：U-OAMD-XYZ
    - 向量说明：U-OAMD-ESCAPED
    - 向量说明：U-OAMD-ROWS
    - 向量说明：U-OAMD-COUNT-31
    - 向量说明：U-OAMD-COUNT-32
    - 向量说明：U-OAMD-COUNT-159
    - 向量说明：U-OAMD-TIME-TABLE
    - 向量说明：U-OAMD-TIME-EXPLICIT
    - 向量说明：XYZ transition neighbors and halfway ties derived from the published integer formula
  - `oamd-single-rowdata-body-decoder`，状态 `LOCALLY_VERIFIED_SINGLE_ROW_DATA_BODY_DECODER_ONLY__OTHER_LAYOUTS_DEFERRED`：Stateless F05O body decoding for a self-described header and exactly one unprotected rowdata section, including single-frame timing and row projections through caller-supplied PG06 XY/Z tables.
    - 向量说明：U-OAMD-ROWS-READ: header row_count=7, leading_lfe=true; 1 LFE and 6 OBJECT rows
    - 向量说明：U-OAMD-TIME-TABLE-READ: header row_count=1, leading_lfe=false; table duration timing decode
    - 向量说明：U-OAMD-TIME-EXPLICIT-READ: header row_count=1, leading_lfe=false; explicit duration timing decode
    - 向量说明：Spec-derived writer-reader checks: U-OAMD-COUNT-31/32/159 header count prefix, extension, and decoded row count
    - 向量说明：Spec-derived timing mutations: dedicated base values 0/8/16/18/24 and dedicated durations plus all duration-table values representable by U11 rejected from the explicit branch
    - 向量说明：Malformed body checks: truncated/unterminated/overlong escaped lengths, rowdata-section and outer-body nonzero padding, truncation, and unsupported layouts
  - `single-frame-oamd-body-assembler`，状态 `LOCALLY_VERIFIED_SINGLE_FRAME_ASSEMBLER__FULL_F05O_F05L_DEFERRED`：Stateless encode/decode of one self-described OAMD body with exactly one rowdata section followed by one Type11 trim section from caller-resolved metadata. This does not complete F05O or F05L.
    - 向量说明：normal_multiline_trim0: seven rows with leading LFE, complete body bytes, section descriptors, and decode projection
    - 向量说明：trim2_sentinel_mask: Type11 mode2 sentinel, masked fields, signed balance, and row presence
    - 向量说明：escaped_one_group_edge: rowdata section length using one bijective base16 group
    - 向量说明：escaped_two_group_edge: rowdata section length using two bijective base16 groups and outer padding
    - 向量说明：Declared rejection IDs: row_count_zero, leading_lfe_not_first, timing_position_64, timing_duration_2049, invalid_trim_signed_balance_17, body_capacity, truncated_body, truncated_escaped_length, overflow_escaped_length
    - 向量说明：Contract-derived decoder boundaries: section count/order/duplicate/missing, nonzero control, section and outer padding

## F05T

- 功能范围：`DBMD default auto trim and trim wire`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q01`, `Q08`；依赖：无。
- 规格身份：`specs/05-oamd-dbmd-and-trim.md` SHA-256 `1e555f51466726833295b53aef5efe1325416e1e99c1652a3200478ceb82ee10`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/trim_section_codec.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `trim-section-codec`，状态 `LOCALLY_VERIFIED_TRIM_SECTION_CODEC_AND_MINIMAL_DBMD_PROJECTION__FULL_F05T_DEFERRED`：Pure Type11 trim-body encode/decode plus DBMD section framing and selected default-trim projection from sections 9 and 10. This does not complete F05T or integrate the trim body into F05O section assembly.
    - 向量说明：group_mode0_all_skip: exact body bytes, valid_bits, padding_bits and decoded Trim
    - 向量说明：group_mode1_all_default: exact body bytes, valid_bits, padding_bits and decoded Trim
    - 向量说明：group_mode2_mask_signed_rows: field mask, signed balance and row-presence bits
    - 向量说明：group_mode2_default_sentinel_and_mask: whole-default sentinel followed by a non-default masked group
    - 向量说明：group_mode2_u4_signed_endpoints: U4 endpoints and signed balance endpoints
    - 向量说明：trim_mode3_no_rows: row_count=0 standalone body
    - 向量说明：dbmd_selected_default: selected sections 9/10 projection with row_count=16
    - 向量说明：DBMD projection row_count boundaries: 1 and 159 accepted; 0 and 160 rejected with DBMD_ROW_COUNT
    - 向量说明：Declared rejection IDs: duplicate_section, truncated_header, truncated_payload, nonzero_terminator_tail, checksum_mismatch, nondefault_trim, dbmd_row_count_zero, dbmd_row_count_160, nonzero_padding, truncated_trim_body
    - 向量说明：Contract-derived boundaries: standalone row_count=160 acceptance, Trim shape/domain/U4/signed balance, reserved/group_mode3, DBMD header/required sections/supplemental/warm mode, signed zero and skipped-record fields

## F05H

- 功能范围：`fit-control marker and metadata generation history`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q03`, `Q08`, `Q14`；依赖：无。
- 规格身份：`specs/05-fit-marker-and-metadata-history.md` SHA-256 `39cb2ef34ef976b369c4390969d6a38e291d3268e9e2c7e43499ea299b692f26`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：`U-MH-FIT`, `U-MH-HISTORY`, `U-MH-MARKER-ACTIVATE`, `U-MH-MARKER-OLD100`, `U-MH-MARKER-OLDGREATER`, `U-MH-MARKER-NEWGREATER`, `U-MH-MARKER-QUIET`, `U-MH-MARKER-UNCHANGED`, `U-MH-MARKER-DISTANCE-EQUAL`, `U-MH-MARKER-DISTANCE-ABOVE`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F05L

- 功能范围：`selected live OAMD ID11 same-body reader/target transaction and writer/queue lifecycle/reset`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`SELECTED_PROFILE_FRESH_R5_LOCAL_REVIEW_PASS`；问题：`Q04`, `Q08`, `Q14`；依赖：无。
- 规格身份：`specs/05-oamd-id11-live-producer.md` SHA-256 `767005b297614e2ad5c374a9018642a22b577af37941f12486523cc43fb247dc`。
- 矩阵 aspect 状态：`DEFINED_FRESH_R5_LOCAL_REVIEW_PASS_WHOLE_PACKAGE_NOT_READY`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：`OAMD-ID11-LIVE-INPUT-VECTORS:10_CALLBACKS+8_WRITERS+QUEUE+CAPACITY+RESET`, `OAMD-ID11-READER-TRANSACTION-VERIFICATION:R5_SAME_BODY_96_TARGETS+NEGATIVE_Z+POST_BODY_FAILURE+FULL_RESET`。
- 洗白包覆盖说明：`R5_502_CHECKS_ZERO_MISMATCH; 10_WRITER_CALLBACKS_8_BODIES; 8_READER_CALLBACKS_96_RECORDS_576_TARGET_WORDS_414_WIRE_BYTES; SAME_BODY_96_TARGETS; NEGATIVE_Z_FATAL_ZERO_COMMIT; 9047_BYTE_COLD_STATE; 7_ORIGINAL_PLUS_2_NEGATIVE_CHECKPOINTS; POST_BODY_FAILURE_AND_FULL_RESET; HISTORICAL_4251_4249_ATTESTATION_ONLY`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F05R56

- 功能范围：`main marker/history to Type11 decode/previous-aware reader56 and sixteen-six target updates with generation/reset/fail-stop`。
- 本地实现状态：`LOCALLY_VERIFIED_F05R56_SELECTED_PRODUCER_ONLY__PARENT_ADMISSION_DEFERRED`。
- 洗白规格状态：`PASS_MODEL_POSTCOMPARE`；问题：`Q04`, `Q08`, `Q14`；依赖：无。
- 规格身份：`specs/05-reader56-target-updates-composition.md` SHA-256 `4f4fe2bc3b0dd08bc59e46ea5d3a48e2eafc4051906c2fda258250351261ce0d`；`specs/05-fit-marker-and-metadata-history.md` SHA-256 `39cb2ef34ef976b369c4390969d6a38e291d3268e9e2c7e43499ea299b692f26`；`specs/05-oamd-id11-live-producer.md` SHA-256 `767005b297614e2ad5c374a9018642a22b577af37941f12486523cc43fb247dc`；`specs/05-oamd-row-and-timing-wire.md` SHA-256 `b49694e3026938c1b5f91ef95fd63cc2eea03e25161367ba26095177ff452b11`。
- 矩阵 aspect 状态：`COMPOSITION_DEFINED_AND_MODEL_POSTCOMPARE_PASS_LEAF_REUSE_DECLARED`：inputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_R3_REVIEW_PASS; REAL_PRODUCTION_AND_WHOLE_SONG_VALIDATION_OPEN`：outputs, acceptance; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：`READER56-TARGET-UPDATES-VERIFICATION:8_CALLBACKS+6_READERS+RESET+7_REJECTIONS`。
- 洗白包覆盖说明：`R3_LOCAL_PASS_96_MAIN_16_MUTE_576_PANNER_WORDS_4_PANNER_REJECTIONS_7_TRANSACTION_REJECTIONS_7_RESETS; TYPE11_WIRE_LEAF_REUSED_F05O_F05T; REAL_PRODUCTION_OPEN`。
- 本地定向测试：`crates/jocforge-domain/src/joc/oamd/reader_target/tests.rs`。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F06

- 功能范围：`measurement/BSI/DRC`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARTIAL`；问题：`Q01`, `Q07`；依赖：`D12`。
- 规格身份：`specs/06-meter-bsi.md` SHA-256 `5402250feaee342af79e6ff53b59a33833722e2123c602a21ccd9ec2af0bc752`。
- 矩阵 aspect 状态：`PARTIAL_OR_NOT_EVALUATED`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：`U-DIAL--120.0`, `U-DIAL--31.5`, `U-DIAL--31.0`, `U-DIAL--24.49`, `U-DIAL--24.5`, `U-DIAL--1.0`, `U-DIAL-0.0`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F06L

- 功能范围：`meter ordered log/energy metric/sample and true-peak metric`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-meter-log-and-peak.md` SHA-256 `c8a9aec8c931391bfbb4ecaca01a8e5cbec3ba680899f516727c6574a170e38f`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/src/meter/true_peak/tests.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `ordered-log-energy-peak-accumulator-window-and-reduction`，状态 `LOCALLY_VERIFIED_F06L_LOG_ENERGY_SAMPLE_TRUE_PEAK_ACCUMULATOR_WINDOW_AND_REDUCTION__WHOLE_METER_DEFERRED`：Direct-vector verification of existing pure ordered-log, energy_metric, sample-peak, and true-peak metric functions plus F06L.7 normalized-energy accumulation, F06L.8 window-end normalization, and F06L.9 stateless ordered reduction. Does not add a duplicate log/peak network, maintain meter-window state, or run FIR or K-weighting.
    - 向量说明：log_0..log_13
    - 向量说明：energy_0..energy_7
    - 向量说明：sample_peak_0..sample_peak_5
    - 向量说明：true_peak_0..true_peak_7
    - 向量说明：accumulator_0..accumulator_6
    - 向量说明：window_0..window_5
    - 向量说明：reduction_0..reduction_6

## F06I

- 功能范围：`meter integrated histogram/absolute-relative gate/continuous state`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-integrated-meter-state.md` SHA-256 `e5caf84805cb24850dd0cf85e18d1dc4a4ee91fcb9e334cfc6f9881952efdb72`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/integrated_meter_state.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `integrated-meter-synthetic-state-and-event-transition`，状态 `LOCALLY_VERIFIED_F06I_SYNTHETIC_STATE_EVENTS_AND_BIN_MAPPING__METER_CLOCK_AND_INTEGRATION_DEFERRED`：Selected fixed 48 kHz F06I period, cold/reset 24,512-byte state, ordered energy-pair appends, 20-pair event, recent-group movement, histogram/bin update, absolute gate, and result. Tests ten synthetic schedules plus seven bin cases against complete state hashes. Excludes the five F06W schedule cases, historical real-music inputs, F06U getter/fallback, frontend/ADM, and whole meter.
    - 向量说明：silence40
    - 向量说明：steady_high45
    - 向量说明：steady_low40
    - 向量说明：steady_below_gate40
    - 向量说明：rise_fall60
    - 向量说明：histogram_halving
    - 向量说明：histogram_threshold25
    - 向量说明：absolute_counter_limit
    - 向量说明：absolute_exponent_alternate
    - 向量说明：reset_repro35
    - 向量说明：7 F06I.2 bin boundary metric/index pairs

## F06W

- 功能范围：`independent meter sample ledger/K-filter/1024-prefix/960-window`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARAMETERIZED_RULES_DEFINED`；问题：`Q01`, `Q07`, `Q14`；依赖：`D12-K`。
- 规格身份：`specs/06-meter-window-and-output-clock.md` SHA-256 `c08dd41c42dd231ef23a683415f59b9b7db60375a3da2b8140501969933d073c`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PARAMETER_PROVIDER_DELIVERED_LOCAL_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/meter_output_schedule.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `meter-output-sample-accounting-and-span-ledger`，状态 `LOCALLY_VERIFIED_F06W_1_2_SYNTHETIC_SPANS_AND_SAMPLE_COUNTS__WINDOW_STATE_DEFERRED`：F06W.1/.2 only: stateless normal/drain callback span accounting, startup/pending/processed invariants, 1536-sample output padding, complete-1024 PCM consumption count, and count-only complete-960 window/remainder totals. Does not consume PCM or execute F06W.3 K-filter/F06W.4 power sum/F06W.5 six-lane window state or emit F06I records.
    - 向量说明：schedule_cases[0] input_source_samples=1
    - 向量说明：schedule_cases[1] input_source_samples=1024
    - 向量说明：schedule_cases[2] input_source_samples=1536
    - 向量说明：schedule_cases[3] input_source_samples=1537
    - 向量说明：schedule_cases[4] input_source_samples=6522906

## F06KG

- 功能范围：`public 48k biquad rational expansion and fixed normalization generated complete K matrix`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-meter-k-matrix-generation.md` SHA-256 `d6287e17dec2a4b93b5756eb25345fd41cc4329eebeacefabd8e71ad84d33e31`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/k_matrix_provider.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `selected-k-matrix-provider-generator`，状态 `LOCALLY_VERIFIED_EXACT_RATIONAL_MATRIX_AND_BOUNDS_PROVIDER__CONSUMER_DEFERRED`：Selected 48 kHz modern two-stage exact-rational K-matrix generator, 64 matrix words, eight cold words, and selected state-bound values. It does not implement F06W.3/.4 sample consumption, windowing, meter output, audio frontend, or whole F06.
    - 向量说明：all 64 generated matrix words and eight cold words in stage/row/lane F32LE order
    - 向量说明：complete 72-word/288-byte identity SHA-256
    - 向量说明：selected lower/upper/noise state-bound provider identity
    - 向量说明：unsupported sample-rate, mode, and stage-count rejection

## F06TP

- 功能范围：`authorized eleven-value true-peak cascade state/1024-prefix/metrics/programme and EOF contract`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q07`, `Q14`；依赖：`D12-TP`。
- 规格身份：`specs/06-true-peak-cascade-and-metrics.md` SHA-256 `e0491288c3a9beb59ecb7b017d505ee16eb9f401d53bd0fd918573644c1876b3`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PARAMETER_PROVIDER_DELIVERED_LOCAL_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/src/meter/true_peak/tests.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `selected-six-channel-true-peak-block-cascade`，状态 `LOCALLY_VERIFIED_AUTHORIZED_4X_CASCADE_STATE_AND_LOCAL_PEAKS__FRONTEND_DEFERRED`：Selected six-channel 48 kHz/1024-sample block only: authorized 12-tap coefficients, 12-tap FIR, two-stage 4x interpolation, LFE ordinal-3 bypass, complete per-lane FIR history, per-block sample/true-peak metrics, and programme maxima. No frontend, sample-window scheduler, full F06 meter, or product chain.
    - 向量说明：9 reproducible cases: 8 continuous blocks plus an independent reset with nonzero LFE history
    - 向量说明：193,536 measured output words validated by per-channel full-output hashes
    - 向量说明：249,912 complete per-lane FIR state words compared against complete serialized next-state fixtures
    - 向量说明：540 per-lane peak fields and 126 programme fields compared exactly
    - 向量说明：LFE bypass output and bit-preserved state; sample/true peak local metric branches

## F06ND

- 功能范围：`none DRC primary-secondary statistics/paired filter/downmix/limiter/packed control continuous state`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARAMETERIZED_RULES_DEFINED`；问题：`Q05`, `Q07`, `Q14`；依赖：`D12-DRC`。
- 规格身份：`specs/06-none-drc-statistics-and-control.md` SHA-256 `62dace058d92eaf8c4b9262c40d885d37801e55798d0be331e07bae0117a7081`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PARAMETER_PROVIDER_DELIVERED_LOCAL_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F06NR

- 功能范围：`secondary unquantized reconstruction frame/flags/217-spectrum/overlap128/carrier/EOF connection`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`SELECTED_SECONDARY_FFT_LOCAL_REVIEW_PASS_REAL_SONG_OPEN`；问题：`Q05`, `Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-secondary-reconstruction-clock-and-overlap.md` SHA-256 `26a67687d019b480fcb450cc8321198f8df3d5182c55d49adefc4179fa23f74a`。
- 矩阵 aspect 状态：`PARTIAL_OR_NOT_EVALUATED`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F06NI

- 功能范围：`secondary inverse256 long/short split/descriptor pre-post rotations/two multiply2 stages/clamp/output layout`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q05`, `Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-secondary-inverse-spectrum-and-rotations.md` SHA-256 `5d811a74d6ae22bd50be1e74656ae81a43e84582100e5cbddd908d394295805d`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F06NF

- 功能范围：`secondary inverse fixed FFT64/128 radix/middle/final-permutation network and complete mathematical parameter provider`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q05`, `Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-secondary-fft64-128.md` SHA-256 `a899c158eeed9485090d4307ef3629eadeca9359ac0d5085612c5f632390ad66`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F06WH

- 功能范围：`selected meter finalize to none-DRC whole composition`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-meter-none-drc-whole-composition.md` SHA-256 `94e8d42e77e71f4f16e99a74ee3c808cfc925074e19dd814762d21cde6f259d2`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F06U

- 功能范围：`single-slot mean/admission/DIoff unavailable-ITU fallback`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-ungated-mean-and-fallback.md` SHA-256 `07c850be31dbe42a1396634bd96dfd4a54fbd80aa27b3d39a36998545cdc3b9e`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/ungated_mean.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `ungated-single-slot-mean-and-di-off-getter`，状态 `LOCALLY_VERIFIED_F06U_SINGLE_SLOT_MEAN_AND_DI_OFF_SELECTION__INTEGRATED_METER_DEFERRED`：Fixed period-one/capacity-one F06U admission, mean update, normalized event result, full eight-field state projection, and selected 1770-4 DI-off getter. Consumes explicit energy mantissa/exponent pairs. Reuses the selected F06L log network only; no meter frontend, F06I integrated state, F06W scheduling, K/FIR processing, measurement owner, source/AU parent, or full none-DRC path.
    - 向量说明：all 10 mean sequences and all 102 trace steps from explicit ordered energy pairs
    - 向量说明：all four coupled mean-state terminal projections; getter comparison only for the three unavailable-ITU fallback cases
    - 向量说明：all four primary vector input rejections
    - 向量说明：all four valid ITU getters and all three unsupported-config rejections
    - 向量说明：legacy -120 getter observation checked against actual F06U cold state, not a legacy output seed

## F06DG

- 功能范围：`actual shared post-OAR DV32 mathematical parameter provider`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-shared-dv32-parameter-generation.md` SHA-256 `a8a88ef1bb74e8f482fb09dfa2fb169f6a51f44f1d58d643957895e2bbbd9723`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F06D

- 功能范围：`shared post-OAR DV32 continuous parameters/history32/callback state and audio`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-shared-dv32-state-and-audio.md` SHA-256 `f99c0f47d502d949f726633d100ae8868acf4669dfe99f4624ebfb1bf42d8691`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F06M

- 功能范围：`OAR gain-row/ramp32 mixing and disabled per-task filter gate`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q03`, `Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-oar-mix-ramp-and-filter-gate.md` SHA-256 `dfe8ceac3e6e9ca9939e3e40bcf985e26a842c70f8646dfc6e61da3065f86de3`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F06OG

- 功能范围：`secondary8 normalized OAR geometry mathematical generation`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q03`, `Q07`；依赖：无。
- 规格身份：`specs/06-oar-secondary-geometry-generation.md` SHA-256 `d6de429d69ceaa6977a622eff408551ace9dadb48dfad8ae4cf6da68c4b2d88a`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F06OM

- 功能范围：`OAR control metadata/source30/canonical record60 conversion`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q03`, `Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-oar-metadata-and-source-records.md` SHA-256 `09ff22c4c7ead62feeeb38d37254b15d04bead13660191bd11494de13d2f5572`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F06OP

- 功能范围：`secondary8 OAR point target/axis equal power/warped y/fold back to side`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q03`, `Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-oar-point-target.md` SHA-256 `28d13502086a7ead03daff76c089449fd871297a6c5c56b30b1e07ce8a0565e8`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F06OE

- 功能范围：`separate frame extension/reset map providers/record cold and full sixty-byte comparison`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q03`, `Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-oar-extra-state-and-record-comparison.md` SHA-256 `10a1c8568da4446c8011a6c6d4534503d8bc0f648d1a3b63a270b2b246c79c70`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F06QT

- 功能范围：`queue U32 time selector/subframe full and remaining ramp/time fallback/48k interpolation length`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q03`, `Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-oar-queue-time-selection.md` SHA-256 `830befc90a52eb8f8689e3d77296c9af8952733de0937b9dfdf17120918a69af`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F06QM

- 功能范围：`queue prepare/tail revoke/front descriptor rotation/append clone boundary/active timestamp rebasing`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q03`, `Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-oar-queue-mutation-and-reference-ownership.md` SHA-256 `8ec12d2d63c56dda72e98057c11b5cb582c1ab0b8e36057f73cbf651a126a21e`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F06QS

- 功能范围：`equal-capacity owned metadata snapshot full source allocation copy/conditional optional payload copy and defaults`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q03`, `Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-oar-snapshot-copy.md` SHA-256 `ee5bbb9cb942a06b03c8f81095ebe9c8a6da21ce22d868182dc0aeaf6965d87e`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F06OD

- 功能范围：`48byte source description fields/selected LFE-object task generation/coded-aux-object descriptor installation`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q03`, `Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-oar-description-generation.md` SHA-256 `1d715bc266890e3feec114b78ec840d7f003255f857299f1fa84b5ccfa6b0616`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F06OS

- 功能范围：`selected single coded bit mathematical direct-selector mapping`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q03`, `Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-oar-coded-mask-direct-selector.md` SHA-256 `a77b26032a9017749940e538fc2f23295831c2194abc1da56e7441dbab436398`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F06OB

- 功能范围：`snapshot timing/source writers and persistent header-record materializer bridge`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q03`, `Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-oar-snapshot-to-record-bridge.md` SHA-256 `5944a659fff3483b90263525ab58106fdc211c97f0901d09b3681b0c4f889c05`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F06OR

- 功能范围：`selected sixteen logical sources task-global-local mapping and coded LFE selector supplier`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q03`, `Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-oar-selected-task-routing.md` SHA-256 `effdbd69f19a0b6469568168c1121d1e279ec12385c8192c627c9d91a810d20d`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F06OC

- 功能范围：`executed independent point-model OAR callback ownership/cache/clock/reset/drain and composition`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q03`, `Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-oar-model-callback-state-and-clock.md` SHA-256 `7342e87525e30e09e2ed95fb07593cebaa81cc113d2ce3798d99ec7cefaf0ae9`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F06OF

- 功能范围：`frontend raw-input normal/drain clock and owned SCP-control callback adapter/LFE ring/metadata history`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q03`, `Q07`, `Q14`；依赖：无。
- 规格身份：`specs/06-oar-frontend-input-and-clock-join.md` SHA-256 `2a98742b9032b900bb42da7f6a7c404a835c9539fbb55040939d7d025c8ce423`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F07

- 功能范围：`input history/transient/dither`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARAMETERIZED_RULES_DEFINED`；问题：`Q05`；依赖：`D05`。
- 规格身份：`specs/07-history-and-transient.md` SHA-256 `61a957410a05e1e44c15d7a3f6622d2ff4c2a1135020f35261c0dee1979f1eac`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PARAMETER_PROVIDER_DELIVERED_LOCAL_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：`U-TRANSIENT-01`, `U-HISTORY-01`。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/input_history.rs`, `crates/jocforge-domain/tests/transient_channel.rs`, `crates/jocforge-domain/tests/transient_flag_primitives.rs`, `crates/jocforge-domain/tests/transient_parameters.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `input-history-block-transition`，状态 `LOCALLY_VERIFIED_TWO_FRAME_SIX_LANE_F07_1_HISTORY__TRANSIENT_AND_PARENT_OWNER_OPEN`：Implements one F07.1 256-word transition over one 1,280-word lane history, returning old history[256..512] and the next history formed by dropping the first 256 words and appending the current block. Six lanes and two frames are replayed against U-HISTORY-01 byte files. It does not implement F07.2 transient filtering, F08 transform history, programme ownership, source/callback clocks, or parent reset/fatal/EOF transactions.
    - 向量说明：U-HISTORY-01 provides two complete frames, six lanes, 1,536 samples per lane per frame, and cold positive-zero history.
    - 向量说明：The test executes six 256-word blocks per lane per frame, then compares all delayed-output bytes and all six terminal 1,280-word histories.
    - 向量说明：A focused transition test checks the 1,024-sample delay and exact preservation of signed-zero/subnormal bit patterns supplied as U32 words.
    - 向量说明：The binary fixture is used as input data only; expected output and terminal files are compared after computation.
  - `structure-refresh-flag-projection`，状态 `LOCALLY_VERIFIED_SELECTED_PROFILE_4096_STRUCTURE_CASES__WHOLE_F07_OPEN`：Projects caller-supplied six-block short_consumed and other_consumed flags to the selected-profile F07.4 structure-refresh flags. SPX=0 and end-bin=217 are fixed by the selected profile. It does not produce either input flag, implement transient filtering/queues, or own frame state.
    - 向量说明：Enumerates short_mask and other_mask over 0..=63, with bit b assigned to block b, in the washed nested-loop order.
    - 向量说明：The test serializes six outputs per case as ASCII 0/1 and compares all 24,576 bytes by SHA-256.
    - 向量说明：This synthetic boundary coverage does not validate the F07.2/F07.3 producers that supply short_consumed and other_consumed.
    - 向量说明：The F07 r1 review status records 4,096 cases, but its reviewed spec SHA differs from the current spec SHA; this local formula replay is not a fresh current-spec review.
  - `selected-dither-flag-plan`，状态 `LOCALLY_VERIFIED_2048_SELECTED_DITHER_PLANS__WHOLE_F07_OPEN`：Projects caller-supplied prior_short[5] and short_consumed[6][5] into six-by-five dither flags, header syntax presence, and next_previous_short[5] for selector 7/acmod 7/coupling disabled/gate enabled. It does not produce short flags, mutate programme state, serialize a header, or close F07/F12.
    - 向量说明：Enumerates previous_mask 0..=31 and pattern 0..=63 in the washed order; current short bits use (pattern >> ((block+lane)%6)) & 1.
    - 向量说明：The test computes flags, syntax presence, and next previous_short for 2,048 independent plans, serializes 36 ASCII bits per plan, and compares the full 73,728-byte digest.
    - 向量说明：The synthetic plans do not produce short flags from F07.2/F07.3 or run F07.8 header writing.
    - 向量说明：The F07 r1 review status records 2,048 plans, but its reviewed spec SHA differs from the current spec SHA; this local formula replay is not a fresh current-spec review.
  - `actual-d05-transient-channel-block`，状态 `LOCALLY_VERIFIED_36_ACTUAL_D05_BLOCKS__CURRENT_F07_REVIEW_BINDING_OPEN`：One full-band lane only: uses the identity-first selected D05 provider, F09G four-lane mean-square, F07.2 two-stage sample filter and three-stage paired filter, block-end history clear, peak/tier decisions, and separate four-slot short/other queue advancement. It runs a caller-ordered single-lane block stream. It does not implement the other lanes, F07.1 source history join, F07.4/.5 wiring, programme owner, F08, F12 transaction, or whole F07.
    - 向量说明：Thirty-six input-only 256-word blocks are processed in one sequence from cold state. Expected results are read only after all 36 outputs and next states are calculated.
    - 向量说明：Each block compares its 1,024-byte filtered F32LE digest, 124-byte next-state digest, mean word, source/consumed flags, three tier flags per producer, and four last-tie peak positions.
    - 向量说明：The six D05 groups contain 33 actual F32 words. Their internal distribution permission is internal-only; the provider is not a public/third-party distribution grant.
    - 向量说明：R3 actual review reports 11,088 field comparisons with zero mismatches, but its reviewed F07 spec SHA differs from the current F07 spec SHA. Local implementation matches the bound results; current-spec independent review remains open.
    - 向量说明：This is one-lane synthetic input coverage. It does not claim real PCM, six-lane programme integration, or natural song producer equivalence.

## F07A

- 功能范围：`authorized D05 transient33 functional values and continuous actual-parameter verification`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q05`；依赖：无。
- 规格身份：`specs/07-transient-authorized-parameters.md` SHA-256 `7d3e85c1281653dfa8228b3162f6df95f0b53b367ce85d9333b4c0151d9dbe1f`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/transient_parameters.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `selected-d05-transient-parameter-provider`，状态 `LOCALLY_VERIFIED_33_HASH_PINNED_D05_WORDS__WHOLE_F07A_OPEN`：Identity-first parsing of the one selected internal-only D05 provider: 33 finite F32 words across P/A/S/O/M/C, exact field/group order, per-group LE hashes, and 132-byte canonical serialization. It performs no filtering, state update, profile fallback, file/environment access, or public distribution.
    - 向量说明：Provider tests validate 33 words, 28 distinct F32 bit patterns, and the canonical 132-byte serialization SHA.
    - 向量说明：The adjacent F07 single-lane test consumes the loaded typed provider over 36 input-only blocks; it does not use the provider JSON as runtime lookup data.
    - 向量说明：The provider is authorized for internal research/implementation only; this artifact does not authorize public or third-party distribution.
    - 向量说明：The r3 review binds the provider bytes but its reviewed F07A spec SHA predates the current F07A spec SHA; this provider identity check does not close the full F07A consumer/lifecycle contract.

## F08

- 功能范围：`primary/companion transforms with selected math-generated-v1 provider`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARAMETERIZED_RULES_DEFINED`；问题：`Q02`, `Q05`；依赖：无。
- 规格身份：`specs/08-transforms.md` SHA-256 `bf4806b1bc6bc183b87892aace46338b2b35fc6bff08fd3ae6b46527dcb1982e`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PARAMETER_PROVIDER_DELIVERED_LOCAL_COMPOSITION_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：`U-XFORM-AB-LSL`, `U-XFORM-LFE-AB`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F08NZ

- 功能范围：`nonzero long/short/LFE transform-to-exponent composition`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q05`；依赖：无。
- 规格身份：`specs/08-nonzero-transform-composition.md` SHA-256 `629d8c5c2853f81f82bcee281370fbb39efea0a62255f367da6fd3677172175d`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F09

- 功能范围：`POWER/PRIMARY/refresh`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARTIAL`；问题：`Q05`；依赖：`D06`。
- 规格身份：`specs/09-exponents.md` SHA-256 `41cd157140d0deaab51fa9a895c33758cb49eb46cf5844a2883c1d6e8bb81739`。
- 矩阵 aspect 状态：`PARTIAL_OR_NOT_EVALUATED`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：`U-CFG-01`, `U-EXP-01`, `U-EXP-02`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/src/codec/configuration_import.rs`, `crates/jocforge-domain/src/transform/lfe_transform.rs`, `crates/jocforge-domain/src/transform/long_transform.rs`, `crates/jocforge-domain/tests/coefficient_power.rs`, `crates/jocforge-domain/tests/configuration_import.rs`, `crates/jocforge-domain/tests/exponent_refresh_state.rs`, `crates/jocforge-domain/tests/lfe_coefficient_exponent.rs`, `crates/jocforge-domain/tests/lfe_transform_to_raw.rs`, `crates/jocforge-domain/tests/long_transform_refresh_composition.rs`, `crates/jocforge-domain/tests/raw_exponent_from_power.rs`, `crates/jocforge-domain/tests/transform_history_owner.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `exponent-refresh-state-transition`，状态 `LOCALLY_VERIFIED_BOUNDED_REFRESH_STATE_MAP__WHOLE_F09_DEFERRED`：One bounded in-place refresh helper over caller-owned I16 current/work rows and marker. It implements marker-copy, weighted refresh, and ordinary refresh branches with the delivered threshold/order rules. It does not produce caller mode/marker, load POWER/bias/configuration, or define reset/EOF/fatal/parent lifecycle.
    - 向量说明：10 input recipes exercise 12 ordered state transitions.
    - 向量说明：Includes weighted normal/special threshold discrimination, marker precedence, ordinary clamp, nonzero begin, and repeated refresh on retained state.
    - 向量说明：All tested calls satisfy the package preconditions; violated-precondition outcomes are not defined.
  - `raw-exponent-from-power-bits`，状态 `LOCALLY_VERIFIED_POWER_BIT_RAW_EXPONENT_KERNEL__WHOLE_F09_DEFERRED`：Stateless U32 POWER-bit kernel: wrapping double, source-exponent extraction, difference clamp, signed I16 bias wrap, signed cap, and arithmetic right shift. It does not produce POWER, bias, cap, or route selection.
    - 向量说明：5 recipes cover 50 input-word/bias output pairs.
    - 向量说明：Inputs include positive/negative zero, subnormal boundaries, and selected normal/large exponent words from the package.
    - 向量说明：Expected values are postcompute assertions and are never used as inputs.
  - `coefficient-pair-to-power-row`，状态 `LOCALLY_VERIFIED_FINITE_253_BIN_POWER_ROW__WHOLE_F09_DEFERRED`：For caller-supplied finite real/imaginary F32 coefficient rows, emits 253 POWER bit words with separate F32 square/add rounding and cap 1.0. Includes only the washed 1x1 and 2x2 wrapper mappings and bias-slot clear-to-zero effect. It does not generate coefficients, choose configuration, define general errors, close bias lifecycle, LFE, or whole F09.
    - 向量说明：Six input-only recipes compare all 3,036 POWER U32 outputs.
    - 向量说明：Five calls compare all 2,387 downstream raw-exponent I16 results using the already implemented helper.
    - 向量说明：Recipes cover zero, periodic values, the last-bin tail, repeated 2x2 buffer reuse, and exact F32 rounding-edge bit cycles.
    - 向量说明：Expected outputs are postcompute assertions and are never inputs; package hashes and independent-check identity are verified.
  - `lfe-coefficient-pair-exponent`，状态 `LOCALLY_VERIFIED_SELECTED_5_1_LFE_TWO_BLOCK_SEVEN_BIN_ROUTE__WHOLE_F09_DEFERRED`：Computes selected 5.1 LFE outputs directly from two blocks of seven finite real/imaginary coefficient pairs. Includes the zero-energy shortcut, energy clamp, bit-derived exponent, fixed cap, caller output overwrite, and tail preservation. Does not consume full-band mode/format/bias, use the separate POWER probe as an internal temporary, generate coefficients, or close whole F09.
    - 向量说明：Five input recipes and ten consecutive calls compare 140 LFE outputs.
    - 向量说明：Mode 0/1, format 2/7, and bias 0/2/4 vary while the same coefficient recipes produce invariant LFE rows.
    - 向量说明：Every call overwrites two seven-bin output rows while preserving the eighth tail-canary slot; block 0 matches the combined-call LFE output.
    - 向量说明：The fixture also records 140 separate POWER probe values, explicitly not LFE internal captures; expected LFE outputs are postcompute checks.
  - `long-transform-refresh-composition`，状态 `LOCALLY_VERIFIED_256_WORD_TRANSFORM_HISTORY_AND_SAME_CALL_REFRESH_COMPOSITION__WHOLE_F09_DEFERRED`：Consumes one finite 256-word F32 block, one branch-owned 128-word history, and the fixed selected-profile transform tables. Returns fold/pre/FFT/post/wrapper stage words and next history. The same-call synthetic composition joins primary/companion spectrum through 253-bin POWER, 217 raw exponents, and 128-bin refresh; no source-frame identity or ADM/sample clock is assigned.
    - 向量说明：Six standalone recipes x two calls compare 3,072 words at each fold/pre/FFT/post/wrapper boundary and 1,536 history words.
    - 向量说明：The 16-complex canary compares 32 output words and distinguishes preincrement, per-pair coefficient selection, and missing per-stage reset variants.
    - 向量说明：Three composition recipes x two calls compare 3,072 spectrum words, 1,518 POWER words, 1,302 raw exponents, 2,604 refresh current/work words, and six marker outcomes.
    - 向量说明：Expected arrays are postcompute comparisons only; native internal per-butterfly FFT stage buffers remain NOT_RUN.
  - `lfe-long-transform-to-raw`，状态 `LOCALLY_VERIFIED_LFE_FOLD_PRE_FFT8_RECURRENCE_POST_AND_RAW__PARENT_OWNER_CLOCK_OPEN`：Consumes one synthetic finite 256-word block, separate primary/companion 128-word histories, and short_flag=0 by fixed route. Shares only the approved fold/pre prefix; applies a dedicated 16-gather FFT8, recurrence, seven-bin post-rotation, then the selected LFE coefficient-pair raw helper. Does not produce source samples, define history cold/reset ownership or clocks, select short mode, or close whole F09.
    - 向量说明：Three recipes x two consecutive calls compare 3,072 fold values, 3,072 pre values, 192 low FFT/recurrence values, 84 seven-bin branch coefficients, 1,536 history values, and 42 raw exponents.
    - 向量说明：Both history rows are independently retained over each two-call recipe; short_flag is fixed to 0.
    - 向量说明：The seven-entry raw output buffer preserves its eighth caller tail-canary slot. Expected outputs are postcompute checks only.
  - `transform-history-owner-and-workspace-initializer`，状态 `LOCALLY_VERIFIED_SIX_SOURCE_PRIMARY_COMPANION_BANKS_AND_ZERO_WORKSPACE__RESET_CLOCK_OPEN`：Models six source positions in order 0..5, each with independent primary/companion 128-word history values, and initializes the selected typed scratch workspace. Slot 5 is only the LFE boundary; no external speaker mapping, native ABI, or pointer values are modeled.
    - 向量说明：Six zero/impulse/periodic branch sequences compare all 1,536 history words across two consecutive calls.
    - 向量说明：The owner container verifies six distinct source-position values and independent primary/companion banks; test-only recipe assignment does not assert native lane mapping.
    - 向量说明：The typed initializer compares 30 POWER rows, five bias rows, 30 scratch rows, six LFE rows, shared work, 36 unbound indicators, and untouched descriptor/arena tail canaries.
    - 向量说明：Package audit reports 12 cold-zero slots, 420 same-owner carry checks, and 0 mismatches; native reset/EOF/fatal remains NOT_RUN.
  - `selected-configuration-record-import`，状态 `LOCALLY_VERIFIED_SELECTED_SOURCE_RECORD_COPY_AND_DERIVED_METADATA__EXTERNAL_WRITER_OPEN`：Imports one fixed 2,752-byte selected configuration record into caller-owned 4,096-byte owner and 64-byte metadata buffers using the exact ordered copies, normalization, rate selector, derived metadata and status behavior. Prefix rejection is no-write; the compatible-rate error preserves partial writes. It does not implement the external job writer, generic metadata importer, or whole F09.
    - 向量说明：Seven source recipes reconstruct exact 2,752-byte input records from byte patches; expected owner and metadata are full post-import byte arrays.
    - 向量说明：Four success cases cover selected 5.1, stereo, mode1 compatibility, and mode4 compatibility-class behavior.
    - 向量说明：Two tag-prefix failures return 140 with unchanged owner and metadata.
    - 向量说明：The compatible unsupported rate returns 59 after earlier owner/metadata writes; tests compare the retained partial mutation byte-for-byte without rollback.
    - 向量说明：Return 161 channel-map logic is implemented from the static contract and has only a static-derived formula test; the package contains no controlled rejecting runtime vector.

## F09E

- 功能范围：`complete exponent refresh difference32 integer mathematical behavior provider`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：无；依赖：无。
- 规格身份：`specs/09-exponents.md` SHA-256 `41cd157140d0deaab51fa9a895c33758cb49eb46cf5844a2883c1d6e8bb81739`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/refresh_difference.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `refresh-difference-index-map`，状态 `LOCALLY_VERIFIED_D06E_INTEGER_INDEX_MAP__WHOLE_F09E_DEFERRED`：Pure strict i16 index-to-I16 D06-E map for indices 0..=31. It does not perform full exponent refresh, coefficient-plane processing, F10 normalization, BAP allocation, frame planning, or policy-provider loading.
    - 向量说明：All 32 valid indices 0..=31 and exact I16 outputs.
    - 向量说明：The two typed i16 range rejections (-1 and 32) map to EXPONENT_INPUT_ERROR.
    - 向量说明：Five non-integer JSON rejection vectors remain package-bound but are not exposed through a new JSON/API layer.
    - 向量说明：The 32 outputs serialize as 64 I16LE bytes with the package-bound SHA-256.

## F09PA

- 功能范围：`authorized D06/D07 selected refresh/gain/db/normalization19 scalar provider and lifecycle vectors`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q05`；依赖：无。
- 规格身份：`specs/09-policy-authorized-parameters.md` SHA-256 `62a4b893d68001065e16693d9d489b845808320d3bc2ada98e569a47224a243d`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/policy_parameters.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `selected-policy-parameter-provider`，状态 `LOCALLY_VERIFIED_HASH_PINNED_POLICY_PARAMETER_PROVIDER__CONSUMER_ALGORITHMS_DEFERRED`：Identity-first loading and typed exposure of 19 selected-profile D06/D07 scalars only; no numeric consumer, state lifecycle, or whole F09 implementation.
    - 向量说明：19 values serialize to the reviewed 76-byte digest.
    - 向量说明：Two profile rejection recipes map to UNSUPPORTED_PROFILE with no state advance.
    - 向量说明：Missing provider maps to PARAMETER_UNAVAILABLE; corrupt provider identity/schema maps to POLICY_PARAMETER_ERROR.
    - 向量说明：17 frozen-model consumer comparisons are evidence only and are not implemented here.

## F09G

- 功能范围：`gain/db`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARAMETERIZED_RULES_DEFINED`；问题：`Q02`；依赖：`D07`。
- 规格身份：`specs/09-gain-and-db.md` SHA-256 `f6e73ca2e361c06413974f4b7725fdc94f8e1d13a6a299062edde9dc3a7e6c12`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PARAMETER_PROVIDER_DELIVERED_LOCAL_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：`U-DB-0-0.0`, `U-DB-0-1e-06`, `U-DB-0-0.01`, `U-DB-0-1.0`, `U-DB-1-0.0`, `U-DB-1-1e-06`, `U-DB-1-0.01`, `U-DB-1-1.0`, `U-DB-2-0.0`, `U-DB-2-1e-06`, `U-DB-2-0.01`, `U-DB-2-1.0`, `U-DB-3-0.0`, `U-DB-3-1e-06`, `U-DB-3-0.01`, `U-DB-3-1.0`, `U-MEAN-01`, `U-GAIN-01`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/src/analysis/mean_square.rs`, `crates/jocforge-domain/tests/db_selector.rs`, `crates/jocforge-domain/tests/gain_history_peak_state.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `precomputed-means-to-db-selector`，状态 `LOCALLY_VERIFIED_SIX_MEANS_TO_ONE_DB_CODE__GAIN_AND_FRAME_STATE_DEFERRED`：Consumes exactly six precomputed F32 means, prior U16 selector, and the selected policy-parameter receipt values to return one db code plus comparison diagnostics. It excludes mean-square, gain-peak, PCM, cross-block lifecycle, frame plan, and all other F09G consumers.
    - 向量说明：Four input-only cases cover six mean bit words and a prior selector, including zero, subnormal, fractional, and large finite means.
    - 向量说明：Expected maximum-mean word, polynomial level, signed difference, strict hysteresis decision, and output selector are compared after computation.
    - 向量说明：The LFE sixth mean is validated but excluded from the first-five maximum; ties in the maximum scan select the later lane.
    - 向量说明：The package review records 17 frozen-model numeric comparisons with zero mismatches; only its four db-selector cases apply to this direct slice.
  - `four-lane-mean-square-256`，状态 `LOCALLY_VERIFIED_256_SAMPLE_FOUR_LANE_MEAN_SQUARE__GAIN_AND_HISTORY_DEFERRED`：Consumes exactly 256 finite selected-profile F32 samples at stride 1 and returns one mean-square value. Maintains four local accumulators only; no cross-block state, gain peak/history, selector, delayed-PCM owner, or frame orchestration.
    - 向量说明：Seven recipes each carry 256 input U32 words, lane accumulation traces (4x64), final lane values, ordered five-value reduction trace, and mean output.
    - 向量说明：All 1,792 input words and 1,792 lane steps are recomputed from input only and checked against expected after computation.
    - 向量说明：The partition-order and reduction-order recipes discriminate global serial, balanced, and reversed alternatives.
    - 向量说明：Intermediate lane values are static-rule calculations, not native register captures; final means have seven runtime-confirmed values.
  - `gain-history-peak-state-pass0-gate1-format7`，状态 `LOCALLY_VERIFIED_PEAK_PREVIOUS_SUMMARY_Q15_GAIN_AND_REUSE_MIN__RESET_FILTER_AND_AUDIO_OWNER_OPEN`：Selected pass-0/summary-gate-1/format-7 six-channel/six-block gain history. Tracks previous peak, per-block summary, cumulative Q15 max, five full-band gain rows, and reuse-min projection. LFE contributes statistics only. Filter auxiliary effects, delayed input production, outer transactions, and lifecycle clocks are excluded.
    - 向量说明：17 history recipes compare 648 peak/state transitions and 540 full-band gain outputs across 165,888 input words.
    - 向量说明：Explicit aligned four-lane and unaligned scalar profiles are tested separately; signed-zero, ties, and horizontal operand order are checked.
    - 向量说明：Three reuse-min recipes compare 90 projected gains with preserved output tail canaries.
    - 向量说明：Derived lane/horizontal traces are input/static-rule calculations; native lane registers were not captured. Filter auxiliary numeric state remains excluded.

## F09LB

- 功能范围：`selected LFE gate producer and PRIMARY/LFE mantissa coefficient boundary`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`SELECTED_PROFILE_LOCAL_R2_REVIEW_PASS`；问题：无；依赖：无。
- 规格身份：`specs/09-lfe-gate-mantissa-boundaries.md` SHA-256 `9aff19192a8b5cba7db8bec26b05106edf214b4149a5f1c9d4be040691c80951`。
- 矩阵 aspect 状态：`DEFINED_LOCAL_FRESH_REVIEW_PASS_WHOLE_PACKAGE_NOT_READY`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`37_BOUNDARY_CASES_AND_9_SUPPLEMENTAL_CONNECTION_CHECKS_FRESH_REVIEW_PASS; WHOLE_PACKAGE_NOT_READY`。
- 本地定向测试：`crates/jocforge-domain/tests/lfe_mantissa_boundary.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `selected-lfe-gate-and-mantissa-boundary`，状态 `LOCALLY_VERIFIED_SELECTED_GATE_CLIP_EXPONENT_AND_STAGED_ROW__F10_NORMALIZER_DEFERRED`：Selected-profile gate zero producer, shared finite-F32 PRIMARY/LFE strict clip and raw-exponent scan, and atomic staged seven-word LFE row/exponent commit, retire, and reset seam. F10.4 LFE normalization and full F09/F10 planning are excluded.
    - 向量说明：13 finite coefficient words: signed zero, subnormal, ±1 equality, adjacent clip values, and maximum finite values
    - 向量说明：Primary and seven-word LFE routes share the same clip/exponent primitive
    - 向量说明：Shape/nonfinite rejections for NaN, infinity, wrong LFE count, and PRIMARY lengths 0/253/254
    - 向量说明：Selected profile gate=0; unsupported AHT, block-count, and sample-rate profiles reject
    - 向量说明：Two seven-word staged source cases cover zero commit on downstream failure, retirement, reset retry, and one successful row+exponent commit
    - 向量说明：F10 normalizer cases are retained in the fixture but explicitly excluded from the implemented/tested slice

## F10

- 功能范围：`exponent normalization`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARAMETERIZED_RULES_DEFINED`；问题：`Q02`；依赖：`D06-N`。
- 规格身份：`specs/10-exponent-normalization.md` SHA-256 `a538f0af5c70c8e8f4e4b5212438a1e5abd304c3fb2efa7165617014a55fc5fc`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PARAMETER_PROVIDER_DELIVERED_LOCAL_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：`U-GROUP-01`, `U-GROUP-02`, `U-GROUP-03`, `U-LFE-6`, `U-LFE-7`, `U-OPT-GATE-0`, `U-OPT-GATE-1`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/exponent_normalization.rs`, `crates/jocforge-domain/tests/exponent_packing.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `fullband-exponent-row-normalization`，状态 `LOCALLY_VERIFIED_SIX_HASH_BOUND_ROWS__WHOLE_F10_OPEN`：Implements the washed F10.2-F10.4 transformation for one caller-supplied full-band I16[217] raw row, span, option, gate, and optional hash-pinned D06-N provider. Four constant rows match the root package results; two authored mod25/zigzag24 recipes match the fresh current-spec supplement. Recipe expansion exists only in tests. It does not merge six-block reuse intervals, normalize LFE rows, serialize exponent digits, own a source-frame clock, or close F09/F10.
    - 向量说明：Four `constant(level)` raw recipes are reconstructed as I16[217] rows using the full-band row length specified by F10.
    - 向量说明：Each case compares base, expanded I16LE digest, initial sum, first-pass square error, shifted error, grouped count, scaled-count F32 word, threshold, and second-pass decision.
    - 向量说明：The supplement defines `mod25` as `(7*i+3)%25` and `zigzag24` as `t` for even `q`, otherwise `24-t`, with `q=i//25` and `t=i%25`, for every i in 0..216. Tests independently reconstruct each row, assert its input-byte digest, then compare all 217 computed output words.
    - 向量说明：Current-spec review and Wash replay passed these two authored recipe cases with 434 native output words and zero mismatches. These are test recipes, not natural exponent producers.
    - 向量说明：The root r4 review still binds an older F10 spec SHA. The supplement review covers only the two authored F10.2-F10.4 replays and does not close whole F10 or establish a natural producer.
  - `exponent-difference-triple-packing`，状态 `LOCALLY_VERIFIED_F10_5_TRIPLE_PACKING__CURRENT_SPEC_REVIEW_OPEN`：Packs one to three final exponent differences in -2..=2 into the F10.5 seven-bit base-five group value. Missing tail digits use digit 2. It owns no exponent row, reuse interval, frame clock, or bitstream writer.
    - 向量说明：The input fixture binds the current F10 spec SHA and provides six partial-group recipes plus four invalid shapes without expected output fields.
    - 向量说明：The test enumerates all 125 complete three-digit groups in the declared difference domain and checks the F10.5 base-five expression, output range, and injectivity.
    - 向量说明：The test checks one- and two-digit tails against F10.5 digit-2 padding and rejects empty, overlong, and out-of-range groups.
    - 向量说明：The r4 policy review binds an older F10 spec SHA, so the tests are local formula checks against the current washed spec, not evidence of independent review of that spec.

## F11

- 功能范围：`virtual converter`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARTIAL`；问题：`Q09`；依赖：`D13`。
- 规格身份：`specs/11-converter.md` SHA-256 `9ecb50bbb125d2a6e2d8f5e565ffbc7514c156c72602d2d2d89d8b17e9e4e4c8`。
- 矩阵 aspect 状态：`PARTIAL_OR_NOT_EVALUATED`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：`U-COST-0`, `U-COST-2`, `U-SEARCH-01`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F12

- 功能范围：`main allocation/wire`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARTIAL`；问题：`Q09`, `Q10`；依赖：`D13`。
- 规格身份：`specs/12-codec-and-wire.md` SHA-256 `3e712f4d8bd0d0eaccd933ce0087a4fc0fbd73e3ca92b4dc673153786936a14e`。
- 矩阵 aspect 状态：`PARTIAL_OR_NOT_EVALUATED`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/grouped_quantizer.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `grouped-quantizer-helper`，状态 `LOCALLY_VERIFIED_GROUPED_QUANTIZER_HELPER__WHOLE_F12_DEFERRED`：Internal-only valid BAP1/2/4 grouped quantizer helper. It reserves item widths, compacts completed groups in place, and flushes only existing reservations. It does not implement caller capacity ownership, BAP values outside the selected domain, higher retry/rollback, producer identity, or full frame/EC3 behavior.
    - 向量说明：R18 covers valid-domain item, grouping, flush, writer-state, pending-accumulator, and byte-hash observations.
    - 向量说明：R15 covers the supplied logical-capacity observation only; no caller-capacity ownership or overrun contract is inferred.

## F12S

- 功能范围：`selected 48k1024k5.1 full codec/header/block/mantissa/search/keyless-tail/CRC route`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q09`, `Q10`；依赖：无。
- 规格身份：`specs/12-selected-codec-wire.md` SHA-256 `9809b6b64c1ca4702640d2b15b1c2efb4e8a3c67ae1962f69a44ad7892a29448`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/mantissa_quantizer.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `mantissa-quantizer`，状态 `LOCALLY_VERIFIED_STATELESS_MANTISSA_QUANTIZER__WHOLE_F12S_DEFERRED`：Pure finite binary32 word plus BAP to mantissa code or candidate-local error for F12S.5. Input is already exponent-scaled; no transform, fit, exponent/BAP producer, grouping, trial search, bitstream/frame, programme state, or source/AU clock is implemented.
    - 向量说明：208 exact quantizer cases: 204 code outputs and 4 BAP 3/5 reserved-sentinel failures
    - 向量说明：All selected BAP values 0..15 are exercised across the delivered finite F32 words
    - 向量说明：Local boundary cases reject non-finite inputs, unsupported BAP, and non-finite intermediates

## F12SC

- 功能范围：`selected F07-F12 per-source frame transaction composition`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q05`, `Q09`, `Q14`；依赖：无。
- 规格身份：`specs/12-selected-source-frame-composition.md` SHA-256 `9e6fcc14489ca0acaff0f7c223fce65539aa5b19cb4119ebcb34d19049cd0769`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F12A

- 功能范围：`fixed-point BAP`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARAMETERIZED_RULES_DEFINED`；问题：`Q02`, `Q09`；依赖：`D13`。
- 规格身份：`specs/12-bit-allocation.md` SHA-256 `fa347045793bf3ce88349052526a758bc47e9d33fb1f1a1265a50c249ccd45f7`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PARAMETER_PROVIDER_DELIVERED_LOCAL_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F13

- 功能范围：`EMDF/timeline`。
- 本地实现状态：`LOCALLY_VERIFIED_SELECTED_SOURCE_PROPOSAL_AND_RECEIPT__GLOBAL_TRANSACTION_BLOCKED`。
- 洗白规格状态：`PARTIAL`；问题：`Q08`；依赖：`D11`。
- 规格身份：`specs/13-emdf-and-timeline.md` SHA-256 `0ddf3e52856b97e505f83576b684e235b21bdacf207de354b8e8b12182dca6fd`。
- 矩阵 aspect 状态：`PARTIAL_OR_NOT_EVALUATED`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：`U-EMDF-01`, `U-EMDF-DISCARD-01`, `U-CARRIAGE-COMPAT`, `U-CARRIAGE-ETSI`, `U-CARRIAGE-UNKNOWN`, `U-CARRIAGE-NULL`, `U-CARRIAGE-BOOL`, `U-CARRIAGE-DEFAULT`, `U-FIFO-3072`, `U-FIFO-4608`, `U-FIFO-2048`, `U-FIFO-PHASE`, `U-FIFO-OCCUPIED`, `U-FIFO-CAPACITY`, `U-HALFDB-0`, `U-HALFDB-1`, `U-ID1-TIMER-SYNTH`, `U-ID1-TIMER-ABSENT`, `U-ID2-ENUMS`, `U-ID2-LFE`, `U-ID2-NO-LFE`, `U-ID2-EXTENDED`, `U-ID2-VERSION`, `U-ID2-SHORTCUT`, `U-ID2-BITWISE-AND`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/src/joc/emdf/metadata_owner/tests.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `selected-metadata-owner-source-stage`，状态 `LOCALLY_VERIFIED_FOUR_QUEUE_TIMER_EMDF_AND_SOURCE_RECEIPT__GLOBAL_CALLBACK_BLOCKED`：Selected 48 kHz/1536-sample profile only: private timer and four-queue state, local source receipt, source2+ EMDF proposal, and EOF leaf proposal. No producer-side ID11/ID2/ID14 algorithms, complexity BSI, global callback receipt, or durable output.
    - 向量说明：5 source calls
    - 向量说明：14 failures
    - 向量说明：EOF
    - 向量说明：reset/retry
    - 向量说明：cold and source2 terminal fixtures
    - 向量说明：occupied destination 9/9

## F13F

- 功能范围：`metadata FIFO`。
- 本地实现状态：`LOCALLY_VERIFIED_SELECTED_FOUR_QUEUE_PUSH_POP__OTHER_PRODUCERS_DEFERRED`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q08`, `Q14`；依赖：无。
- 规格身份：`specs/13-emdf-and-timeline.md` SHA-256 `0ddf3e52856b97e505f83576b684e235b21bdacf207de354b8e8b12182dca6fd`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：`U-FIFO-3072`, `U-FIFO-4608`, `U-FIFO-2048`, `U-FIFO-PHASE`, `U-FIFO-OCCUPIED`, `U-FIFO-CAPACITY`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/src/joc/emdf/metadata_owner/tests.rs`。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F13T

- 功能范围：`ID1 timer/wire`。
- 本地实现状态：`LOCALLY_VERIFIED_17_TIMER_PRIMITIVES_AND_40_ID1_BODY_CALLS__MEASUREMENT_DEFERRED`。
- 洗白规格状态：`PARAMETERIZED_RULES_DEFINED`；问题：`Q07`, `Q08`, `Q14`；依赖：`D11-TIMER`。
- 规格身份：`specs/13-emdf-and-timeline.md` SHA-256 `0ddf3e52856b97e505f83576b684e235b21bdacf207de354b8e8b12182dca6fd`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PARAMETER_PROVIDER_DELIVERED_LOCAL_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：`U-HALFDB-0`, `U-HALFDB-1`, `U-ID1-TIMER-SYNTH`, `U-ID1-TIMER-ABSENT`。
- 洗白包覆盖说明：`LOCAL_REVIEW_STATUS_FILE_PRESENT; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/src/joc/emdf/metadata_owner/tests.rs`, `crates/jocforge-domain/src/joc/emdf/metadata_owner/timer/timer_vectors.rs`。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F13C

- 功能范围：`optional EMDF carriage modes`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q08`, `Q11`；依赖：无。
- 规格身份：`specs/13-emdf-and-timeline.md` SHA-256 `0ddf3e52856b97e505f83576b684e235b21bdacf207de354b8e8b12182dca6fd`；`specs/13-joc-complexity-timer-payload-order.md` SHA-256 `09bcff06bf48d6bd6c28fcd2a3748229ee254f762576df45a24ef12729f4381b`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：`U-CARRIAGE-COMPAT`, `U-CARRIAGE-ETSI`, `U-CARRIAGE-UNKNOWN`, `U-CARRIAGE-NULL`, `U-CARRIAGE-BOOL`, `U-CARRIAGE-DEFAULT`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-application/tests/input_eof_lifecycle.rs`, `crates/jocforge-domain/src/joc/emdf/configuration_writer.rs`, `crates/jocforge-domain/src/joc/emdf_carrier.rs`, `crates/jocforge-domain/tests/f13c_additional_bsi.rs`, `crates/jocforge-domain/tests/id11_parser_boundary_evidence.rs`, `crates/jocforge-domain/tests/id14_coefficient_payload_writer.rs`, `crates/jocforge-domain/tests/washed_f13c_scope_evidence.rs`, `crates/jocforge-io/tests/reader_input_padding.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `selected-bsi-count-and-payload-import`，状态 `LOCALLY_VERIFIED_SELECTED_COUNT_AND_COPY__PREFIX_BYTES_ID14_AND_PARENT_OPEN`：Selected ordinary non-bypass wrapper only: derive configured count from rate code 0..=5 and signed-I16 channel labels, model the validated configuration-count status/slot projection, produce U16 enable/length plus payload bytes 0/1, and copy all 64 payload bytes into a typed importer projection. The observed prefix-write fact is recorded, but unavailable header/selector prefix bytes are not synthesized. This selected producer does not parse ID14: a separate frozen model/helper does parse ID14 and computes object_count+1, but that model fact does not supply the selected native BSI count, which comes from validated configuration count. Native malformed/full-container semantic ID14 parser calls remain NOT_RUN. No ID14 body implementation, bypass path, multi-substream writer, or whole F13C/F13 is implemented.
    - 向量说明：Three ordinary-wrapper cases reproduce counts 16 (rate5 plus LFE), 12 (rate0 plus LFE), and 15 (rate5 without LFE), then compare both produced and imported 64-byte payloads.
    - 向量说明：Seven configuration-count cases cover absent default 11, valid endpoints 1/16, and invalid -1/0/17/272 with native return 220, unchanged count slot, and observed prefix mutation. The exact prefix bytes are unavailable and are not modeled.
    - 向量说明：Two validated pipeline cases cover explicit counts 1/16, producer return 0, U16 enable=1, U16 length-minus-one=1, and literal 64-byte import copy with preserved tail.
    - 向量说明：Validation-bypassed low-byte probes are diagnostic only and are not passed to the production helper.
    - 向量说明：R2 supplies two fresh private-owner cases with counts 1/16 and absent ID14 queue/serializer/body holder before and after. This proves no encoded-ID14 dependency for the selected BSI count producer only; it is not a global parser-absence claim.
    - 向量说明：The hash-bound clarification package separates the frozen helper's object_count+1 calculation from the selected native BSI producer; 13 helper records are not fresh parser execution.
    - 向量说明：The base Section 1 EMDF/ID14-derived count claim is superseded by the correction overlay. The selected carrier transport is separately mapped below; ID14 body generation/serialization, bypass, and parent F13C remain open.
  - `selected-emdf-carrier-planner-reservation-writer-and-publication-gate`，状态 `LOCALLY_VERIFIED_18_PRIMITIVES_FOUR_NATIVE_DIAGNOSTIC_AU_AND_SIX_CALL_GATE__WHOLE_F13C_OPEN`：Selected ordinary 48 kHz, 1024 kbps, one independent substream only. Implements the six-block signed-length planner with retained partial mutation on residual return, a present/9-bit reservation writer, first-fragment marker and body-length prefix, raw continuation fragment writer over a SerializedEmdfBodyFragment typed view, and the strict outer publication gate. The typed view does not attest provenance and this slice is not wired into production encode; production callers must supply a real model-owned body/provider receipt. Fixture body bytes are native diagnostic inputs, not model-produced bytes. Gate AU ordinals are distinct from frontend/JOC invocation ordinals and are assigned only for nonzero caller-visible output.
    - 向量说明：All 18 controlled primitive cases compare planner return/state/segments and, when admitted, complete 544-byte fragment buffers including untouched bits and guard bytes. These synthetic writer bytes use body[i]=(37*i+11) mod 256 over initial 0xA5 buffers. Cases 2040/2041/2551/2552 are primitive transport reachability only.
    - 向量说明：The two residual failures (3063 bytes and 3055 bytes at period wrap) retain the observed counter, enabled/first-block and partial segment mutation; writer execution is not inferred after planner failure.
    - 向量说明：The four body inputs are native diagnostic boundary vectors, not model-produced bytes. For each AU, the model writes the reservation and carrier fragment at the washed native bit cursor, then compares only those carrier bits with the caller-published frame oracle.
    - 向量说明：The six gate calls suppress the first two internal outputs and assign published AU ordinals 0..3 only to calls 3..6, joining JOC producer calls 2..5.
    - 向量说明：The package explicitly leaves protected body production/authentication, ordinary codec frames, ID14 semantic parsing, whole-parent rollback, source EOF/reset, absolute ADM/host clocks, production encode wiring, and whole F13C/F13 open.
  - `selected-input-eof-lifecycle-observation-replay`，状态 `LOCALLY_VERIFIED_READER_SAMPLE_COORDINATE_JOIN_AND_FOUR_PROCESS_FIVE_FLUSH_REPLAY__WHOLE_F13C_OPEN`：Test-only application observation replay for one selected reader/source-position capture and the selected F13C process/flush/EOF/close sequence. It consumes washed control-record values, reader cursors and lengths, observed renderer spans, and diagnostic planner/pending-byte extents only inside the test harness; it does not export a production API, perform PCM/renderer work, generate protected body bytes, or wire encode. The selected sample coordinate is item start plus the relative reader cursor before that frame. It does not establish an absolute file byte offset, host clock, general seek rule, or pointer alias between constructor and CompositeMaster descriptors. The separate pure carrier primitives may accept a typed body slice, but a selected product caller still needs a real model-owned body/provider receipt. Same-owner reset/recreate remains NOT_RUN.
    - 向量说明：Four external process calls use 38 indexed control records each. Process calls 1/2 run renderer spans of zero and do not call frontend; calls 3/4 carry frontend calls 1/2.
    - 向量说明：Flush calls 1..4 are active and each returns 4096 bytes after one child source step. Flush call 5 is empty and preserves its flush_started flag without advancing frontend, planner, or renderer work.
    - 向量说明：The fixture drives planner-request inputs 0,0,422,316,320,316 and pending ID14 extents 132,326,222,226,222,218. These are diagnostic inputs; the test creates no body, descriptor bytes, or PCM.
    - 向量说明：The one selected reader item spans source sample coordinates 2880000..2886000. Relative API/cursor positions 0/1536/3072/4608 map to selected source coordinates 2880000/2881536/2883072/2884608 for this capture only.
    - 向量说明：The final read_float actual length is 1392 while the packet reports 1536. No PCM values or padding words are exported by this reader/source package; the positive-zero tail is covered separately by the selected reader-padding provider slice. Descriptor linkage is value equality, not pointer alias. No absolute byte offset, host clock, or general seek rule is claimed.
    - 向量说明：Post-flush process returns raw U32 4294967293 / I32 -3, consumed=1, output length=0, with no state change. Empty flush cases for flag 0 and 1 both return 0/length 0 and preserve the flag.
    - 向量说明：At EOF the scalar projection retains gate progress 4608, planner counter 6/first block 2, pending extent 218, flush_started=1 and flush_remaining=0. Close returns 0 and removes runtime; scalar projection is captured before workspace retirement, with no post-free read and no semantic queue-clear claim.
    - 向量说明：Carrier join checks map published AU0..3 to frontend calls 3..6 and JOC producer calls 2..5. Whole F13C remains deferred; absolute control-position mapping and same-owner reset remain NOT_RUN.
  - `selected-reader-partial-frame-input-padding`，状态 `LOCALLY_VERIFIED_1392_TO_1536_POSITIVE_ZERO_INPUT_TAIL__NO_DSP_PADDING`：Selected 38-channel, 48 kHz reader/provider input boundary only. A fresh 38x1536 row table is initialized to positive-zero words before the opaque 38x1392 valid prefix is copied bit-for-bit. The remaining 144 words per channel are positive zero. This helper does not read files or inspect prefix values, is not wired into DSP/renderer output, and does not establish whole F13C or numeric valid-prefix acceptance.
    - 向量说明：Synthetic opaque prefix words are used only to verify bit-preserving copy; they are not PCM vectors or native valid-prefix numeric acceptance.
    - 向量说明：The output has 38 rows x 1536 words; each row retains the first 1392 input words and appends 144 positive-zero words. Aggregate tail count is 5472 words / 21888 bytes.
    - 向量说明：Native statistics observed 1747 positive zeros and 3725 finite nonzero words before clear; 11 aggregate observations after clear/read/interleave/API/renderer input report 5472 positive zeros.
    - 向量说明：Renderer/DSP output tail is generally nonzero. No DSP output is passed to this provider helper or zeroed by this rule.
    - 向量说明：PCM values/content digests, valid-prefix equality, padding byte contents beyond aggregate bit classification, other profiles, and whole F13C remain unverified.
  - `selected-id11-parser-boundary-evidence`，状态 `LOCALLY_VERIFIED_WASHED_BOUNDARY_RECEIPTS_ONLY__NO_ID11_PARSER_IMPLEMENTATION`：Verify the sealed selected-seed ID11 parser-boundary input/observation receipt is hash-bound and internally self-consistent. This is evidence custody and boundary mapping only: it does not implement or emulate an ID11 semantic parser, generalize the 69-byte seed, attribute generic collector behavior to ID14, or claim parent rollback.
    - 向量说明：72 controlled selected-seed cases: prefixes 0..69 plus one trailing 00 and one trailing FF case
    - 向量说明：Prefixes 0..68 fail; exact 69-byte seed succeeds after 546 semantic bits and 6 final-alignment bits
    - 向量说明：Trailing 00 and FF are accepted with 8 bits remaining; no general trailing-byte policy is claimed
    - 向量说明：69 failures: 2 owner-equal-cold and 67 owner-changed; allocation equality does not prove rollback
    - 向量说明：WASH-VALIDATION notes the separate 9-case generic collector receipt; this test does not replay or attribute collector behavior to ID11/ID14
  - `selected-configuration-writer-and-descriptor-bit-budget`，状态 `LOCALLY_VERIFIED_SELECTED_CONFIG_BITS_AND_DESCRIPTOR_BUDGET__PARENT_TRANSACTION_OPEN`：Implements the selected typed F13.3 ten-field configuration projection, conditional MSB-first configuration writer, and bounded descriptor bit budget. Nonrepresentable typed values return None before writer output creation and map to the generic local INPUT_REJECTION/LOCAL_TRANSLATION outcome. No native status, ID body production, whole-container failure, parent rollback, or product encode is claimed.
    - 向量说明：Selected ID11, ID14, ID2 and ID1 configuration bit strings match R18 native-expected cases.
    - 向量说明：Approved sample-offset, duration, group-id, priority and processing ranges reject out-of-domain typed values before output creation.
    - 向量说明：Wash R18 package independently replayed 29 configuration/descriptor boundary cases with zero mismatches.
    - 向量说明：512-byte descriptor capacity retains the complete local bit prefix; larger capacity completes the bounded body write.
  - `id14-model-parser-and-native-bsi-source-clarification`，状态 `WASHED_EVIDENCE_CUSTODY_ONLY__NO_PARSER_IMPLEMENTATION`：Records the hash-bound frozen model/helper ID14 parser and object_count+1 result separately from the selected native BSI producer, whose validated configuration-count source is present with ID14 storage absent in selected cases. The 13 vectors are model observations, not fresh parser execution or native semantic-parser evidence. R22 separately establishes that the recovered selected outgoing encode path has no ID14 semantic-reader invocation; this does not establish global parser absence or whole F13C readiness.
    - 向量说明：13 recorded model-helper cases; debug/release recorded outputs agree, but the checker does not run a fresh parser execution.
    - 向量说明：Selected native validated configuration counts 1 and 16 reach the BSI complexity field while ID14 body/queue/serializer storage is absent.
    - 向量说明：The selected model's object_count+1 value equals 16 numerically but is not the selected native BSI producer source.
    - 向量说明：R22 closes parser applicability only for the recovered selected outgoing encode path; other DLL modules, profiles, indirect entry points and decoders remain outside scope.
  - `selected-id14-semantic-reader-path-applicability`，状态 `WASHED_CONTROL_FLOW_EVIDENCE_CUSTODY_ONLY__NO_ID14_PARSER_IMPLEMENTED`：The recovered selected outgoing encode path produces ID14, routes it through generic descriptor/queue-copy handling, and serializes its body as opaque bytes. Only the ID11 ingress branch invokes the semantic reader family. Native ID14 semantic malformed behavior is NOT_APPLICABLE to this selected outgoing path; this does not claim parser absence elsewhere in the DLL or other entries.
    - 向量说明：70 bounded native candidates and 18 semantic-reader-family functions are rooted at the ID11 semantic reader family.
    - 向量说明：Selected outgoing ID14 is produced, generically queued, and serialized as an opaque descriptor body; no semantic ID14 reader invocation is present in the recovered path.
    - 向量说明：The four descriptor/serializer lineage cases rehash prior R14 observations; they are not new dynamic captures or parser tests.
    - 向量说明：No malformed ID14 parser vectors are fabricated; applicability is NOT_APPLICABLE only for this recovered selected outgoing path.
    - 向量说明：The frozen model helper's semantic parser and object_count+1 behavior remain a separate model fact from the selected native configured-count BSI source.
  - `container-fragmentation-scratch-parent-observation`，状态 `WASHED_RECEIPT_CUSTODY_ONLY__PRODUCTION_COLD_NOT_ESTABLISHED`：Records 16 bounded scratch-parent cases and two separate raw-reservation controls. The observed 512-byte container split into 511/5-byte transport fragments and empty-provider failure changed some scratch state with zero published bytes. This is not production-cold equivalence, does not establish semantic carrier_block, and does not supersede approved F13.1.
    - 向量说明：8 public-provider positive and 8 empty-provider negative scratch-parent cases, plus two raw-reservation controls.
    - 向量说明：A 512-byte scratch container was observed as 511-byte and 5-byte transport wire fragments.
    - 向量说明：Empty-provider child/parent returned 9/7 with zero published bytes while some scratch state changed.
    - 向量说明：Normal and optimized receipts have zero mismatches; fragment contents were not independently reproduced.
    - 向量说明：The approved selected-production F13.1 length rule remains unchanged pending semantic carrier_block and same-path evidence.
  - `selected-production-cold-root-observation`，状态 `WASHED_COLD_ROOT_CUSTODY_ONLY__FULL_PARENT_EQUIVALENCE_NOT_ESTABLISHED`：Records the selected top-constructor and parent/frontend identity joins plus a separate reconstruction of five queue-control initializers. This is not a complete production cold-owner reconstruction, full codec equivalence, or failure-injection result. R21 later clarifies the observed ID11 field 2103 as per-descriptor native queue capacity, separate from the approved 160-byte public API limit; natural selected producer output above 160 remains unproven.
    - 向量说明：Selected actual top constructor and its top/parent identity are joined to frontend cold initialization and six parent/frontend invocations; compatibility-branch construction was not used by this selected run.
    - 向量说明：Normal and optimized receipts each report five queue constructors, six frontend calls and six parent-boundary replays with zero mismatches.
    - 向量说明：The 221,221,0,0,0,0 assembly results and tolerance 2-to-0 are natural priming observations, not injected fatal outcomes.
    - 向量说明：Five independent scratch queue-control roots are reconstructed within captured fields only; no byte-exact full codec owner equivalence is claimed.
    - 向量说明：R21 clarifies ID11 constructor capacity_bytes=2103 as per-descriptor native queue capacity, not the F13.4 public API limit160; generic ID6 capacity=3 is distinct from ID2 capacity=8.
    - 向量说明：Durable file commit is NOT_OBSERVED; production failure injection is NOT_RUN; global rollback is not proven.
  - `id11-native-queue-capacity-and-public-policy-boundary`，状态 `WASHED_QUEUE_BOUNDARY_EVIDENCE_CUSTODY_ONLY__PRODUCER_OVER_160_NOT_PROVEN`：R21 resolves selected ID11 capacity_bytes=2103 as a U32 per-descriptor native queue admission limit derived for O=16,F=8. The separately approved public API payload cap remains160 bytes and continues to reject at preflight. R23 observes only 69-byte natural output across the frozen primary-metadata model and a bounded native secondary run; this supports keeping160 but does not establish a universal all-metadata/profile bound. The observed 2104-byte ingress silent-drop quirk is evidence only and is not implemented as product behavior.
    - 向量说明：For selected O=16,F=8, native ID11 generic queue admits per-descriptor payloads through 2103 bytes; 2104 returns queue status5 before copy/publication.
    - 向量说明：The approved F13.4 160-byte limit is a separate narrower public API policy; native admission observations at 161 bytes do not alter that policy.
    - 向量说明：R23 records 4249 natural 69-byte frozen-model payloads and 69 natural 69-byte secondary native payloads with no observed output above160; this supports the policy only for the sampled scopes.
    - 向量说明：With a 69-byte seed plus zero tail at supplied length2104, parser consumes552 semantic/alignment bits, queue returns5, wrapper returns0 and marks owner-valid while leaving queue empty; this silent-drop behavior is not recommended or implemented.
    - 向量说明：Generic ID6 queue limit3 maps a four-byte push to group status3; ID6 identity remains unresolved and differs from dedicated ID2 capacity8.
    - 向量说明：Normal and optimized checkers cover18 cases each with zero mismatches; they do not implement ID11 semantic parsing.
  - `selected-type11-natural-output-length-bound`，状态 `WASHED_LENGTH_AND_IDENTITY_RECEIPT_CUSTODY_ONLY__NO_PRODUCER_OR_PCM_RERUN`：R23 records 4249 naturally generated 69-byte Type11 outputs across a frozen selected primary-metadata model run and 69 naturally generated 69-byte outputs in a bounded native secondary run. The samples support retaining the approved 160-byte public preflight, but do not establish a universal limit across all metadata/profile inputs. Keep natural length69, producer buffer375, native queue capacity2103 and public policy160 distinct.
    - 向量说明：Frozen selected primary-metadata model run covers 4251 callbacks: 4249 fresh natural outputs, 3735 unique metadata inputs, each payload length69 bytes and every fresh payload byte-matching the archive.
    - 向量说明：Bounded native secondary run covers 71 source invocations and 69 naturally emitted payloads, all69 bytes with same-generation producer/formatter/frontend/ingress/queue-copy identity receipts.
    - 向量说明：No natural output above160 appears in the sampled scopes; this supports retaining the approved public API policy but does not prove a universal all-profile/all-ADM bound.
    - 向量说明：Standalone length checker does not regenerate all 4249 payload byte strings; their fresh-write/archive equality is bound by execution receipt and per-output observations.
    - 向量说明：Natural length69, producer buffer375, R21 queue capacity2103 and approved API policy160 remain separate quantities; F13C and product acceptance remain NOT_READY.
  - `controlled-id14-coefficient-to-payload-writer`，状态 `LOCALLY_VERIFIED_15_SYNTHETIC_CASES_588_CODEBOOK_SYMBOLS__UPSTREAM_AND_PARENT_OPEN`：Implements the controlled selected input slice from explicit native-domain F32 coefficient values through quantization, full/sparse Huffman body, ID14 header/bitstream and an owned descriptor copy. The explicit clip byte remains caller input; this does not implement the native peak/hold clip producer, upstream fitting/audio path, writer retry, production owner, parent transaction or durable publication.
    - 向量说明：All 15 controlled self-owned synthetic cases are checked; six approved Huffman tables contain 588 symbols and the local approved-codebook bytes match the supplement byte-for-byte.
    - 向量说明：Selected downmix3/five-channel, 15-object, band5, full-fine, one-point, clip128, sequence0 case yields 150 header bits plus 900 body bits, total1050 bits and 132 bytes.
    - 向量说明：The remaining cases cover all four coefficient modes, mixed presence, explicit clip/sequence limits, seven-channel sparse mode, absent objects, output capacity edges, scratch overflow and offset boundaries.
    - 向量说明：Successful payload hashes match the frozen input-first model postcompare, and successful output is copied to a distinct owned ID14 descriptor-cache buffer.
    - 向量说明：Native offset24 may return writer success with status false; the selected safe API rejects the invalid offset. Writer failures expose no local payload/descriptor, without claiming native prefix rollback.

## F13D2

- 功能范围：`ID2 profile/configuration/header and wire`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q07`, `Q08`, `Q14`；依赖：无。
- 规格身份：`specs/13-id2-profile-metadata.md` SHA-256 `35aa89d0dd67fb0133c4898c1007c84d48b39c5df6683dd88a173600975b48cc`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：`U-ID2-ENUMS`, `U-ID2-LFE`, `U-ID2-NO-LFE`, `U-ID2-EXTENDED`, `U-ID2-VERSION`, `U-ID2-SHORTCUT`, `U-ID2-BITWISE-AND`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/tests/id2_wire.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `id2-profile-header-and-body-wire`，状态 `LOCALLY_VERIFIED_ID2_PROFILE_HEADER_AND_BODY_WIRE__PROGRAMME_PRODUCER_DEFERRED`：Stateless exact profile-label to ID2 U3-code mapping, strict local Type11 LFE-header read, and five-input ID2 body serialization to 21 valid bits/3 bytes. No live DRC producer, programme profile admission, complete Type11 validation, FIFO, EMDF container, source/AU clock, or programme transaction.
    - 向量说明：7 selected cases from id2-profile-enums, id2-profile-from-header, and id2-field-wire
    - 向量说明：six exact profile-name mappings plus unknown-name rejection
    - 向量说明：Type11 LFE/no-LFE/extended-count success and invalid-version/shortcut rejection
    - 向量说明：21-bit, three-byte serializer output with integer bitwise-AND distinction

## F14

- 功能范围：`HMAC/CRC`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARTIAL`；问题：`Q10`, `Q11`；依赖：无。
- 规格身份：`specs/14-integrity.md` SHA-256 `7e9df3565d0ab25dad1ab4989de57003d399a0e839a5fb298a6bfc53b240894f`。
- 矩阵 aspect 状态：`PARTIAL_OR_NOT_EVALUATED`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：`U-COMPACT-0-15`, `U-COMPACT-0-23`, `U-COMPACT-0-24`, `U-COMPACT-4-7`, `U-PROTECT-01`, `U-HMAC-RFC4231-1`, `U-CRC-01`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F14K

- 功能范围：`public CRC32/CRC8 protection profile`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q11`；依赖：无。
- 规格身份：`specs/14-keyless-checksum-profile.md` SHA-256 `830a8a9b92a5819a5fde3363f0667bf8da204c277690c8289be439de85371382`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, constants, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior
- 洗白包 unit-vector ID：`U-CARRIAGE-COMPAT`, `U-CARRIAGE-ETSI`, `U-CARRIAGE-UNKNOWN`, `U-CARRIAGE-NULL`, `U-CARRIAGE-BOOL`, `U-CARRIAGE-DEFAULT`。
- 洗白包覆盖说明：`PACKAGE_RULES_AND_VECTORS_PRESENT; DEDICATED_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：`crates/jocforge-domain/src/integrity/programme_owner/tests.rs`。
- 已实现的局部 slice 及父 ID 剩余范围：
  - `public-keyless-integrity-owner`，状态 `LOCALLY_VERIFIED_KEYLESS_PUBLIC_OWNER__PRIVATE_AUTH_BLOCKED`：Public keyless CRC profile and its programme-local integrity owner only. Private authentication, native receipt wire format, CallbackTransaction integration, and other owner roots are excluded.
    - 向量说明：5 continuous source calls with complete zero-container and frame-template bytes
    - 向量说明：10 exact failure mutations including SOURCE/DISPATCH/AU test-harness fault plans
    - 向量说明：cold canonical state and provider digest
    - 向量说明：attempt reset and new-attempt retry sequence
    - 向量说明：five-call EOF/finalize sequence with no extra flush output
    - 向量说明：20 independently checked facts; all_pass=true and expected_used_as_seed=false

## F15

- 功能范围：`EC3 publication`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`PARTIAL`；问题：`Q02`, `Q13`；依赖：无。
- 规格身份：`PIPELINE-AND-INTERFACES.md` SHA-256 `61e4f4a968c93a2dc074656db656cf757246a5f316a1ca06502ebc9d4d842e12`。
- 矩阵 aspect 状态：`PARTIAL_OR_NOT_EVALUATED`：inputs, outputs, state, constants, preconditions, numeric_semantics, error_behavior, configuration, acceptance
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`PACKAGE_RULE_OR_PROVIDER_PRESENT; DEDICATED_VECTOR_OR_LOCAL_REVIEW_NOT_REGISTERED; WHOLE_EXECUTION_DOWNSTREAM_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## F15P

- 功能范围：`selected keyless F13-to-F15 staging publication owner/durable append/suppression/EOF/finalize`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`RULES_DEFINED_MODEL_SUBSET`；问题：`Q10`, `Q11`, `Q14`；依赖：无。
- 规格身份：`specs/15-keyless-publication.md` SHA-256 `47beaa9d415d6d26eb8d6a7df63ca3d86514c37740618c3a041a46c7961b3a72`。
- 矩阵 aspect 状态：`BOUNDED_SYNTHETIC_OR_RESEARCH_CHECKS_ONLY_PRODUCT_NOT_RUN`：acceptance; `DEFINITION_PROVIDED_CONSUMPTION_AUDIT_PENDING`：inputs, outputs, state, preconditions, numeric_semantics, configuration; `LOCAL_REJECTION_OR_FAIL_STOP_DEFINED_NATIVE_ERROR_STATE_NOT_PROVED`：error_behavior; `SELECTED_PROVIDER_MANIFEST_DELIVERED_R1_LOCAL_REVIEW_PASS`：constants
- 洗白包 unit-vector ID：`KEYLESS-PUBLICATION-INPUT-VECTORS:2_MODES+8_SOURCES+4_DURABLE_AUS+FAILURE_RESET`。
- 洗白包覆盖说明：`LOCAL_ROOT_2_MODE_8_SOURCE_4_DURABLE_AU_SYNTHETIC_PASS; HISTORICAL_4247_ATTESTATION_ONLY; FRESH_4249_NOT_RUN`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。

## OWNJ

- 功能范围：`selected callback owner projection and J05 adapter`。
- 本地实现状态：`DEFERRED_DETAILED_MAPPING`。
- 洗白规格状态：`SELECTED_PROFILE_LOCAL_REVIEWS_PASS`；问题：无；依赖：无。
- 规格身份：`specs/00-selected-owner-projection-and-j05.md` SHA-256 `9ff01aca7b6be6e89511e4d493f9fc9e661a50e2f098065f4a32c3fed777542f`。
- 矩阵 aspect 状态：矩阵未给出 aspect 级记录。
- 洗白包 unit-vector ID：无。
- 洗白包覆盖说明：`OWNER_R6_130_OF_130; F03_COLD_R1_40_OF_40`。
- 本地定向测试：无。
- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。
