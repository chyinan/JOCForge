# Type10 assembly map

由 IMPLEMENTATION-MAP.json 生成，不得手工修改。

Mapping status: `MAPPING_COMPLETE`. Implementation status: `LOCALLY_VERIFIED_TYPE10_ASSEMBLY__RECEIVER_PARENT_DEFERRED`. Receiver mode=1 routing/state status: `LOCALLY_VERIFIED_RECEIVER_MODE1_ROUTING_AND_STATE__CROSS_STAGE_J05_EXECUTION_DEFERRED`; whole F04 receiver, J05 projection and parent transaction remain deferred.

This E4 slice only assembles the E2 component proposal with the E3 delayed LFE row; it does not invoke the separate receiver mode=1 stage or commit spatial_audio state.

```json
{
  "commit_authority": "enclosing callback transaction only; no E4/local commit",
  "error_translation": {
    "authority": "local translation of the existing F04C.7 error categories; no native error-code claim",
    "failure_result": "first applicable local outcome only; no Type10AssemblyProposal or packet; caller-owned committed root unchanged",
    "outcomes": [
      {
        "error_code": "F04_PROFILE_UNSUPPORTED",
        "trigger": "unsupported or proposal/parent profile mismatch"
      },
      {
        "error_code": "F04_CALLBACK_ORDER",
        "trigger": "callback or explicit phase mismatch"
      },
      {
        "error_code": "F04_STATE_ERROR",
        "trigger": "proposal spatial_audio before generation/digest differs from expected parent"
      },
      {
        "error_code": "F04_INPUT_ERROR",
        "trigger": "programme/attempt/transaction/provider lineage mismatch"
      },
      {
        "error_code": "F04_STATE_ERROR",
        "trigger": "missing row0 after priming, row0 origin/ring disagreement, malformed component or LFE output shape, nonfinite row bits"
      }
    ],
    "priority": [
      "selected_profile",
      "callback_phase",
      "spatial_audio_before_generation_digest",
      "cross_proposal_lineage_and_provider",
      "ring_origin_and_payload_shape"
    ],
    "retirement_boundary": "parent/E2/E3 stages reject RETIRED owners before proposals exist; E4 has no programme retirement authority"
  },
  "fatal_behavior": "return the first local error without a packet or stage proposal; parent owns programme retirement",
  "fixture_files": [
    "crates/jocforge-domain/tests/fixtures/direct_lfe/INPUT-ONLY-VECTORS.json"
  ],
  "fixture_inventory": [
    {
      "bytes": 13306,
      "path": "crates/jocforge-domain/tests/fixtures/direct_lfe/INPUT-ONLY-VECTORS.json",
      "sha256": "52c2b1a3b52e53198e47e90e4d6e51e8435fbe660ad996b4005f741107a7eb7f"
    }
  ],
  "functional_scope": [
    {
      "clauses": [
        "F04C.3",
        "F04C.7"
      ],
      "functional_id": "F04C",
      "status": "SCOPED_SUBSET_ONLY"
    }
  ],
  "historical_implementation_inventory": [
    {
      "bytes": 12074,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/type10_assembler.rs",
      "sha256": "22c336456921c174ce1d1e72d4a265b4e61a7321e9c3753cf20d1924e0d77413"
    },
    {
      "bytes": 37303,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state.rs",
      "sha256": "ff4beb3b80aa3d157777244778ef397bc4b86abcb701609bad827d42729acb0b"
    },
    {
      "bytes": 46100,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/component_stage.rs",
      "sha256": "6eb9640e126dd8e6c7e9bbb69782d2901fb38c79343f48089bea2d7b7b227fcc"
    },
    {
      "bytes": 16562,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/direct_lfe.rs",
      "sha256": "bfa2808ba2aa5566cf832a76a5ef93a6180bc1e81c35a63934809f3ffe9abce7"
    },
    {
      "bytes": 26366,
      "path": "crates/jocforge-domain/src/spatial/spatial_control.rs",
      "sha256": "25ddd1832309801765697d7c9e014766e159a5b7571234a86d790c71a895f426"
    }
  ],
  "historical_test_inventory": [
    {
      "bytes": 24485,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/type10_assembler/tests.rs",
      "sha256": "bb9fc89e6914b4b9fb9dc94cd0da47ee83886183661d792957cef352896bab22"
    },
    {
      "bytes": 57070,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/component_stage/tests.rs",
      "sha256": "c6295d0d4e04f6be4c3dc1418cc460d9dcca4b13d744d81e3dd219d62a8c9c53"
    }
  ],
  "implementation_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/type10_assembler.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state/component_stage.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state/direct_lfe.rs",
    "crates/jocforge-domain/src/spatial/spatial_control.rs"
  ],
  "implementation_inventory": [
    {
      "bytes": 12122,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/type10_assembler.rs",
      "sha256": "70cd0b70feee7f5bed86f5c2942fa163c007361a7679df6d4fc584439e625b05"
    },
    {
      "bytes": 55099,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state.rs",
      "sha256": "eda80fadd3964f0faf3f34189da3f9e6b7031032c27dfa395ab2328698758c84"
    },
    {
      "bytes": 48742,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/component_stage.rs",
      "sha256": "9ed4b36687e6694d73716bb774c7773c9b545f93ca49e1025336d355f24b8e2b"
    },
    {
      "bytes": 17481,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/direct_lfe.rs",
      "sha256": "93f15f67ea011a5c078dc78018d57d521a124f2b19a4f18985462c99a73312fa"
    },
    {
      "bytes": 47463,
      "path": "crates/jocforge-domain/src/spatial/spatial_control.rs",
      "sha256": "ca200e1f8aebfd23cc1e4082ffec35f15f73696810cf9ee19d81598cced16e85"
    }
  ],
  "implementation_prohibitions": [
    "no receiver or QMF implementation",
    "no persistent Type10 packet field",
    "no packet Clone/Copy/default/raw constructor",
    "no local commit/abort/fatal authority",
    "no parent CallbackTransaction",
    "no generic row assembler",
    "no JOC/codec logic",
    "no whole-chain/product acceptance promotion"
  ],
  "implementation_status": "LOCALLY_VERIFIED_TYPE10_ASSEMBLY__RECEIVER_PARENT_DEFERRED",
  "input_contract": {
    "component": "linear E2 ComponentStageProposal with 15x1536 finite component output and complete after-state",
    "lfe": "linear E3 LfeStageProposal with complete ring after-state, explicit DirectLfeStageLineage, and optional Type10LfeRow0Input",
    "parent": "SpatialAudioJoinMetadata and explicit CallbackPhase are supplied as expected typed parent identity from the enclosing coordinator; both proposal lineages must match programme, attempt, transaction, callback, phase, profile, spatial_audio before generation/digest, and shared spatial_control provider lineage",
    "phase": "explicit expected NORMAL or CACHED_DRAIN; never inferred from callback index or row content",
    "provider_lineage": "preserve both E2 F04 parameter-provider lineages and the common F01E/F03 spatial_control provider lineage in the immutable assembly receipt"
  },
  "mapping_document": "docs/implementation/TYPE10-ASSEMBLY-MAP.md",
  "mapping_status": "MAPPING_COMPLETE",
  "not_run": [
    "F04 callback integration and parent transaction fault injection",
    "receiver/QMF",
    "real ADM/audio/render/listening acceptance",
    "W04 product acceptance",
    "full song output"
  ],
  "output_contract": {
    "capability": "consume both input proposals once; packet has no public/raw/default constructor, Clone, Copy, or persistent owner field",
    "commit_authority": "none; the parent callback owns any later root replacement",
    "outcome_discriminant": "Type10AssemblyOutcome::Priming/Ready carries the outcome; Type10AssemblyReceipt contains lineage, origins, and fixed shape only",
    "packet_payload": "private immutable Arc<[FiniteF32]> of exactly 16*1536 samples; expose borrowed read-only samples only",
    "priming_outcome": "Priming plus immutable receipt and both preserved after-states; no packet",
    "ready_outcome": "opaque non-Clone Type10Packet plus immutable receipt and both preserved after-states",
    "receipt_fields": [
      "programme_id",
      "attempt",
      "transaction_id",
      "callback_index",
      "CallbackPhase",
      "selected profile",
      "spatial_audio before generation and digest",
      "spatial_control before generation and digest",
      "spatial_control provider lineage",
      "two F04 parameter-provider lineages",
      "validated LFE host identity",
      "component origin callback g",
      "optional LFE origin callback g-2",
      "fixed 16x1536 shape"
    ],
    "stage_type": "linear Type10AssemblyProposal"
  },
  "overall_p1_3_e_status_remains": "BLOCKED_ON_F04_DSP_RENDER_AND_PARENT",
  "planned_implementation_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/type10_assembler.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state/component_stage.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state/direct_lfe.rs",
    "crates/jocforge-domain/src/spatial/spatial_control.rs"
  ],
  "planned_test_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/type10_assembler/tests.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state/component_stage/tests.rs"
  ],
  "planned_test_vector_fixtures": [
    "crates/jocforge-domain/tests/fixtures/direct_lfe/INPUT-ONLY-VECTORS.json"
  ],
  "programme_root": "spatial_audio",
  "required_e2_lineage_seam": {
    "current_gap": "ComponentStageReceipt does not retain SpatialAudioJoinMetadata.before_generation or before_digest and exposes no read-only lineage accessors",
    "minimal_change": "store the E2 spatial_audio before generation and digest in ComponentStageReceipt and expose crate-internal read-only getters; do not change E2 arithmetic or commit authority"
  },
  "reset_authority": "none; programme-origin reset remains with the enclosing owner coordinator",
  "ring_agreement": {
    "priming": "for g<2, row0 is absent and the ring after-state remains the only LFE result",
    "required": "for g>=2, row0 callback_generation equals g, origin_generation equals g-2, and exactly one matching after-ring slot has that origin and identical row bytes"
  },
  "scope": "Consume one same-callback E2 ComponentStageProposal and E3 LfeStageProposal, stage Type10 row assembly for receiver handoff, and preserve both after-states for the parent. Do not implement receiver/QMF, local commit, parent transaction, JOC, or codec.",
  "specification": {
    "clauses": [
      "F04C.3",
      "F04C.7"
    ],
    "package_manifest_sha256": "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a",
    "priming": "g=0 and g=1 return Priming and no Type10Packet; g>=2 requires one row0 with origin g-2",
    "receiver_qmf": "not implemented by E4",
    "rows": "Type10[g] is F32[16][1536]; row0 is LFE[g-2] and rows1..15 are component15[g][0..14], copied in row-major order without numeric transformation",
    "spec_bytes": 9103,
    "spec_file": "specs/04-whole-spatial-chain-composition.md",
    "spec_sha256": "92325b12128023001e1f3fd4696dcec68bbd47a46acdb7886781b411a10c5d59"
  },
  "stage_authority": "F04C E4 callback-local assembly only; consumes E2/E3 proposals and preserves their after-states",
  "status": "LOCALLY_VERIFIED_TYPE10_ASSEMBLY__RECEIVER_PARENT_DEFERRED",
  "task_id": "P1.3-E4-F04-TYPE10-ASSEMBLY",
  "test_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/type10_assembler/tests.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state/component_stage/tests.rs"
  ],
  "test_inventory": [
    {
      "bytes": 24485,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/type10_assembler/tests.rs",
      "sha256": "bb9fc89e6914b4b9fb9dc94cd0da47ee83886183661d792957cef352896bab22"
    },
    {
      "bytes": 61890,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/component_stage/tests.rs",
      "sha256": "9275736d43853f6b137abdeeda5ae4aa4530acda241eacad02dcd28f22f7d555"
    }
  ],
  "vectors": {
    "binding_rule": "E4 asserts every output sample bit equals the corresponding consumed proposal row bit; no whole-chain Type10 expected hash is used as an assembly input or oracle",
    "callbacks": [
      "g0",
      "g1",
      "g2",
      "g3",
      "g4",
      "g5"
    ],
    "cases": [
      "g0_g1_priming_without_packet",
      "g2_g5_row_order_origins_and_bit_preservation",
      "component_lfe_programme_attempt_mismatch",
      "callback_phase_mismatch",
      "transaction_mismatch",
      "profile_mismatch",
      "provider_lineage_mismatch",
      "stale_spatial_audio_before_generation",
      "stale_spatial_audio_before_digest",
      "missing_row0_after_priming",
      "row0_origin_or_ring_slot_mismatch",
      "malformed_component_output_shape",
      "dropped_stage_preserves_committed_root"
    ],
    "component_fixture": "E4 test-only ComponentStageProposal rows with unique finite F32 bit markers; E4 makes no E2 DSP/vector claim",
    "expected_values_are_assertions_only": true,
    "lfe_fixture": "crates/jocforge-domain/tests/fixtures/direct_lfe/INPUT-ONLY-VECTORS.json; consume only the g0..g5 callback/phase/origin assertions"
  },
  "verification": {
    "boundary_normal": "PASS",
    "boundary_optimized": "PASS",
    "cargo_check": "PASS",
    "cargo_clippy": "PASS",
    "cargo_fmt": "PASS",
    "full_repository_boundary_mutations_rejected": 685,
    "implementation_map_render": "PASS",
    "local_commit": "NOT_IMPLEMENTED",
    "parent_transaction": "NOT_IMPLEMENTED",
    "planned_test_count": 13,
    "receiver_qmf": "NOT_IMPLEMENTED",
    "targeted_implementation_mutations_rejected": 36,
    "targeted_mapping_mutations_rejected": 20,
    "type10_assembly_test_count": 13,
    "type10_packet": "LOCALLY_VERIFIED_CALLBACK_LOCAL_NONCLONE",
    "workspace_test_ignored": 1,
    "workspace_test_passed": 209
  }
}
```
