# PostWindow fanout map

由 IMPLEMENTATION-MAP.json 生成，不得手工修改。

Status: `LOCALLY_VERIFIED__RENDER_PRODUCER_DEFERRED`. The render producer remains deferred, as do matrix/destination production, F04 assembly, and the parent transaction.

The linear proposal carries one immutable Arc payload. Its two consumer views share that allocation and carry an explicit callback phase.

```json
{
  "boundary": "INSTALLED48_POST_WINDOW_48X3072",
  "current_features_input": {
    "algorithm_changed": false,
    "callback_phase_input": "explicit CallbackPhase; never inferred from sample values",
    "lineage_validation": [
      "programme",
      "attempt",
      "transaction",
      "callback",
      "phase",
      "profile",
      "provider lineage",
      "spatial_control before generation/digest"
    ],
    "local_commit": false,
    "none_behavior": "Allowed only when every source flag is RESET or CARRY; if any source is READ (00), reject with ReadAudioMissing before state or output proposal",
    "some_behavior": "Always validate phase, programme, attempt, transaction, callback, profile, provider lineage, and spatial_control before generation/digest, including when no source reads samples",
    "type": "Option<CurrentFeaturesAudioView>",
    "unused_view_behavior": "The parent may drop an unconsumed CurrentFeaturesAudioView when no source is READ; dropping the view does not commit owner state"
  },
  "fanout": {
    "operation": "fanout(self)",
    "returns_exactly": [
      "CurrentFeaturesAudioView",
      "SpatialAudioPostWindowView"
    ],
    "shared_payload": "both views use the proposal's same immutable Arc allocation; only the Arc handle is cloned",
    "views_clone_copy": false
  },
  "future_spatial_audio_assembler": {
    "additional_same_callback_inputs": [
      "matrix",
      "destinations",
      "LFE inputs"
    ],
    "implemented": false,
    "input_view": "SpatialAudioPostWindowView"
  },
  "implementation_files": [
    "crates/jocforge-domain/src/spatial/post_window_fanout.rs",
    "crates/jocforge-domain/src/spatial/current_features.rs",
    "crates/jocforge-domain/src/spatial/mod.rs"
  ],
  "implementation_status": "LOCALLY_VERIFIED__RENDER_PRODUCER_DEFERRED",
  "mapping_document": "docs/implementation/POSTWINDOW-FANOUT-MAP.md",
  "mapping_status": "MAPPING_COMPLETE",
  "not_run": [
    "render production",
    "matrix/destination production",
    "SpatialAudioInputPacket assembler",
    "F04 DSP",
    "parent transaction",
    "real ADM/audio/runtime/listening acceptance"
  ],
  "phase": [
    "NORMAL",
    "CACHED_DRAIN"
  ],
  "proposal": {
    "clone_copy": false,
    "lineage": [
      "programme",
      "attempt",
      "CallbackLineageIdentity",
      "FrontendCallbackIndex",
      "CallbackPhase",
      "selected EncodingProfile",
      "validated F01E/F03 provider lineage",
      "spatial_control before generation and complete root digest",
      "typed render-stage identity",
      "typed spatial-control after-proposal identity",
      "payload byte count and SHA-256"
    ],
    "payload_byte_count": 589824,
    "payload_sha256": "SHA-256 over exact source-major F32 U32LE bytes",
    "payload_type": "Arc<[FiniteF32]>",
    "production_constructor": "future spatial_control/F03W renderer only; requires opaque SpatialControlRenderAuthority minted only inside spatial_control; no public raw/default constructor",
    "shape": [
      48,
      3072
    ],
    "source_major": true,
    "type": "linear PostWindowRenderProposal"
  },
  "status": "LOCALLY_VERIFIED__RENDER_PRODUCER_DEFERRED",
  "status_non_promotion": {
    "CURRENT_FEATURES_PRODUCER": "LOCALLY_VERIFIED_MATH_AND_CAPABILITY__RENDER_AND_PARENT_DEFERRED",
    "P1.3-E": "BLOCKED_ON_F04_DSP_RENDER_AND_PARENT",
    "P1.3-E2": "LOCALLY_VERIFIED_COMPONENT_PRODUCER__UPSTREAM_AND_PARENT_DEFERRED"
  },
  "supplement": {
    "api_schema_bytes": 732,
    "api_schema_file": "API-SCHEMA.json",
    "api_schema_sha256": "1ebf6145695ff88013345b9d68aec3266854d8f482351a10c40ded0a3a5134a1",
    "contract_bytes": 2835,
    "contract_file": "CONTRACT.md",
    "contract_sha256": "b04df4c2badee8cb189c5361ab5a42e248af071e1c92109913a733e1f2a4c729",
    "identity": "postwindow-fanout-contract.v1",
    "manifest_sha256": "2a55e66314d6f2aa37cf792b62068185358b1a4b294e6c7b95e8b8d77d99f107",
    "root": "outputs/IMPLEMENTER_SUPPLEMENTS/POSTWINDOW-FANOUT-CONTRACT-V1"
  },
  "task_id": "POSTWINDOW-FANOUT-FOUNDATION-V1",
  "test_files": [
    "crates/jocforge-domain/src/spatial/post_window_fanout/tests.rs",
    "crates/jocforge-domain/src/spatial/current_features/tests.rs"
  ],
  "test_scope": [
    "fixed shape and finite sample admission",
    "source-major payload byte count and SHA-256",
    "explicit normal/cached-drain phase",
    "one-time two-view fanout",
    "shared Arc payload allocation identity",
    "different proposal identities despite equal payload hashes",
    "lineage and view mismatch",
    "consumer error drops capabilities without state commit",
    "capabilities remain non-Clone/non-Copy"
  ],
  "verification": {
    "compile_fail_contracts": 5,
    "current_features_tests": 9,
    "full_repository_boundary_mutations_rejected": 585,
    "matrix_destination_producer": "NOT_IMPLEMENTED",
    "parent_transaction": "NOT_IMPLEMENTED",
    "post_window_fanout_tests": 5,
    "render_producer": "NOT_IMPLEMENTED",
    "spatial_audio_assembler": "NOT_IMPLEMENTED",
    "targeted_boundary_mutations_rejected": 32
  }
}
```
