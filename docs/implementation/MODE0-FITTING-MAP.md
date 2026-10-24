# Mode=0 QMF fitting map

由 IMPLEMENTATION-MAP.json 生成，不得手工修改。

Mapping status: MAPPING_COMPLETE. Implementation status: LOCALLY_VERIFIED_MODE0_FITTING_DIRECT_KATS__WHOLE_RECEIVER_NOT_RUN. Implementation readiness: LOCALLY_VERIFIED_MODE0_FITTING_DIRECT_KATS__WHOLE_RECEIVER_NOT_RUN.

This map authorizes only the F04RB mode=0 fitting stage. The receiver route, F05JC fit, J05/F05JL, final-six tail, and parent transaction remain separate and deferred.

The carrier path re-analyzes same-generation s09_input in [L,R,C,Ls,Rs] order from the receiver analysis before-state. Its analysis after-state must equal the receiver analysis after-state wordwise, but the fitting branch proposes only its own delay/FIR/window after-state; it has no local commit authority. Mode=0 updates delay and the 20-complex history while skipping phase rotation, tap multiplication, band0 replacement, and synthesis.

Target and carrier windows each join their own previous 24 slots to the current 24 slots. F05JC may consume the resulting 15x48 target and 5x48 carrier only when eligibility, weights, geometry, and the descriptor after index are same-generation ready. s09_return and final_six are not carrier inputs.

F04RB-MODE0-FITTING-V1 binds cold g2, warm g3, and cached-drain g4 recipes to post-computation literals and complete canonical after-state hashes. Execute all stages from recipes and prior computed state before comparing literals. The g4 drain repeats g3 input, but still advances the fitting state once. U-QMF-SURROUND-RESET remains a mode=1 case. Existing whole-binding assertions remain model-first and do not qualify whole receiver execution.

The API precedence is provider availability, provider identity/shape/finite, callback and projection lineage, before-state, input shape/finite, then numeric or receiver-analysis join. The API has no separate receiver-analysis join code, so this map uses the clean local F04RB_LINEAGE_ERROR outcome; it makes no native error-code claim.

The full provider, state, clock, outcome, KAT, and implementation-scope record follows.

## Machine-readable contract

```json
{
  "api_schema_shape_binding": "Raw storage shapes are pinned by F04RB.7 and the actual-literal field shapes: previous_targets=[15,24,128] F32 words, previous_carriers=[5,24,128] F32 words, fitting_fir=[5,40] F32 words (20 complex per lane). API-SCHEMA uses compact dimensions; it is not a second serialization schema.",
  "approved_supplement": {
    "files": [
      {
        "bytes": 8252,
        "file": "SPEC.md",
        "sha256": "54f76a4935d7fef0171d71c4422c40d466be5aa0b94df6c4437a7f32422f19da"
      },
      {
        "bytes": 1019,
        "file": "API-SCHEMA.json",
        "sha256": "c9ba108c602ab37f24713628b1bcf0e5dcd06189dd89501a70bde04b77c9ba8c"
      },
      {
        "bytes": 7696,
        "file": "F04RB-MODE0-INPUT-VECTORS.json",
        "sha256": "733a9502491c7d1e1e543b15b36bf34135960e285c8240615f44b63225720a5a"
      },
      {
        "bytes": 19861457,
        "file": "F04RB-MODE0-ACTUAL-LITERALS.json",
        "sha256": "7c5d6b034412c09264117e210fccb270f20550f099df2b00209e762876c30c77"
      }
    ],
    "identity": "f04rb-mode0-fitting.v1",
    "manifest_bytes": 1192,
    "manifest_file": "MANIFEST.json",
    "manifest_sha256": "0f597e960eb98ae6ffb7436ea6ae0c435fb73e942801f2cd2dd3222028152be2",
    "root": "outputs/IMPLEMENTER_SUPPLEMENTS/F04RB-MODE0-FITTING-V1",
    "secrets_included": false,
    "status": "IMPLEMENTATION_SPEC_READY_WHOLE_RECEIVER_NOT_RUN"
  },
  "blockers": [],
  "clock_contract": {
    "dispatch_relation": "d=g-1=r+1 for the same callback; mode0 prepares fitting windows and does not itself execute F05JL/J05.",
    "eof": "EOF adds no receiver callback, mode0 flush, or synthetic zero QMF slot.",
    "frontend_cached_drain": "g=4247..4250",
    "frontend_normal": "g=0..4246",
    "receiver_count": "r=0..4248; 4,249 receiver callbacks, including actual cached-drain callbacks.",
    "receiver_execution": "For g>=2, r=g-2, receiver generation=r+1=g-1; g=0/1 prime Type10 and do not run receiver/mode0.",
    "slot_clock": "Each successful receiver callback processes 24 ordered 64-sample slots for each applicable QMF lane."
  },
  "downstream_consumer": "F05JC",
  "error_mapping": {
    "authority": "Approved F04RB/F04C boundary and I03 retirement policy; this is a clean-room local outcome mapping, not a native error-code equivalence claim.",
    "fatal_result": "No fitting output or accepted after-state; I03 retires the programme and enclosing callback does not commit spatial_audio.",
    "native_error_equivalence": false,
    "out_of_scope": "F04_PROFILE_UNSUPPORTED is resolved by the selected profile gate before a receiver callback; it is not a mode0 stage error.",
    "outcomes": [
      "RECEIVER_PARAMETER_UNAVAILABLE for absent selected QMF/FFT/descriptor provider",
      "RECEIVER_PARAMETER_INVALID for provider identity, shape, or finite failure",
      "F04RB_LINEAGE_ERROR for stale callback lineage or receiver-analysis-after join mismatch",
      "F04RB_DESCRIPTOR_PROJECTION for selected index/PG03 gain/generation mismatch",
      "F04RB_MODE0_STATE_SHAPE for fitting before-state shape, cursor, or finite failure",
      "F04RB_INPUT_SHAPE for s09_input shape failure",
      "F04RB_TARGET_TEMPORAL_SHAPE for target input shape failure",
      "F04RB_INPUT_NONFINITE for input finite validation failure",
      "F04RB_NUMERIC_NONFINITE for a nonfinite QMF intermediate or output"
    ],
    "precedence": [
      "provider availability",
      "provider identity/shape/finite",
      "programme/attempt/callback/generation/lane/descriptor lineage",
      "fitting before-state shape/finite",
      "s09/target input shape/finite",
      "numeric intermediate or receiver-analysis join",
      "success"
    ],
    "receiver_analysis_join_error_mapping": "F04RB_LINEAGE_ERROR; API has no separate join code. This is a clean local mapping, not a native error-code claim."
  },
  "fixture_inventory": [
    {
      "bytes": 7696,
      "path": "crates/jocforge-domain/tests/fixtures/mode0_fitting/input-vectors.json",
      "sha256": "733a9502491c7d1e1e543b15b36bf34135960e285c8240615f44b63225720a5a"
    },
    {
      "bytes": 19861457,
      "path": "crates/jocforge-domain/tests/fixtures/mode0_fitting/actual-literals.json",
      "sha256": "7c5d6b034412c09264117e210fccb270f20550f099df2b00209e762876c30c77"
    }
  ],
  "fixture_materialization_status": "MATERIALIZED_AND_HASH_PINNED",
  "functional_id": "F04RB",
  "implementation_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/fitting_mode0.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state.rs"
  ],
  "implementation_inventory": [
    {
      "bytes": 40594,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/fitting_mode0.rs",
      "sha256": "b90dd8904d7c93ef258871edcebf3f4ffd8b6d5347f39a7fdc964af4b4f2e2ff"
    },
    {
      "bytes": 37519,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state.rs",
      "sha256": "dc5516d1bef5b18983b469116c3f8c959d12931dc10af18b5233794865587621"
    }
  ],
  "implementation_prohibitions": [
    "no receiver mode=1 state aliasing or behavior changes",
    "no F04R producer, target producer integration, or receiver routing implementation",
    "no descriptor selection/ramp; consume only the selected after index/target projection",
    "no F05JC fit algorithm, J05/F05JL, final-six tail, or parent transaction",
    "no mode0 local commit/abort/fatal authority",
    "no whole-receiver or product acceptance promotion"
  ],
  "implementation_readiness": "LOCALLY_VERIFIED_MODE0_FITTING_DIRECT_KATS__WHOLE_RECEIVER_NOT_RUN",
  "implementation_status": "LOCALLY_VERIFIED_MODE0_FITTING_DIRECT_KATS__WHOLE_RECEIVER_NOT_RUN",
  "input_contract": {
    "callback_index": "F04RB callback_index_zero_based is the receiver index r, not frontend index g; generation=r+1.",
    "carrier_branch": "Re-analyze same-generation s09_input lanes [0,1,2,4,5] from receiver before analysis histories. Raw analysis after must be wordwise equal to receiver analysis after; fitting does not commit receiver analysis history a second time.",
    "descriptor_clip_index": "Bind the explicit same-generation descriptor after projection: selected_index is 128..255, target_gain bits equal PG03 gain[selected_index-128], and projection generation is r+1. Emit clip_index=selected_index. Do not derive from the first-256 ramp current, receiver before gain, s09_return, or final_six.",
    "forbidden_audio_aliases": [
      "s09_return",
      "final_six"
    ],
    "geometry_binding": "F05JC geometry consumes receiver row_states_before and the same-generation target_updates; geometry_row_states_after must equal receiver row_states_after fieldwise.",
    "receiver_callback": "Same-generation Type10[16][1536] F32 plus target_updates map<U4,row6F32>; row0 is direct LFE and rows1..15 are component0..14. Preflight PCM/control type, shape, and finite status before processing.",
    "target_branch": "Use preconditioned[1..15], not row0. Target has independent span-896 temporal exchange, 15 target-analysis histories, and no mode0 carrier delay/FIR.",
    "target_updates_rule": "Each update replaces only that row's target[6]; preserve the before current, slope, and remaining values."
  },
  "mapping_document": "docs/implementation/MODE0-FITTING-MAP.md",
  "mapping_status": "MAPPING_COMPLETE",
  "mode0_contract": {
    "analysis": "Use the selected same-generation s09_input and the receiver before five-row analysis history; the recomputed five-row raw analysis after joins wordwise to the receiver after.",
    "band0_history": "Append the 24 current band0 complex values to that lane's independent 20-complex before history and save the final 20 complex values. Mode0 does not multiply the 21 taps and does not replace band0.",
    "carrier_order": [
      "L",
      "R",
      "C",
      "Ls",
      "Rs"
    ],
    "carrier_window": "For each of five lanes, carrier_window48 is previous_carrier24 concatenated with current mode0 delayed 24 slots; after-state previous_carrier24 becomes this current24.",
    "clip_index": "Emit the same selected_index from the explicit descriptor after projection; do not rerun descriptor selection or read ramp/final audio.",
    "delay": "For each lane concatenate fitting before delay 10x128 words with 24 current analysis slots; mode0 emits the first 24 slots and saves the last 10 current slots as fitting delay after.",
    "fitting_output": "Target 15x48x128 F32 plus carrier 5x48x128 F32. F05JC may begin only after both, eligibility, weights, and geometry are ready for the same generation.",
    "j05_relation": "F04RB/mode0 does not produce J05. F04C assembles the selected J05 receipt only after its whole callback; F05JL consumes that projection and does not rerun receiver or fitting histories.",
    "phase_rotation": "None. The mode1 (r,i)->(i XOR signbit,r) transform is not applied.",
    "receiver_lane_map": [
      0,
      1,
      2,
      4,
      5
    ],
    "route": "QMF_FFT_RADIX2_MATH_V1",
    "slot_count": 24,
    "synthesis": "None. No inverse reorder, synthesis FFT, postmodulation, or polyphase synthesis is run for mode0.",
    "target_window": "For each of fifteen rows, target_window48 is previous_target24 concatenated with current target24; after-state previous_target24 becomes this current24. Detector input is the current target24, the window's second half."
  },
  "mode_difference_and_reuse": {
    "mode0": "five fitting-owned delay histories and five fitting-owned 20-complex histories advance; no phase rotation, tap multiplication, band0 overwrite, or synthesis",
    "mode1": "two receiver-owned surround delay/FIR/synthesis histories; phase rotation and 21-tap band0 FIR overwrite are selected only for lanes [Ls,Rs]",
    "not_shareable_state": "Receiver analysis history is seeded as read-only before-state for mode0 recomputation; receiver and fitting delay/FIR histories remain separate owners and are never aliased, even when cold buffers compare equal.",
    "provider_consumption": "Mode0 consumes the selected QWIN 640-word prototype (analysis is the strict reverse of the synthesis table) plus QMF FFT roots and rotation. Mode1 band0 taps are not consumed by mode0. receiver_blend32 is upstream F04R.5 and is not a direct mode0 dependency.",
    "shareable_pure_stage": "The fixed QMF FFT and QMF analysis transform/provider arithmetic can be called twice from different before-history slices for the receiver and mode0 carrier path, subject to existing identity validation."
  },
  "package_assets": [
    {
      "bytes": 13595,
      "clauses": [
        "F04RB.1",
        "F04RB.2",
        "F04RB.3",
        "F04RB.4",
        "F04RB.5",
        "F04RB.6",
        "F04RB.7",
        "F04RB.8",
        "F04RB.9"
      ],
      "file": "specs/04-qmf-receiver-whole-binding.md",
      "sha256": "104d604cf876cc522a85b21f352bd0a09fc20c3171ddcd71da981c8076ea3a75"
    },
    {
      "bytes": 5584,
      "clauses": [
        "F04Q.1",
        "F04Q.2",
        "F04Q.3",
        "F04Q.4",
        "F04Q.5",
        "F04Q.6"
      ],
      "file": "specs/04-qmf-analysis-and-synthesis.md",
      "sha256": "13a2907c8e60bd81f948f33b9ad07f2d7dc0552b0b12f53224d27b80af610edf"
    },
    {
      "bytes": 8332,
      "clauses": [
        "F04R.1",
        "F04R.2",
        "F04R.3",
        "F04R.4",
        "F04R.5",
        "F04R.6"
      ],
      "file": "specs/04-receiver-state-and-routing.md",
      "sha256": "74b9b01537eb76a37582535b84426138ee8c8ecdca3486f9f43d0504c7c26306"
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
      "bytes": 15276,
      "clauses": [
        "F05JC"
      ],
      "file": "specs/05-joc-selected-producer-and-commit.md",
      "sha256": "92d7be8c41adf94056c81d98514b66318b3a38b006db3426bf56919effd9f012"
    },
    {
      "bytes": 20325,
      "file": "parameters/qmf-receiver-functional-v1.json",
      "sha256": "30e63cc110b63dd88118ad37f02750a39167bd01279a3b40853afdf3151b552c"
    },
    {
      "bytes": 6983,
      "file": "parameters/qmf-mathematical-fft-v1.json",
      "sha256": "e8d3fb1514a5ffd857740222eacc8995208a29c3447e48c1b63083af368c0431"
    },
    {
      "bytes": 4861,
      "file": "parameters/descriptor-mathematical.json",
      "sha256": "a493fbdaf2dab30e940cae9976ed3305f3af083e5614ab0cbd6889c71a02b197"
    },
    {
      "bytes": 226344,
      "file": "test-vectors/F04-WHOLE-SPATIAL-STATE-SCHEMA.json",
      "schema_pointer": "#/whole_state_by_boundary/cold/fields/7/schema",
      "sha256": "4fdab9f591821d7c50e7daca17bc98d4a5b0616e9cb45b5937cae47b26ed657c"
    },
    {
      "bytes": 74454,
      "cases": [
        "QRB-CONT-0",
        "QRB-CONT-1",
        "QRB-CONT-2",
        "QRB-CONT-3",
        "QRB-CONT-4"
      ],
      "file": "test-vectors/QMF-RECEIVER-WHOLE-BINDING-VERIFICATION.json",
      "qualification": "MODEL_FIRST_SUPPLIED_REVIEW_ONLY_NOT_ISOLATED_RECEIVER_EXECUTION",
      "sha256": "1d809ed24c805a182e9f2ba668562f1f3c8028e4dfc0ce7cd039e1255e4cda07"
    },
    {
      "bytes": 1468,
      "file": "test-vectors/QMF-RECEIVER-WHOLE-BINDING-RESULT.json",
      "qualification": "MODEL_FIRST_SUPPLIED_REVIEW_ONLY_NOT_ISOLATED_RECEIVER_EXECUTION",
      "sha256": "cb42de15ce4fa787aa03e511b9f2c6da39a72d3927f00eb020295af8a02862c7"
    },
    {
      "bytes": 260481,
      "cases": 6,
      "file": "test-vectors/F04-WHOLE-SPATIAL-CHAIN-VERIFICATION.json",
      "qualification": "MODEL_FIRST_ASSERTIONS; mode0 receiver/fitting calls are present only in the g>=2 cases",
      "sha256": "ffb1dea3b1e69383312da4059b9d8ddabb4366ed8fc22e7537f750ba041f2d63"
    },
    {
      "bytes": 1503142,
      "file": "test-vectors/unit/vectors.json",
      "negative_boundary_case": "U-QMF-SURROUND-RESET is mode=1 and is not mode0 fitting evidence",
      "sha256": "a14dd6c3700fae1765399c72c816dd3146d57ab7dd026b375efb6aed7408cfa8"
    }
  ],
  "package_identity": {
    "manifest_bytes": 76302,
    "manifest_file": "MANIFEST.json",
    "manifest_sha256": "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a"
  },
  "persistent_owner": "spatial_audio.F04CompositionState.fitting_qmf_state",
  "planned_fixture_files": [
    "crates/jocforge-domain/tests/fixtures/mode0_fitting/input-vectors.json",
    "crates/jocforge-domain/tests/fixtures/mode0_fitting/actual-literals.json"
  ],
  "planned_implementation_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/fitting_mode0.rs"
  ],
  "planned_test_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/fitting_mode0/tests.rs"
  ],
  "planned_vector_sources": [
    "outputs/IMPLEMENTER_SUPPLEMENTS/F04RB-MODE0-FITTING-V1/F04RB-MODE0-INPUT-VECTORS.json",
    "outputs/IMPLEMENTER_SUPPLEMENTS/F04RB-MODE0-FITTING-V1/F04RB-MODE0-ACTUAL-LITERALS.json",
    "test-vectors/QMF-RECEIVER-WHOLE-BINDING-VERIFICATION.json",
    "test-vectors/F04-WHOLE-SPATIAL-CHAIN-VERIFICATION.json"
  ],
  "programme_root": "spatial_audio",
  "readiness_boundary": {
    "direct_mode0_stage_execution": "LOCALLY_VERIFIED_DIRECT_KATS",
    "direct_mode0_vectors": "THREE_SUCCESS_NINE_ERROR_CASES_HASH_PINNED",
    "j05_parent": "NOT_RUN",
    "mode0_spec_semantics": "MAPPED",
    "provider_identities": "BOUND_TO_APPROVED_BASE_PACKAGE_AND_MODE0_SUPPLEMENT",
    "real_adm_and_product_acceptance": "NOT_RUN",
    "whole_receiver": "NOT_RUN"
  },
  "resolved_prior_blockers": [
    {
      "id": "mode0-direct-stage-literal",
      "resolved_by": "F04RB-MODE0-FITTING-V1; three selected cold/warm/cached direct KATs carry all stages and complete after-state bytes/hash."
    },
    {
      "id": "f04rb-error-precedence",
      "resolved_by": "F04RB-MODE0-FITTING-V1/API-SCHEMA.json; provider, identity, lineage/projection, state, input, numeric/join, success order and nine input-only error cases."
    }
  ],
  "state_ownership": {
    "canonical_encoding": "Use F04RB.7 encode_node-v1 tags N/B/I/T/D, fixed little-endian lengths/integers, ASCII class/field names, and declared field order. Do not invent a parallel serialization root.",
    "canonical_schema_bytes": 226344,
    "canonical_schema_file": "test-vectors/F04-WHOLE-SPATIAL-STATE-SCHEMA.json",
    "canonical_schema_sha256": "4fdab9f591821d7c50e7daca17bc98d4a5b0616e9cb45b5937cae47b26ed657c",
    "cold_provider": "Verified F04 generated cold recipe bound to the whole-state schema; no runtime cold-state JSON provider. All QmfWindowState buffer fields start at positive zero.",
    "commit_authority": "Only the enclosing spatial_audio root/callback transaction; mode0 has no local commit.",
    "eof_drain": "Process every actual cached/padded frontend callback with Ready Type10; update previous target/carrier windows on each successful callback; do not synthesize a QMF flush callback.",
    "fatal_behavior": "Any fatal stage returns no accepted fitting after-state; the enclosing I03/F04 callback retires the programme and discards the callback-staged spatial_audio root.",
    "fields": [
      {
        "cold": "both span-sized temporal buffers in every lane are positive zero",
        "exact_schema": "TemporalState(span=896,stride_bytes=4,index_28=896,index_32=0,buffers=T15<B7168>)",
        "field": "target_temporal"
      },
      {
        "cold": "all positive zero",
        "exact_schema": "T15<T9<B256>> = 15 rows x 9 x 64 F32",
        "field": "target_history"
      },
      {
        "cold": "all positive zero",
        "exact_schema": "T15<T24<B512>> = 15 rows x 24 x 128 F32",
        "field": "previous_targets"
      },
      {
        "cold": "all positive zero",
        "exact_schema": "T5<T24<B512>> = [L,R,C,Ls,Rs] x 24 x 128 F32",
        "field": "previous_carriers"
      },
      {
        "cold": "all positive zero",
        "exact_schema": "T5<T10<B512>> = [L,R,C,Ls,Rs] x 10 x 128 F32",
        "field": "fitting_delay"
      },
      {
        "cold": "all positive zero",
        "exact_schema": "T5<B160> = [L,R,C,Ls,Rs] x 40 F32 = 20 complex values per lane",
        "field": "fitting_fir"
      }
    ],
    "owner": "spatial_audio.F04CompositionState.fitting_qmf_state",
    "receipt_type": "No mode0-local receipt is specified; the staged result is subordinate to the enclosing F04C callback. Selected J05 receipt and parent commit remain deferred.",
    "reset_authority": "Programme-origin full reset reconstructs all six QmfWindowState fields from the schema's positive-zero cold state.",
    "schema_id": "encode_node-v1; whole QmfWindowState is the canonical root; field order is declaration order",
    "schema_pointer": "#/whole_state_by_boundary/cold/fields/7/schema",
    "stage_authority": "F04Q.4 defines mode0 delay/history behavior; F04RB.2/.3 binds the five-lane fitting path and windows; F04C executes the enclosing receiver callback.",
    "state_type": "QmfWindowState"
  },
  "status": "LOCALLY_VERIFIED_MODE0_FITTING_DIRECT_KATS__WHOLE_RECEIVER_NOT_RUN",
  "supplied_review_boundary": {
    "current_binding": "The supplied local-review status carries stale package/spec identities and is not reused as a current isolated execution pass.",
    "reported_f04rb_local_ready": "YES",
    "reported_implementation_ready": "NO"
  },
  "task_id": "F04RB-MODE0-FITTING-MAPPING-AUDIT",
  "test_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/fitting_mode0/tests.rs"
  ],
  "test_inventory": [
    {
      "bytes": 23025,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/fitting_mode0/tests.rs",
      "sha256": "842a854b6fb9be7066842559ed07c06ef832ca88012c6d1b8d0fd02a5bffb27f"
    }
  ],
  "vector_coverage": {
    "direct_mode0": {
      "actual_bytes": 19861457,
      "actual_file": "outputs/IMPLEMENTER_SUPPLEMENTS/F04RB-MODE0-FITTING-V1/F04RB-MODE0-ACTUAL-LITERALS.json",
      "actual_first_rule": "Compute target/carrier stages and complete after-state from input recipes and validated providers first; compare actual literals and canonical byte hashes only afterward. Never seed any runtime state from expected literals or hashes.",
      "actual_sha256": "7c5d6b034412c09264117e210fccb270f20550f099df2b00209e762876c30c77",
      "after_binary_note": "The canonical_after file names in ACTUAL-LITERALS are labels; the package distributes after-state U32 fields and canonical byte count/SHA in ACTUAL-LITERALS, not separate FITTING-AFTER-*.bin files.",
      "api_shape_binding": "F04RB.7 and actual-literal shapes pin previous target/carrier slots to 128 F32 words and fitting_fir to 40 F32 words (20 complex) per lane.",
      "cached_drain_rule": "g4 CACHED_DRAIN repeats g3 input recipes exactly and still executes one fitting transition; it is not skipped and does not add a zero flush.",
      "canonical_after_kats": [
        {
          "bytes": 421039,
          "case_id": "cold_normal_g2",
          "sha256": "466067702d5054a0e25c231dbdcd53b72c589c3c74a7cf95a6b33abb930786c0",
          "target_temporal_read": 1792,
          "target_temporal_write": 896
        },
        {
          "bytes": 421039,
          "case_id": "warm_normal_g3",
          "sha256": "46bb20d1e327735dcc03d13a36e36d7ddc3f37b74df7e3abd046ec0fa839fd05",
          "target_temporal_read": 896,
          "target_temporal_write": 1792
        },
        {
          "bytes": 421039,
          "case_id": "cached_drain_g4",
          "sha256": "9c3cef69fbe4e08d4ce58718cad125c2fcbf54f5fad2bed9e1941f7eac057d32",
          "target_temporal_read": 1792,
          "target_temporal_write": 896
        }
      ],
      "descriptor_projections": [
        {
          "case_id": "cold_normal_g2",
          "generation": 1,
          "selected_index": 128,
          "target_gain_u32": 1031798784
        },
        {
          "case_id": "warm_normal_g3",
          "generation": 2,
          "selected_index": 162,
          "target_gain_u32": 1040711680
        },
        {
          "case_id": "cached_drain_g4",
          "generation": 3,
          "selected_index": 179,
          "target_gain_u32": 1045168128
        }
      ],
      "error_cases": [
        "provider_missing_precedes_stale",
        "provider_identity_invalid",
        "stale_lineage",
        "descriptor_projection_mismatch",
        "state_delay_shape",
        "s09_shape",
        "target_shape",
        "nonfinite_input",
        "finite_numeric_overflow"
      ],
      "input_bytes": 7696,
      "input_file": "outputs/IMPLEMENTER_SUPPLEMENTS/F04RB-MODE0-FITTING-V1/F04RB-MODE0-INPUT-VECTORS.json",
      "input_sha256": "733a9502491c7d1e1e543b15b36bf34135960e285c8240615f44b63225720a5a",
      "literal_output_shapes": {
        "carrier_mode0_current24": [
          5,
          24,
          128
        ],
        "carrier_raw_analysis": [
          5,
          24,
          128
        ],
        "carrier_windows48": [
          5,
          48,
          128
        ],
        "receiver_analysis_after_join": [
          5,
          9,
          64
        ],
        "target_raw_analysis": [
          15,
          24,
          128
        ],
        "target_temporal_output": [
          15,
          1536
        ],
        "target_windows48": [
          15,
          48,
          128
        ]
      },
      "qualification": "Direct mode0 fitting stage KAT only; receiver whole, F05JC/J05, parent transaction, real ADM/listening, native error equivalence, and product acceptance remain NOT_RUN.",
      "success_cases": [
        "cold_normal_g2",
        "warm_normal_g3",
        "cached_drain_g4"
      ],
      "supplement_manifest_sha256": "0f597e960eb98ae6ffb7436ea6ae0c435fb73e942801f2cd2dd3222028152be2"
    },
    "existing_direct_qmf_supplements": {
      "analysis_manifest_sha256": "79830470e3101a10d96852f44c855252c7b78533e0c2b32a73c86d93f74542c6",
      "coverage": "shared QMF analysis and mode1 synthesis only; mode0-specific delay/FIR/window coverage comes from the separately identity-bound F04RB-MODE0-FITTING-V1 supplement",
      "synthesis_manifest_sha256": "bff23be87f4b7ceb27c145ab6106dd1b786aa2e6dbc35d1e4892da73743e9f4a"
    },
    "mode0_direct_input_and_literal": "PRESENT_AND_HASH_PINNED",
    "mode1_negative_case": "U-QMF-SURROUND-RESET is mode=1 only; it cannot satisfy mode0 coverage.",
    "whole_binding": {
      "assertion_counts": {
        "carrier_window_words": 153600,
        "raw_analysis_history_join_words": 14400,
        "receiver_output_words": 46080,
        "target_window_words": 460800
      },
      "case_ids": [
        "QRB-CONT-0",
        "QRB-CONT-1",
        "QRB-CONT-2",
        "QRB-CONT-3",
        "QRB-CONT-4"
      ],
      "coverage": "model-first assertions for continuous receiver/fitting states, descriptor index, windows, and analysis-history joins; callback_index_zero_based is r=0..4, corresponding to parent g=2..6",
      "execution_status": "NOT_ISOLATED_IMPLEMENTATION_EXECUTION",
      "file": "test-vectors/QMF-RECEIVER-WHOLE-BINDING-VERIFICATION.json",
      "sha256": "1d809ed24c805a182e9f2ba668562f1f3c8028e4dfc0ce7cd039e1255e4cda07"
    },
    "whole_chain": {
      "coverage": "six model-first frontend callbacks g=0..5; receiver/fitting runs only for g>=2",
      "execution_status": "NOT_ISOLATED_IMPLEMENTATION_EXECUTION",
      "file": "test-vectors/F04-WHOLE-SPATIAL-CHAIN-VERIFICATION.json",
      "sha256": "ffb1dea3b1e69383312da4059b9d8ddabb4366ed8fc22e7537f750ba041f2d63"
    }
  },
  "verification": {
    "after_state_words_compared": 310680,
    "canonical_after_sha256_kats": 3,
    "error_case_count": 9,
    "mode0_boundary_mutations_rejected": 42,
    "module_test_count": 2,
    "repository_boundary_mutations_rejected": 857,
    "stage_output_words_compared": 676800,
    "status": "LOCALLY_VERIFIED_MODE0_FITTING_DIRECT_KATS__WHOLE_RECEIVER_NOT_RUN",
    "success_case_count": 3,
    "w04_execution": "NOT_RUN",
    "whole_receiver_j05_parent_product": "NOT_RUN"
  },
  "whole_receiver_executor": "F04C"
}
```
