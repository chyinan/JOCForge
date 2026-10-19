# Receiver mode=1 route and state map

由 IMPLEMENTATION-MAP.json 生成，不得手工修改。

Mapping status: `MAPPING_COMPLETE`. Implementation readiness: `LOCALLY_VERIFIED_RECEIVER_MODE1_ROUTING_AND_STATE__CROSS_STAGE_J05_EXECUTION_DEFERRED`.

All five carriers `[L,R,C,Ls,Rs]` run 24-slot analysis on receiver lanes `[0,1,2,4,5]`. Only surround lanes `[4,5]` run mode=1 delay/FIR/synthesis. L/R/C update analysis history only in this audio-QMF path; lane 3 LFE bypasses QMF.

Frontend callback `g` remains distinct from receiver count `r`: `g<2` is Type10 priming and does not advance receiver/QMF; for `g>=2`, `r=g-2`, receiver after-generation is `r+1=g-1`, and JOC dispatch is `d=g-1`. The callback count is proposed only with the complete `spatial_audio` root.

Fresh review R1 is bound to the current package and passes 44/44 static checks. Local synthetic tests verify the receiver routing/state stage. The older review mismatches remain recorded as stale identities. F04R production, same-runner F05R56 targets, J05 projection, parent transaction and product acceptance remain NOT_RUN.

## Machine-readable contract

```json
{
  "blockers": [
    {
      "id": "refresh-current-review-bindings",
      "reason": "R1 verifies the current package and F04RB identities, and separately reviews the new carrier/owner/J05 static contract. The prior stale identities remain recorded above.",
      "resolution": "F04-CURRENT-FRESH-REVIEW-R1, 44/44 static checks, manifest c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a; cross-stage execution remains NOT_RUN.",
      "status": "RESOLVED_BY_CURRENT_PACKAGE_FRESH_STATIC_REVIEW"
    }
  ],
  "clock_contract": {
    "cached_drain": "Run receiver/QMF for each actual cached-drain callback with g>=2 and a Ready Type10Packet. Full selected-profile drains are g=4247..4250. Do not append a synthetic QMF flush callback.",
    "count_transition": "On one successful F04C receiver callback, advance receiver_callback_count exactly once from r to r+1. It is a proposal in the full spatial_audio after-root, not a QMF-local commit.",
    "frontend_callback": "g is zero-based and advances on each successful frontend callback, including callbacks where Type10 is Priming.",
    "joc_dispatch": "For g>=2, d=r+1=g-1. First route is g=2, r=0, receiver generation=1, d=1.",
    "priming": "At g=0/1 E4 emits Priming with no Type10Packet; component and LFE stages advance, receiver/QMF does not run, and receiver_callback_count remains 0.",
    "receiver_callback": "For g>=2, r=g-2 is the zero-based ReceiverCallbackCount before this receiver call; receiver generation/count after success is r+1=g-1."
  },
  "executor_and_receipt_producer": "F04C",
  "fresh_review_resolution": {
    "cross_stage_execution": "NOT_RUN",
    "current_f04rb_sha256": "104d604cf876cc522a85b21f352bd0a09fc20c3171ddcd71da981c8076ea3a75",
    "current_package_manifest_sha256": "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a",
    "manifest_sha256": "75e7313563c37f79a9afb895704b5eb77e6dda55d41117f820e62c445bbe4999",
    "old_review_record_policy": "Retain prior mismatches above as STALE identity evidence; do not count them as current PASS.",
    "review_sha256": "926c8a3c9cd21e68665ac287ad98403481b55adf3cdeef6db7660c0f4fb37a60",
    "scope": [
      "current numerical providers and vectors are identity-checked and unchanged",
      "receiver state schema and receiver/source/dispatch clocks are unchanged",
      "carrier order, receiver lanes, unique F04C executor, and selected-J05 receipt rules received fresh static review"
    ],
    "source": "outputs/IMPLEMENTER_SUPPLEMENTS/F04-CURRENT-FRESH-REVIEW-R1",
    "static_checks": {
      "passed": 44,
      "total": 44
    },
    "status_sha256": "4aa0f1d9f0fbc718772193159069077618e0c0138c8002798c893d29f0296fb8",
    "verdict": "SCOPED_NUMERIC_AND_STATE_IMPLEMENTATION_ALLOWED; CROSS_STAGE_INTEGRATION_NOT_READY"
  },
  "implementation_binding": {
    "api": "stage_receiver_mode1 consumes one linear Type10AssemblyProposal and a typed F04R stage-3 ReceiverS09InputView; it returns a linear proposal with preserved E2/E3 after-state, Type10 outcome, five raw analysis spectra, surround PCM, and QMF after slices.",
    "implementation_files": [
      {
        "bytes": 17166,
        "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/receiver_mode1.rs",
        "sha256": "5362379d8c33a2c7ae349c34d1836a083c453c43b61f191a075ecc8f4afd3aca"
      },
      {
        "bytes": 35306,
        "path": "crates/jocforge-domain/src/spatial/spatial_audio_state.rs",
        "sha256": "794c5980f2c6f29f8aad45768a6a8b14e4fc1992b2715171a5f747afdaf4f05b"
      }
    ],
    "s09_lineage_type": "ReceiverS09InputView binds six finite 1536-sample lanes to SpatialAudioJoinMetadata and explicit CallbackPhase; the receiver stage compares it with the Type10 receipt before any QMF processing.",
    "test_cases": [
      "g0_g1_priming_preserves_component_lfe_and_skips_receiver_qmf",
      "normal_receiver_callback_analyzes_five_carriers_and_synthesizes_only_surrounds",
      "cached_drain_consumes_warm_histories_once_and_advances_receiver_count_once",
      "stale_receiver_count_rejects_before_returning_after_state",
      "phase_and_type10_callback_mismatch_reject_before_qmf_work",
      "priming_and_ready_input_shapes_are_checked_before_qmf_work"
    ],
    "test_count": 6,
    "test_files": [
      {
        "bytes": 23303,
        "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/type10_assembler/tests/receiver_mode1.rs",
        "sha256": "9bc20e80319c4a5df182a7937f69b28587de85fb3c037825890e300f3c117d53"
      },
      {
        "bytes": 23795,
        "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/type10_assembler/tests.rs",
        "sha256": "5d7bfe329d68793d05e5587c9383eda0c7d503277ec31ba9a00c0221ffe5a043"
      }
    ]
  },
  "implementation_exclusions": [
    "F04R preprocessing before s09_input",
    "descriptor and threshold selection",
    "mode=0 fitting and QmfWindowState fitting histories",
    "post-QMF receiver temporal/descriptor/final-six output tail",
    "J05/F05JL implementation",
    "parent CallbackTransaction and spatial_audio commit",
    "real ADM/listening/product acceptance"
  ],
  "implementation_readiness": "LOCALLY_VERIFIED_RECEIVER_MODE1_ROUTING_AND_STATE__CROSS_STAGE_J05_EXECUTION_DEFERRED",
  "input_contract": {
    "parent_identity": "FrontendCallbackIndex g and explicit CallbackPhase from the enclosing callback; never infer phase or generation from hashes or samples.",
    "qmf_boundary": "F04R stage 3 supplies a typed ReceiverS09InputView over s09_input[6][1536], bound to the same SpatialAudioJoinMetadata and explicit phase as Type10. QMF analyzes lanes [0,1,2,4,5]; lane 3 is LFE and bypasses QMF. This slice does not implement F04R or produce s09_input.",
    "target_updates": "The full F04R callback requires same-g immutable target_updates[16][6] from the F05R56 target-owner receipt before row/lane accumulation. This receiver mode=1 stage begins at S09 and does not consume target_updates directly; same-runner F05R56 production remains NOT_RUN.",
    "type10": "For g>=2 consume the E4 Ready outcome as a packet-plus-receipt pair. Type10Packet.callback_index is g; shape is 16x1536 F32; row0 is LFE origin g-2; rows1..15 are component output from g."
  },
  "lane_routing": {
    "analysis_history_rows": [
      "L",
      "R",
      "C",
      "Ls",
      "Rs"
    ],
    "analysis_lanes": [
      0,
      1,
      2,
      4,
      5
    ],
    "analysis_only_receiver_audio_lanes": [
      0,
      1,
      2
    ],
    "carrier_order": [
      "L",
      "R",
      "C",
      "Ls",
      "Rs"
    ],
    "carrier_receiver_lanes": [
      0,
      1,
      2,
      4,
      5
    ],
    "front_lane_contract": "L/R/C update only their receiver analysis histories in this audio-QMF path. They do not run receiver mode=1 delay/FIR/synthesis. A separate mode=0 fitting reanalysis is excluded.",
    "lfe_lane_contract": "Lane 3 bypasses QMF; its F04R LFE post-filter is outside this map.",
    "mode1_delay_fir_synthesis_lanes": [
      4,
      5
    ],
    "six_lane_order": [
      "L",
      "R",
      "C",
      "LFE",
      "Ls",
      "Rs"
    ],
    "slot_contract": "Each selected lane has 24 ordered 64-sample slots. Analysis produces 24x64 complex bands (24x128 interleaved F32 words). Only lane 4/5 mode=1 output replaces those scratch lanes with 1536 synthesized PCM samples.",
    "surround_state_rows": [
      "Ls",
      "Rs"
    ]
  },
  "mapping_document": "docs/implementation/RECEIVER-MODE1-MAP.md",
  "mapping_status": "MAPPING_COMPLETE",
  "package_identity": {
    "manifest_sha256": "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a"
  },
  "persistent_owner": "spatial_audio",
  "programme_root": "spatial_audio",
  "provider_bindings": [
    {
      "bytes": 20325,
      "file": "parameters/qmf-receiver-functional-v1.json",
      "selected_fields": [
        "qwin_synthesis_640",
        "qwin_analysis_640 is the reverse of qwin_synthesis_640",
        "band0_real_mode1_21",
        "band0_imag_mode1_21"
      ],
      "sha256": "30e63cc110b63dd88118ad37f02750a39167bd01279a3b40853afdf3151b552c"
    },
    {
      "bytes": 6983,
      "file": "parameters/qmf-mathematical-fft-v1.json",
      "selected_fields": [
        "63 fixed FFT root pairs",
        "rotation_u32[128]"
      ],
      "sha256": "e8d3fb1514a5ffd857740222eacc8995208a29c3447e48c1b63083af368c0431"
    },
    {
      "bytes": 2704,
      "file": "parameters/window-q15.json",
      "selected_fields": [
        "receiver_blend32[32] for upstream F04R.5; not consumed by QMF_CORE"
      ],
      "sha256": "a12f9cbf91f1cf540c2eae77e4d236970defa70409f00ddf62ba87bfeafbb130"
    }
  ],
  "readiness_basis": "The current package fresh review authorizes the scoped routing/state slice. Six local tests verify five-lane analysis, surround-only mode=1 synthesis, priming, warm cached transition, lineage and input errors. This status does not claim F04RB/F04C/J05 cross-stage execution, same-runner F05R56 targets, parent transaction, real ADM/listening, or product acceptance.",
  "review_binding_mismatches": [
    {
      "current_package_sha256": "104d604cf876cc522a85b21f352bd0a09fc20c3171ddcd71da981c8076ea3a75",
      "file": "specs/04-qmf-receiver-whole-binding.md",
      "review_status": "test-vectors/QMF-RECEIVER-WHOLE-BINDING-LOCAL-REVIEW-STATUS.json",
      "reviewed_sha256": "254e2a0427901ec5f715d40fc9f682b74e1c8d2f2193ff18366d02d3847cd75e"
    },
    {
      "current_package_sha256": "92325b12128023001e1f3fd4696dcec68bbd47a46acdb7886781b411a10c5d59",
      "file": "specs/04-whole-spatial-chain-composition.md",
      "review_status": "test-vectors/F04-WHOLE-SPATIAL-CONSUMER-LOCAL-REVIEW-STATUS.json",
      "reviewed_sha256": "80975e186d7be6b717e9257e28f87c922ca9a2b845b728bd97f595a594c375de"
    },
    {
      "current_package_sha256": "bc294fd3d2a4cbe25ca7bb9206fd0ef91663213a81ab93c10fd7df84198d595a",
      "file": "test-vectors/F04-WHOLE-SPATIAL-GENERATION-JOIN-AUDIT.json",
      "review_status": "test-vectors/F04-WHOLE-SPATIAL-CONSUMER-LOCAL-REVIEW-STATUS.json",
      "reviewed_sha256": "5c68e9757a036236495791ab3af67aa1bcab044361419273dbc70a18781f978e"
    },
    {
      "current_package_sha256": "b3389286e8308e0be04cce3e17a1e8fb1accbbeb2d9530d27d881b71110c3a7a",
      "file": "test-vectors/F04-WHOLE-SPATIAL-GAP-INVENTORY.json",
      "review_status": "test-vectors/F04-WHOLE-SPATIAL-CONSUMER-LOCAL-REVIEW-STATUS.json",
      "reviewed_sha256": "587489a691ea10e16a50de641c3650c883c397fa927c1c262824561d4e96d80c"
    }
  ],
  "same_callback_lineage": {
    "component_E2": "ComponentStageReceipt binds programme, attempt, transaction, g, phase, profile, spatial_audio before generation/digest, spatial_control before generation/digest/provider lineage, and F04 provider lineage.",
    "future_selected_j05": "F04C alone produces selected-j05-owner-receipt.v1 for g>=2. Preserve g, r=g-2, d=g-1, component/LFE/target input owner tokens and hashes, Type10 shape 16x1536 F32, final-six shape 6x1536 F32 (its producer is excluded here), target-QMF shape 5x48x192 complex-F32, carrier-QMF shape 5x48x64 complex-F32, and receiver/QMF before/after generations and hashes. F05JL consumes this projection and must not rerun F04R/F04RB. This audit creates no such receipt.",
    "lfe_E3": "DirectLfeStageLineage binds the same parent identity and before-state/provider lineage plus semantic LFE host. Type10LfeRow0Input binds packet callback g and delayed row origin g-2.",
    "qmf_core": "Current QMF_CORE is a pure numeric stage API with before/after histories only; it carries no g, phase, profile, transaction, or Type10 receipt.",
    "receiver_mode1_stage": "ReceiverS09InputView carries the same parent metadata plus explicit phase and six finite lane slices. The receiver stage validates that view and the E4 Type10 receipt against one parent before analysis. The linear stage proposal returns component/LFE after-state, the Type10 outcome, QMF after slices, and mode=1 surround PCM; it creates no owner receipt and cannot commit.",
    "type10_E4": "Type10AssemblyReceipt retains the E2/E3 lineages, component origin g, optional LFE origin g-2, and 16x1536 shape. Ready returns packet and receipt together; Priming has no packet."
  },
  "specifications": [
    {
      "bytes": 13595,
      "clauses": [
        "F04RB.1",
        "F04RB.2",
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
        "F04R.2"
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
      "bytes": 4504,
      "clauses": [
        "Callback split",
        "J05 receipt",
        "Source timing"
      ],
      "file": "specs/00-selected-owner-projection-and-j05.md",
      "sha256": "9ff01aca7b6be6e89511e4d493f9fc9e661a50e2f098065f4a32c3fed777542f"
    }
  ],
  "state_transition": {
    "callback_count_field": "F04CompositionState.receiver_callback_count: ReceiverCallbackCount",
    "cold_count": 0,
    "commit_authority": "Only the enclosing spatial_audio root callback may accept these after slices. QMF_CORE has no owner, receipt, local commit, abort, or retirement authority.",
    "failure_effect": "Reject the callback result; discard all staged F04C after slices and let the enclosing I03/GTC owner retire the programme. Never commit a QMF-only partial state.",
    "fields": [
      {
        "advance": "one [x,H0,...,H7] shift per lane per 64-sample slot",
        "cold": "positive zero",
        "field": "ModelState.analysis_history",
        "lane_order": [
          "L",
          "R",
          "C",
          "Ls",
          "Rs"
        ],
        "shape": "5x9x64 F32"
      },
      {
        "advance": "mode=1 delayed X, retaining the last 10 unrotated input slots",
        "cold": "positive zero",
        "field": "ModelState.qmf_delay_history",
        "lane_order": [
          "Ls",
          "Rs"
        ],
        "shape": "2x10x128 F32"
      },
      {
        "advance": "mode=1 band0 FIR uses current analysis X; keep final 20 complex history values",
        "cold": "positive zero",
        "field": "ModelState.qmf_fir_history",
        "lane_order": [
          "Ls",
          "Rs"
        ],
        "shape": "2x40 F32 = 2x20 complex"
      },
      {
        "advance": "one synthesis polyphase transition per processed slot",
        "cold": "positive zero",
        "field": "ModelState.synthesis_history",
        "lane_order": [
          "Ls",
          "Rs"
        ],
        "shape": "2x576 F32"
      }
    ],
    "reset_authority": "Programme-origin full spatial_audio reset reconstructs all four QMF state families from the declared positive-zero cold state.",
    "root_field": "F04CompositionState.receiver_state: ModelState"
  },
  "status": "LOCALLY_VERIFIED_RECEIVER_MODE1_ROUTING_AND_STATE__CROSS_STAGE_J05_EXECUTION_DEFERRED",
  "task_id": "F04RB-RECEIVER-MODE1-ROUTING-STATE-MAP",
  "vector_coverage": {
    "direct_qmf_vectors": {
      "analysis_manifest_sha256": "79830470e3101a10d96852f44c855252c7b78533e0c2b32a73c86d93f74542c6",
      "coverage": "Cold and warm core stages; mode=1 one supplied surround lane over 24 slots; not five-lane receiver routing.",
      "synthesis_manifest_sha256": "bff23be87f4b7ceb27c145ab6106dd1b786aa2e6dbc35d1e4892da73743e9f4a"
    },
    "error_coverage": "QMF core tests cover nonfinite input, numeric overflow, and provider identity mismatch. The local receiver stage tests stale receiver count, Type10/S09 callback and phase mismatch, priming/ready input presence, and malformed S09 lane shape. E4 covers same-parent/phase/origin mismatch; whole consumer model lists 11 preflight rejects. Full F04C parent failure/retirement remains NOT_RUN.",
    "qmf_receiver_whole_binding": {
      "cases": 5,
      "coverage": "Cold-to-warm continuous receiver/QMF/target/fitting model; 46080 receiver output words, 14400 carrier analysis-history joins; hashes are assertions, not seeds.",
      "execution_status": "MODEL_FIRST_SUPPLIED_REVIEW_ONLY_NOT_ISOLATED_RECEIVER_EXECUTION",
      "file": "test-vectors/QMF-RECEIVER-WHOLE-BINDING-VERIFICATION.json",
      "sha256": "1d809ed24c805a182e9f2ba668562f1f3c8028e4dfc0ce7cd039e1255e4cda07"
    },
    "whole_spatial_chain": {
      "cases": 6,
      "coverage": "g=0..5; two NORMAL then four CACHED_DRAIN callbacks; receiver runs at g=2..5; 11 preflight rejection cases; target/carrier QMF values are whole-chain hashes, not QMF-step literals.",
      "execution_status": "MODEL_FIRST_SUPPLIED_REVIEW_ONLY_NOT_ISOLATED_RECEIVER_EXECUTION",
      "file": "test-vectors/F04-WHOLE-SPATIAL-CHAIN-VERIFICATION.json",
      "sha256": "ffb1dea3b1e69383312da4059b9d8ddabb4366ed8fc22e7537f750ba041f2d63"
    }
  },
  "verification": {
    "full_repository_boundary_mutations_rejected": 780,
    "listening": "NOT_RUN",
    "real_adm": "NOT_RUN",
    "receiver_mode1_stage": "LOCAL_SYNTHETIC_TESTS_PASS",
    "receiver_mode1_test_count": 6,
    "targeted_boundary_mutations_rejected": 8,
    "whole_receiver_qmf_execution": "NOT_RUN",
    "whole_spatial_chain_implementation_execution": "NOT_RUN"
  }
}
```
