# Type10 assembly map

由 IMPLEMENTATION-MAP.json 生成，不得手工修改。

Mapping status: `MAPPING_COMPLETE`. Implementation status: `NOT_STARTED`. Receiver/QMF and parent transaction remain deferred.

This slice only assembles the E2 component proposal with the E3 delayed LFE row; it does not execute receiver/QMF or commit spatial_audio state.

```json
{
  "commit_authority": "enclosing callback transaction only; no E4/local commit",
  "error_translation": {
    "authority": "local translation of the existing F04C.7 error categories; no native error-code claim",
    "failure_result": "first applicable local outcome only; no Type10AssemblyProposal or packet; caller-owned committed root unchanged",
    "outcomes": [
      {
        "error_code": "F04_PIPELINE_RETIRED",
        "trigger": "retired spatial_audio owner"
      },
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
      "retired",
      "selected_profile",
      "callback_phase",
      "spatial_audio_before_generation_digest",
      "cross_proposal_lineage_and_provider",
      "ring_origin_and_payload_shape"
    ]
  },
  "fatal_behavior": "return the first local error without a packet or stage proposal; parent owns programme retirement",
  "fixture_files": [],
  "fixture_inventory": [],
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
  "implementation_files": [],
  "implementation_inventory": [],
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
  "implementation_status": "NOT_STARTED",
  "input_contract": {
    "component": "linear E2 ComponentStageProposal with 15x1536 finite component output and complete after-state",
    "lfe": "linear E3 LfeStageProposal with complete ring after-state, explicit DirectLfeStageLineage, and optional Type10LfeRow0Input",
    "parent": "SpatialAudioJoinMetadata and explicit CallbackPhase are supplied as expected typed parent identity; both proposal lineages must match programme, attempt, transaction, callback, phase, profile, spatial_audio before generation/digest, and shared spatial_control provider lineage",
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
      "component origin callback g",
      "optional LFE origin callback g-2",
      "assembly outcome and fixed 16x1536 shape"
    ],
    "stage_type": "linear Type10AssemblyProposal"
  },
  "overall_p1_3_e_status_remains": "BLOCKED_ON_F04_DSP_RENDER_AND_PARENT",
  "planned_implementation_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/type10_assembler.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state/component_stage.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state/direct_lfe.rs"
  ],
  "planned_test_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/type10_assembler/tests.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state/component_stage/tests.rs"
  ],
  "planned_test_vector_fixtures": [
    "crates/jocforge-domain/tests/fixtures/f04_component/F04-WHOLE-SPATIAL-CHAIN-VERIFICATION.json",
    "crates/jocforge-domain/tests/fixtures/direct_lfe/INPUT-ONLY-VECTORS.json",
    "crates/jocforge-domain/tests/fixtures/direct_lfe/G1-VECTOR-ERRATUM.json"
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
  "status": "MAPPING_COMPLETE",
  "task_id": "P1.3-E4-F04-TYPE10-ASSEMBLY",
  "test_files": [],
  "test_inventory": [],
  "vectors": {
    "binding_rule": "E4 asserts every output sample bit equals the corresponding consumed proposal row bit; the whole-chain expected Type10 hashes are not cross-paired with the different Direct-LFE g1 row",
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
    "component_fixture": "crates/jocforge-domain/tests/fixtures/f04_component/F04-WHOLE-SPATIAL-CHAIN-VERIFICATION.json; use only the E2 component input/row identity; do not take its Type10 expected hash when pairing with the distinct Direct-LFE g1 supplement row",
    "expected_values_are_assertions_only": true,
    "lfe_fixture": "crates/jocforge-domain/tests/fixtures/direct_lfe/INPUT-ONLY-VECTORS.json and G1-VECTOR-ERRATUM.json"
  },
  "verification": {
    "full_repository_boundary_mutations_rejected": "NOT_RUN",
    "local_commit": "NOT_IMPLEMENTED",
    "parent_transaction": "NOT_IMPLEMENTED",
    "planned_test_count": 13,
    "receiver_qmf": "NOT_IMPLEMENTED",
    "targeted_implementation_mutations_rejected": "NOT_RUN",
    "targeted_mapping_mutations_rejected": 19,
    "type10_packet": "NOT_IMPLEMENTED"
  }
}
```
