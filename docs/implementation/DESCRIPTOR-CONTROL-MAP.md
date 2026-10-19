# Descriptor control map

由 IMPLEMENTATION-MAP.json 生成，不得手工修改。

Mapping status: `MAPPING_COMPLETE`. Implementation readiness: `READY_FOR_IMPLEMENTATION_F04D_DESCRIPTOR_CONTROL_ONLY__WHOLE_RECEIVER_NOT_RUN`.

F04D is a receiver substage owned by `spatial_audio`. It selects descriptor control from the post-temporal0..2 scratch before temporal3, then applies the selected gain to six lanes after temporal3. The mode=1 QMF path, mode=0 fitting state, final-six tail, and selected-J05 projection remain separate consumers and owners.

P1.4-D2 validates provider identity and typed values only. The selected F04D lookup threshold comes from the QMF receiver functional provider; the mathematical threshold in the descriptor provider is a candidate that differs in 36 of 127 words and remains pending audio acceptance. No descriptor DSP or W04 audio acceptance is claimed.

The approved `f04-descriptor-control.v1` supplement binds six selected-provider success vectors, eight error cases, semantic U32-to-I64LE hold encoding, and six local outcomes. This mapping gate is implementation-ready only for the two-phase F04D descriptor-control slice; it does not claim that implementation has started or passed. Whole receiver, J05/F05JC, real ADM/listening, and W04 execution remain `NOT_RUN`.

## Machine-readable contract

```json
{
  "approved_descriptor_supplement": {
    "actual_literal_file": {
      "bytes": 3705450,
      "file": "F04D-DIRECT-ACTUAL-LITERALS.json",
      "sha256": "390cf469a1dde8a4fde9c79de2238f414ea041d6b64630cb669ca4ab26205ad0"
    },
    "api_file": {
      "bytes": 987,
      "file": "API-SCHEMA.json",
      "sha256": "459482727625de393f9b2299b76f76e21f0892ef978f783c2b1d8b4dbc610baa"
    },
    "base_package_manifest_sha256": "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a",
    "contract_sha256": "78a94254d47f13a7533e7b329d62f2a889e0f69f35da6307e98a9bc71ce781c6",
    "directory": "outputs/IMPLEMENTER_SUPPLEMENTS/F04-DESCRIPTOR-CONTROL-V1",
    "identity": "f04-descriptor-control.v1",
    "independent_check_file": {
      "bytes": 2063,
      "file": "INDEPENDENT-CHECK.json",
      "sha256": "01ec148eeab548eaafa608daea35c765583289a3c3ed4912508e77746ec3a4a2"
    },
    "input_vector_file": {
      "bytes": 8878,
      "file": "F04D-DIRECT-INPUT-VECTORS.json",
      "sha256": "c622899783197a953fb6c0bcd262b0e7b1a110d605e9301e27782e44cb247528"
    },
    "manifest_sha256": "40d67c32fdebf97fe8dc249486ce4cb655642146a2470e6bcad553f1639f9e04",
    "secrets_included": false,
    "spec_file": {
      "bytes": 8512,
      "file": "SPEC.md",
      "sha256": "ebb493ed913f5b00beddb20b4bb91b3470986ea94a116877a7f2e40314d7ccf2"
    },
    "status": "IMPLEMENTATION_SPEC_READY_WHOLE_RECEIVER_NOT_RUN",
    "whole_receiver_j05_real_adm_listening": "NOT_RUN"
  },
  "audit_scope": "Map F04D control inputs, provider selection, canonical state, callback timing, downstream consumers, vectors, errors, and the minimum opaque stage boundary. Do not implement descriptor DSP or any neighboring audio/JOC stage.",
  "blockers": [],
  "clock_contract": {
    "cached_drain": "Each actual cached-drain callback with a Ready Type10 packet runs F04D once. EOF adds no synthetic descriptor callback or flush.",
    "frontend_callback": "g is the enclosing zero-based FrontendCallbackIndex.",
    "generation_source": "No independent descriptor clock or counter; use the same parent g and receiver count r as the enclosing receiver callback.",
    "priming": "g=0/1 has no Ready Type10 receiver callback, so F04D does not run and descriptor_state does not advance.",
    "receiver_callback": "For g>=2, r=g-2 identifies one real receiver callback; F04D runs once within that callback and proposes the one descriptor-state transition in spatial_audio."
  },
  "consumer_boundaries": {
    "final_six": "Descriptor gain applies after temporal3 to six lanes; F04R then runs post-S09 delay, half-scale, and second-ring stages to produce final_six[6][1536]. F04D must not reverse-read final_six to derive peak, level, or index.",
    "forbidden_aliases": [
      "mode=1 receiver audio histories",
      "mode=0 fitting histories",
      "DescriptorState",
      "descriptor-gained audio",
      "final_six",
      "J05 projection"
    ],
    "mode0_fitting": "F05JC independently recomputes raw analysis from receiver before-state and the same generation's s09_input, owns separate five-lane mode=0 fitting delay/FIR state, and consumes this generation's descriptor after index as clip_index. It does not use descriptor-gained s09_return or final_six as carrier input, and it does not own DescriptorState.",
    "receiver_mode1": "QMF analysis runs on lanes [0,1,2,4,5]; only Ls/Rs lanes [4,5] run mode=1 delay/FIR/synthesis before F04D. Descriptor peak reads the resulting temporal0..2 receiver scratch, not QMF histories and not Type10 directly.",
    "selected_j05": "F04C alone assembles selected-j05-owner-receipt.v1 for g>=2 and F05JL only consumes that projection. J05 does not rerun F04D/F04R/F04RB; final_six is a downstream projection input, not F04D control input."
  },
  "control_contract": {
    "gain_ramp": "F04D.4 uses gain[index-128]. Samples 0..255 share one callback-wide current ramp across all six lanes; later samples use target directly. Store target, not the accumulated ramp endpoint, as next.gain.",
    "hold_and_index": "Apply the F04D.3 attack/release branches, preserve hold across callbacks, and select index 128..255 with strict lookup<threshold[ordinal]. The final ordinal 127 does not read threshold[127].",
    "output": "The F04D contribution is the selected index, six-lane descriptor-gain output, and proposed complete DescriptorState after. It is not final_six; temporal3, post-S09 delay, half-scale, and second-ring work remain ordered F04R stages.",
    "peak_and_level": "P=max(old.tail_peak, head_max, tail_max); shift is derived from the clamped peak's raw F32 word using the wide-unsigned E expression in F04D.2. P=0 or shift>=48 gives level 1; otherwise evaluate the five supplied F32 polynomial/headroom parameters with the separately rounded operations in F04D.2."
  },
  "error_mapping": {
    "authority": "Approved clean-room F04D API outcome mapping; not a native DEE error-code equivalence claim.",
    "error_code_and_precedence_status": "MAPPED_BY_F04_DESCRIPTOR_CONTROL_V1",
    "error_map_file": "docs/implementation/ERROR-MAP.md",
    "f04d_error_outcome_rows": 6,
    "outcomes": [
      {
        "caller_action": "return no index, ramp output, or after proposal; I03 applies STAGE_FATAL and retires the parent; committed descriptor state is unchanged; do not retry the same attempt",
        "category": "provider availability",
        "error_code": "RECEIVER_PARAMETER_UNAVAILABLE",
        "programme_state_after": "RETIRED",
        "trigger": "selected descriptor threshold, required scalar, or PG03 gain provider is missing"
      },
      {
        "caller_action": "return no index, ramp output, or after proposal; I03 applies STAGE_FATAL and retires the parent; committed descriptor state is unchanged; do not retry the same attempt",
        "category": "provider identity/shape/finite/gainpositive",
        "error_code": "RECEIVER_PARAMETER_INVALID",
        "programme_state_after": "RETIRED",
        "trigger": "selected threshold/gain/scalar identity, count, finite-word, or positive-gain validation fails"
      },
      {
        "caller_action": "return no index, ramp output, or after proposal; I03 applies STAGE_FATAL and retires the parent; committed descriptor state is unchanged; do not retry the same attempt",
        "category": "state shape/range",
        "error_code": "F04D_HOLD_RANGE",
        "programme_state_after": "RETIRED",
        "trigger": "canonical hold_count cannot be decoded as a semantic U32 in 0..0xffffffff"
      },
      {
        "caller_action": "return no index, ramp output, or after proposal; I03 applies STAGE_FATAL and retires the parent; committed descriptor state is unchanged; do not retry the same attempt",
        "category": "input shape",
        "error_code": "F04D_INPUT_SHAPE",
        "programme_state_after": "RETIRED",
        "trigger": "control or post-temporal3 ramp input is not finite F32[6][1536] shape"
      },
      {
        "caller_action": "return no index, ramp output, or after proposal; I03 applies STAGE_FATAL and retires the parent; committed descriptor state is unchanged; do not retry the same attempt",
        "category": "input finite",
        "error_code": "F04D_INPUT_NONFINITE",
        "programme_state_after": "RETIRED",
        "trigger": "a control or post-temporal3 ramp input sample is not finite"
      },
      {
        "caller_action": "return no index, ramp output, or after proposal; I03 applies STAGE_FATAL and retires the parent; committed descriptor state is unchanged; do not retry the same attempt",
        "category": "numeric intermediate",
        "error_code": "NONFINITE_INTERMEDIATE",
        "programme_state_after": "RETIRED",
        "trigger": "a descriptor control or gain-ramp intermediate/output becomes nonfinite after finite inputs and valid providers"
      }
    ],
    "precedence": [
      "provider availability",
      "provider identity/shape/finite/gainpositive",
      "state shape/range",
      "input shape/finite",
      "numeric intermediate",
      "success"
    ],
    "programme_rule": "F04D.6 delegates to I03."
  },
  "executor_and_receipt_producer": "F04C owns the enclosing receiver callback and selected-J05 receipt; F04D is a receiver substage and has no local commit or receipt authority.",
  "fixture_materialization_status": "NOT_MATERIALIZED_IN_MAPPING_COMMIT",
  "functional_id": "F04D",
  "implementation_files": [],
  "implementation_prohibitions": [
    "no mode=0 fitting implementation",
    "no F04R preprocessing or target producer",
    "no final-six or J05 implementation",
    "no local state commit/abort/fatal authority",
    "no W04 audio or product acceptance promotion"
  ],
  "implementation_readiness": "READY_FOR_IMPLEMENTATION_F04D_DESCRIPTOR_CONTROL_ONLY__WHOLE_RECEIVER_NOT_RUN",
  "implementation_status": "NOT_IMPLEMENTED",
  "input_contract": {
    "control_scratch": "Six finite F32 lanes of 1536 samples from receiver scratch after temporal0..2 and before temporal3. F04D.2 peak consumes lanes [0,1,2,4,5], head [0,1280), tail [1280,1536), and old tail_peak; it saves only this callback's tail maximum.",
    "gain_scratch": "After temporal3, descriptor F04D.4 applies the selected target across all six lanes, including LFE. F04R then applies its 32-sample post-S09 delay, 0.5 scale, and second ring to form final_six.",
    "inputs_are_opaque": "The future stage consumes the F04R temporal-stage results and spatial_audio before-state through typed same-callback views. It must not add a public raw-slice constructor or rebuild upstream F04R stages.",
    "parent_lineage": "Same parent callback lineage: programme, attempt, transaction, FrontendCallbackIndex g, phase, profile, and spatial_audio before generation/digest; the F04C Ready Type10 outcome is from callback g. Reuse approved parent tokens/receipts; F04D does not pin concrete Rust type names."
  },
  "mapping_document": "docs/implementation/DESCRIPTOR-CONTROL-MAP.md",
  "mapping_status": "MAPPING_COMPLETE",
  "minimum_implementable_slice": {
    "candidate_implementation_file": "crates/jocforge-domain/src/spatial/spatial_audio_state/descriptor_control.rs",
    "candidate_test_file": "crates/jocforge-domain/src/spatial/spatial_audio_state/descriptor_control/tests.rs",
    "commit_authority": "none; enclosing F04C/spatial_audio transaction only",
    "does_not_include": [
      "F04R before temporal2 producers",
      "mode=0 fitting producer",
      "post-S09 final-six tail",
      "J05/F05JL",
      "parent CallbackTransaction",
      "W04 product/audio acceptance"
    ],
    "stage_boundary": "One callback-scoped F04D operation has two ordered phases separated by F04R temporal3. Before temporal3, consume post-temporal0..2 scratch and descriptor before-state to select the index and stage the F04D.3 control fields. Carry that private same-callback result through temporal3. After temporal3, consume the six-lane scratch and selected index for the gain ramp; return the gain output and complete DescriptorState after-proposal. Both phases belong to one enclosing F04C callback and have no local commit."
  },
  "package_assets": [
    {
      "bytes": 4009,
      "file": "specs/04-descriptor-control.md",
      "sha256": "da42e76828eb9cf4c882a72af65b5b26f3ca29d0437643ba5c79ed8546618b7e"
    },
    {
      "bytes": 8332,
      "file": "specs/04-receiver-state-and-routing.md",
      "sha256": "74b9b01537eb76a37582535b84426138ee8c8ecdca3486f9f43d0504c7c26306"
    },
    {
      "bytes": 13595,
      "file": "specs/04-qmf-receiver-whole-binding.md",
      "sha256": "104d604cf876cc522a85b21f352bd0a09fc20c3171ddcd71da981c8076ea3a75"
    },
    {
      "bytes": 9103,
      "file": "specs/04-whole-spatial-chain-composition.md",
      "sha256": "92325b12128023001e1f3fd4696dcec68bbd47a46acdb7886781b411a10c5d59"
    },
    {
      "bytes": 4504,
      "file": "specs/00-selected-owner-projection-and-j05.md",
      "sha256": "9ff01aca7b6be6e89511e4d493f9fc9e661a50e2f098065f4a32c3fed777542f"
    },
    {
      "bytes": 20325,
      "file": "parameters/qmf-receiver-functional-v1.json",
      "sha256": "30e63cc110b63dd88118ad37f02750a39167bd01279a3b40853afdf3151b552c"
    },
    {
      "bytes": 4861,
      "file": "parameters/descriptor-mathematical.json",
      "sha256": "a493fbdaf2dab30e940cae9976ed3305f3af083e5614ab0cbd6889c71a02b197"
    },
    {
      "bytes": 226344,
      "file": "test-vectors/F04-WHOLE-SPATIAL-STATE-SCHEMA.json",
      "sha256": "4fdab9f591821d7c50e7daca17bc98d4a5b0616e9cb45b5937cae47b26ed657c"
    },
    {
      "bytes": 1503142,
      "file": "test-vectors/unit/vectors.json",
      "sha256": "a14dd6c3700fae1765399c72c816dd3146d57ab7dd026b375efb6aed7408cfa8"
    },
    {
      "bytes": 74454,
      "file": "test-vectors/QMF-RECEIVER-WHOLE-BINDING-VERIFICATION.json",
      "sha256": "1d809ed24c805a182e9f2ba668562f1f3c8028e4dfc0ce7cd039e1255e4cda07"
    },
    {
      "bytes": 1468,
      "file": "test-vectors/QMF-RECEIVER-WHOLE-BINDING-RESULT.json",
      "sha256": "cb42de15ce4fa787aa03e511b9f2c6da39a72d3927f00eb020295af8a02862c7"
    },
    {
      "bytes": 260481,
      "file": "test-vectors/F04-WHOLE-SPATIAL-CHAIN-VERIFICATION.json",
      "sha256": "ffb1dea3b1e69383312da4059b9d8ddabb4366ed8fc22e7537f750ba041f2d63"
    },
    {
      "bytes": 1863,
      "file": "test-vectors/F04-WHOLE-SPATIAL-CHAIN-RESULT.json",
      "sha256": "cbb776edb314ff43f72bb296af63dd02e1b65a73092185e613199d7dd54c5dc4"
    },
    {
      "bytes": 3744,
      "file": "parameters/w04-audio-spatial-thresholds-v1.json",
      "sha256": "5ff12522076a0ef99d8937d4960db2af2b50b6b42beb9fb68311bf40ebbe36c6"
    },
    {
      "bytes": 3283,
      "file": "test-vectors/W04-AUDIO-SPATIAL-LOCAL-REVIEW-STATUS.json",
      "sha256": "9dec871360697077d11f2160734ffe12b40a66b98d656742a0d51685a56f773b"
    }
  ],
  "package_identity": {
    "manifest_file": "MANIFEST.json",
    "manifest_sha256": "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a"
  },
  "persistent_owner": "spatial_audio.ModelState.descriptor_state",
  "planned_fixture_files": [
    "crates/jocforge-domain/tests/fixtures/descriptor_control/input-vectors.json",
    "crates/jocforge-domain/tests/fixtures/descriptor_control/actual-literals.json"
  ],
  "planned_implementation_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/descriptor_control.rs"
  ],
  "planned_test_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/descriptor_control/tests.rs"
  ],
  "programme_root": "spatial_audio",
  "provider_bindings": {
    "descriptor_scalars": {
      "count": 6,
      "fields": [
        "descriptor_level_offset",
        "descriptor_level_linear",
        "descriptor_level_quadratic",
        "descriptor_headroom_step",
        "descriptor_result_scale",
        "descriptor_rise"
      ],
      "file": "parameters/qmf-receiver-functional-v1.json",
      "representation": "JSON U32 raw finite F32 bits"
    },
    "mathematical_gain_and_threshold": {
      "bytes": 4861,
      "consumer_rule": "PG03 generated gain[128] is the F04D gain route. Do not substitute this mathematical threshold candidate for selected qmf-receiver-functional-v1 descriptor_threshold_127; they differ at 36 of 127 words.",
      "file": "parameters/descriptor-mathematical.json",
      "gain_formula": "(32+j%32)*2^(floor(j/32))/512",
      "gain_qualification": "PASS_ALL128_WORDS",
      "gain_words": 128,
      "sha256": "a493fbdaf2dab30e940cae9976ed3305f3af083e5614ab0cbd6889c71a02b197",
      "threshold_formula": "R32(-log2(16*gain[j])/8), j0..126",
      "threshold_qualification": "36_WORD_DIFFERENCES_NOT_IDENTICAL",
      "threshold_route_status": "MATHEMATICAL_CANDIDATE_AUDIO_ACCEPTANCE_PENDING",
      "threshold_words": 127,
      "typed_provider_status": "LOCALLY_VERIFIED_TYPED_PROVIDER_VALIDATORS"
    },
    "selected_threshold": {
      "bytes": 20325,
      "consumer": "F04D strict lookup for ordinals 0..126",
      "count": 127,
      "f32le_sha256": "5ba787b1ee814e20bbc86290eca1daa6af33968e4e2458b5a3011c87a78fceac",
      "field": "arrays.descriptor_threshold_127.u32_words",
      "file": "parameters/qmf-receiver-functional-v1.json",
      "selected_route": true,
      "sha256": "30e63cc110b63dd88118ad37f02750a39167bd01279a3b40853afdf3151b552c",
      "source_class": "AUTHORIZED_NONSECRET_COMPILED_AUDIO_FUNCTIONAL_CONSTANTS"
    },
    "validation_boundary": "P1.4-D2 identity, schema, shape, finite-word, and qualification validation is PASS. It validates providers only; it does not execute or accept descriptor control, F04D DSP, W04 audio, or product behavior."
  },
  "readiness_boundary": {
    "descriptor_dsp_execution": "NOT_RUN",
    "provider_validation": "LOCALLY_VERIFIED_TYPED_PROVIDER_VALIDATORS",
    "w04_current_execution": {
      "candidate_encode": "NOT_RUN",
      "decode_render_metrics": "NOT_RUN",
      "final_status": "NOT_RUN",
      "listening_signature": "NOT_RUN",
      "player_evidence": "NOT_RUN"
    },
    "w04_implementation_ready": "NO",
    "w04_local_spec_validator_review": "PASS_150_OF_150_ONLY",
    "w04_profile_bytes": 3744,
    "w04_profile_file": "parameters/w04-audio-spatial-thresholds-v1.json",
    "w04_profile_sha256": "5ff12522076a0ef99d8937d4960db2af2b50b6b42beb9fb68311bf40ebbe36c6",
    "w04_review_status_bytes": 3283,
    "w04_review_status_file": "test-vectors/W04-AUDIO-SPATIAL-LOCAL-REVIEW-STATUS.json",
    "w04_review_status_sha256": "9dec871360697077d11f2160734ffe12b40a66b98d656742a0d51685a56f773b",
    "whole_receiver_j05_parent_and_product": "NOT_RUN"
  },
  "resolved_prior_blockers": [
    {
      "id": "selected-provider-direct-vector",
      "resolved_by": "F04-DESCRIPTOR-CONTROL-V1; six selected-provider success vectors and eight error vectors; independent two-path check PASS"
    },
    {
      "id": "canonical-hold-encoding",
      "resolved_by": "F04-DESCRIPTOR-CONTROL-V1; semantic U32 is zero-extended to nonnegative I64LE with boundary examples"
    },
    {
      "id": "error-outcome-translation",
      "resolved_by": "F04-DESCRIPTOR-CONTROL-V1 API-SCHEMA and direct error vectors; six exact local outcome rows are generated into ERROR-MAP"
    }
  ],
  "specifications": [
    {
      "bytes": 4009,
      "clauses": [
        "F04D.1",
        "F04D.2",
        "F04D.3",
        "F04D.4",
        "F04D.5",
        "F04D.6"
      ],
      "file": "specs/04-descriptor-control.md",
      "sha256": "da42e76828eb9cf4c882a72af65b5b26f3ca29d0437643ba5c79ed8546618b7e"
    },
    {
      "bytes": 8332,
      "clauses": [
        "F04R.1",
        "F04R.8"
      ],
      "file": "specs/04-receiver-state-and-routing.md",
      "sha256": "74b9b01537eb76a37582535b84426138ee8c8ecdca3486f9f43d0504c7c26306"
    },
    {
      "bytes": 13595,
      "clauses": [
        "F04RB.1",
        "F04RB.4",
        "F04RB.5",
        "F04RB.6",
        "F04RB.7"
      ],
      "file": "specs/04-qmf-receiver-whole-binding.md",
      "sha256": "104d604cf876cc522a85b21f352bd0a09fc20c3171ddcd71da981c8076ea3a75"
    },
    {
      "bytes": 9103,
      "clauses": [
        "F04C.3",
        "F04C.4",
        "F04C.5",
        "F04C.6",
        "F04C.7",
        "F04C.8"
      ],
      "file": "specs/04-whole-spatial-chain-composition.md",
      "sha256": "92325b12128023001e1f3fd4696dcec68bbd47a46acdb7886781b411a10c5d59"
    },
    {
      "bytes": 4504,
      "clauses": [
        "F04C selected-J05 receipt and F05JL consumer boundary"
      ],
      "file": "specs/00-selected-owner-projection-and-j05.md",
      "sha256": "9ff01aca7b6be6e89511e4d493f9fc9e661a50e2f098065f4a32c3fed777542f"
    }
  ],
  "state_owner": {
    "after_state_authority": "The callback owner stages all six DescriptorState fields with the complete spatial_audio after-root; no partial descriptor-state replacement.",
    "canonical_field": "ModelState.descriptor_state: DescriptorState",
    "canonical_schema_bytes": 226344,
    "canonical_schema_file": "test-vectors/F04-WHOLE-SPATIAL-STATE-SCHEMA.json",
    "canonical_schema_sha256": "4fdab9f591821d7c50e7daca17bc98d4a5b0616e9cb45b5937cae47b26ed657c",
    "cold": {
      "exponent": 2,
      "gain": "1/16",
      "hold": 0,
      "reference": "R32(2/48)",
      "smoothed": "+0",
      "tail_peak": "+0"
    },
    "commit_authority": "Enclosing F04C/spatial_audio callback transaction only; no F04D local commit, receipt, abort, fatal, or programme retirement.",
    "field_order": [
      {
        "canonical": "B4 raw F32 bits",
        "field": "tail_peak"
      },
      {
        "canonical": "I64LE",
        "field": "hold_count"
      },
      {
        "canonical": "B4 raw F32 bits",
        "field": "smoothed"
      },
      {
        "canonical": "I64LE",
        "field": "exponent"
      },
      {
        "canonical": "B4 raw F32 bits",
        "field": "reference"
      },
      {
        "canonical": "B4 raw F32 bits",
        "field": "gain"
      }
    ],
    "hold_encoding_examples": {
      "0": "0000000000000000",
      "4294967295": "ffffffff00000000",
      "5": "0500000000000000"
    },
    "logical_cold_spec": "specs/04-descriptor-control.md F04D.1",
    "logical_to_canonical_hold_encoding": "Semantic hold is U32 in 0..0xffffffff. Canonical hold_count is the same nonnegative integer zero-extended into I64 value space, then encoded as signed I64LE; do not reinterpret raw U32 bytes or sign-extend.",
    "reset_authority": "Programme-origin full spatial_audio reset reconstructs the declared cold DescriptorState; do not clear hold on each callback.",
    "root": "spatial_audio.F04CompositionState.receiver_state: ModelState",
    "schema_pointer": "#/whole_state_by_boundary/cold/fields/6/schema/fields/10/schema",
    "stage_authority": "F04R executes the callback substage; F04D returns a proposal only."
  },
  "status": "READY_FOR_IMPLEMENTATION_F04D_DESCRIPTOR_CONTROL_ONLY__WHOLE_RECEIVER_NOT_RUN",
  "task_id": "F04D-DESCRIPTOR-CONTROL-AUDIT",
  "test_files": [],
  "vector_coverage": {
    "approved_supplement_direct_vectors": {
      "actual_comparison": "all six-lane 1536-sample ramp output words, peak/level/hold/index traces, state-after fields, and canonical hold_count I64LE bytes are compared only after computation",
      "actual_file": "F04D-DIRECT-ACTUAL-LITERALS.json",
      "actual_sha256": "390cf469a1dde8a4fde9c79de2238f414ea041d6b64630cb669ca4ab26205ad0",
      "error_cases": [
        "missing_threshold_precedes_bad_input",
        "threshold_dimension_precedes_bad_state",
        "nonpositive_gain_provider",
        "hold_negative",
        "hold_u32_overflow",
        "bad_input_shape",
        "nonfinite_input",
        "finite_input_intermediate_overflow"
      ],
      "error_count": 8,
      "expected_use": "input recipes and raw input projections are inputs; ACTUAL-LITERALS expected fields are assertions only and never state/provider seeds",
      "independent_check": "PASS_TWO_INDEPENDENT_PATHS; 57079 minimum stage words compared; primary_core_imported=false",
      "input_file": "F04D-DIRECT-INPUT-VECTORS.json",
      "input_sha256": "c622899783197a953fb6c0bcd262b0e7b1a110d605e9301e27782e44cb247528",
      "success_count": 6,
      "successful_cases": [
        "cold_zero",
        "head_tail_lfe_excluded",
        "threshold_equal_hold_decrement",
        "lookup_last_ordinal",
        "attack_and_nontrivial_ramp",
        "old_tail_peak_carry_current_tail_zero"
      ],
      "supplement_manifest_sha256": "40d67c32fdebf97fe8dc249486ce4cb655642146a2470e6bcad553f1639f9e04"
    },
    "selected_provider_direct_vector_status": "PASS_DIRECT_LITERAL_AND_TWO_REFERENCE_PATHS",
    "unit_vectors": {
      "bytes": 1503142,
      "case_count": 3,
      "cases": [
        "U-QMF-DESCRIPTOR-CONTINUOUS",
        "U-QMF-DESCRIPTOR-EQUAL",
        "U-QMF-DESCRIPTOR-LAST"
      ],
      "coverage": "Synthetic direct cases exercise continuous attack/hold/release, strict equality, and last ordinal. They are not direct expected values for the selected production descriptor provider.",
      "file": "test-vectors/unit/vectors.json",
      "qualification": "SYNTHETIC_POLYNOMIAL_NOT_PRODUCTION",
      "sha256": "a14dd6c3700fae1765399c72c816dd3146d57ab7dd026b375efb6aed7408cfa8",
      "threshold_binding": "The vectors use the 127-word mathematical candidate: 127/127 words match descriptor-mathematical.json, but only 91/127 match the selected qmf-receiver-functional-v1 threshold. The selected descriptor scalar polynomial also differs from the synthetic vector polynomial.",
      "vector_polynomial_matched_selected_scalar_count": 1
    },
    "whole_receiver_binding": {
      "bytes": 74454,
      "cases": 5,
      "descriptor_evidence": "Selected descriptor index, target gain, before/after state digests, and whole-stage hashes are present as model-first assertions.",
      "execution_status": "MODEL_FIRST_SUPPLIED_REVIEW_ONLY_NOT_ISOLATED_RECEIVER_EXECUTION",
      "file": "test-vectors/QMF-RECEIVER-WHOLE-BINDING-VERIFICATION.json",
      "sha256": "1d809ed24c805a182e9f2ba668562f1f3c8028e4dfc0ce7cd039e1255e4cda07"
    },
    "whole_spatial_chain": {
      "bytes": 260481,
      "cases": 6,
      "coverage": "g=0..5; receiver and descriptor run only for g>=2; whole-chain expected hashes are assertions, not F04D isolated stage literals.",
      "execution_status": "MODEL_FIRST_SUPPLIED_REVIEW_ONLY_NOT_ISOLATED_RECEIVER_EXECUTION",
      "file": "test-vectors/F04-WHOLE-SPATIAL-CHAIN-VERIFICATION.json",
      "sha256": "ffb1dea3b1e69383312da4059b9d8ddabb4366ed8fc22e7537f750ba041f2d63"
    }
  }
}
```
