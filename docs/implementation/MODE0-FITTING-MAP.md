# Mode=0 QMF fitting map

由 IMPLEMENTATION-MAP.json 生成，不得手工修改。

Mapping status: MAPPING_COMPLETE. Implementation status: NOT_IMPLEMENTED. Implementation readiness: NO_IMPLEMENTATION.

This is a documentation-only mapping of the F04RB mode=0 fitting branch. It does not implement a QMF stage, connect the receiver, run the F05JC fit, produce J05, or authorize the parent transaction.

The carrier path re-analyzes same-generation s09_input in [L,R,C,Ls,Rs] order from the receiver analysis before-state. Its analysis after-state must equal the receiver analysis after-state wordwise, but the fitting branch proposes only its own delay/FIR/window after-state; it has no local commit authority. Mode=0 updates delay and the 20-complex history while skipping phase rotation, tap multiplication, band0 replacement, and synthesis.

Target and carrier windows each join their own previous 24 slots to the current 24 slots. F05JC may consume the resulting 15x48 target and 5x48 carrier only when eligibility, weights, geometry, and the descriptor after index are same-generation ready. s09_return and final_six are not carrier inputs.

The current whole-binding cases are model-first assertions, not isolated implementation execution. U-QMF-SURROUND-RESET is a mode=1 case and is not mode=0 evidence. A direct mode=0 stage vector is absent, and the approved package does not pin the first-failure priority among provider, callback/lineage, state, and input checks; these remain blockers.

The complete package, provider, state, clock, error, vector, and scope record follows.

## Machine-readable contract

```json
{
  "blockers": [
    {
      "gap": "No selected mode0 direct input/literal vector binds five-lane delay, band0-history update, carrier current/window output, and canonical fitting after-state.",
      "id": "mode0-direct-stage-literal",
      "resolution": "Add an approved mode0 input-only direct-stage vector and post-computation literals before isolated mode0 implementation verification."
    },
    {
      "gap": "The package specifies error categories and fatal outcomes but does not pin the first-failure order among provider, callback/lineage, state, and input checks.",
      "id": "f04rb-error-precedence",
      "resolution": "Obtain an approved precedence rule or explicitly authorize a consumer-local order before implementing error selection; until then, map outcomes without claiming priority."
    }
  ],
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
      "RECEIVER_PARAMETER_UNAVAILABLE for missing required selected parameter/provider",
      "RECEIVER_PARAMETER_INVALID for selected parameter identity, length, or finite-value failure",
      "QMF_PROTOTYPE_UNAVAILABLE for a missing selected QWIN prototype",
      "QMF_PROTOTYPE_INVALID_INPUT for QWIN length or finite-word failure",
      "QMF_PROTOTYPE_IDENTITY_MISMATCH for a QWIN provider identity mismatch",
      "F04ParameterProviderError for the existing typed fixed-FFT provider failure",
      "F04_STATE_ERROR for invalid fitting/receiver before-state schema or range",
      "F04_CALLBACK_ORDER for callback/generation/phase/Type10 lineage mismatch",
      "F04_PIPELINE_RETIRED for a stage presented after programme retirement",
      "F04_INPUT_ERROR for PCM/control/QMF input type, shape, or finite preflight rejection",
      "F04_NUMERIC_FAILURE for a nonfinite numeric intermediate or output after valid preflight"
    ],
    "precedence": "UNSPECIFIED_BY_APPROVED_PACKAGE among provider, lineage/callback, state, and input checks; do not invent a first-failure priority."
  },
  "functional_id": "F04RB",
  "implementation_prohibitions": [
    "no Rust or DSP implementation in this mapping audit",
    "no receiver mode=1 state aliasing or behavior changes",
    "no F04R producer, target producer integration, or receiver routing implementation",
    "no F05JC fit algorithm, J05/F05JL, final-six tail, or parent transaction",
    "no promotion from model-first assertions to isolated execution or product acceptance"
  ],
  "implementation_readiness": "NO_IMPLEMENTATION",
  "implementation_status": "NOT_IMPLEMENTED",
  "input_contract": {
    "callback_index": "F04RB callback_index_zero_based is the receiver index r, not frontend index g; generation=r+1.",
    "carrier_branch": "Re-analyze same-generation s09_input lanes [0,1,2,4,5] from receiver before analysis histories. Raw analysis after must be wordwise equal to receiver analysis after; fitting does not commit receiver analysis history a second time.",
    "descriptor_clip_index": "F05JC consumes this generation's descriptor after selected index/target. Do not derive it from the first-256 ramp current, receiver before gain, s09_return, or final_six.",
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
  "planned_implementation_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/fitting_mode0.rs"
  ],
  "planned_test_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/fitting_mode0/tests.rs"
  ],
  "planned_vector_sources": [
    "test-vectors/QMF-RECEIVER-WHOLE-BINDING-VERIFICATION.json",
    "test-vectors/F04-WHOLE-SPATIAL-CHAIN-VERIFICATION.json"
  ],
  "programme_root": "spatial_audio",
  "readiness_boundary": {
    "direct_mode0_stage_execution": "NOT_RUN",
    "j05_parent": "NOT_RUN",
    "mode0_spec_semantics": "MAPPED",
    "provider_identities": "BOUND_TO_APPROVED_PACKAGE",
    "real_adm_and_product_acceptance": "NOT_RUN",
    "whole_receiver": "NOT_RUN"
  },
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
  "status": "MAPPED_MODE0_RULES__DIRECT_VECTOR_AND_ERROR_PRECEDENCE_BLOCKED",
  "supplied_review_boundary": {
    "current_binding": "The supplied local-review status carries stale package/spec identities and is not reused as a current isolated execution pass.",
    "reported_f04rb_local_ready": "YES",
    "reported_implementation_ready": "NO"
  },
  "task_id": "F04RB-MODE0-FITTING-MAPPING-AUDIT",
  "vector_coverage": {
    "existing_direct_qmf_supplements": {
      "analysis_manifest_sha256": "79830470e3101a10d96852f44c855252c7b78533e0c2b32a73c86d93f74542c6",
      "coverage": "shared QMF analysis and mode1 synthesis only; no independent mode0 fitting delay/FIR/window literal",
      "synthesis_manifest_sha256": "bff23be87f4b7ceb27c145ab6106dd1b786aa2e6dbc35d1e4892da73743e9f4a"
    },
    "mode0_direct_input_and_literal": "NOT_PRESENT",
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
  "whole_receiver_executor": "F04C"
}
```
