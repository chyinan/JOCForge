# Direct-LFE ring map

由 IMPLEMENTATION-MAP.json 生成，不得手工修改。

Mapping status: `MAPPING_COMPLETE`. Implementation status: `LOCALLY_VERIFIED_DIRECT_LFE__UPSTREAM_PARENT_DEFERRED`. The full F04 chain and parent transaction remain blocked.

This is only the direct-LFE ring transition and a future Type10 row0 input; it does not construct Type10 or execute receiver/QMF.

```json
{
  "clock_contract": {
    "callback_advance": "each successful actual NORMAL or CACHED_DRAIN callback advances the ring once and adds exactly 1536 model samples",
    "downstream_clocks": "receiver r=g-2, receiver after generation g-1, JOC dispatch d=g-1; E3 does not execute or advance receiver/QMF/JOC",
    "full_profile_counts": {
      "actual_callbacks": 4251,
      "cached_drains": 4,
      "normal_callbacks": 4247
    },
    "isolated_vector_trace": "the six supplement cases g0..g5 explicitly label g0/g1 NORMAL and g2..g5 CACHED_DRAIN for a bounded ring trace only; this does not override production phase ranges or imply a second profile clock",
    "output_origin": "for g>=2, row0 origin is source generation g-2",
    "priming": "g=0 and g=1 produce no LFE output row",
    "production_cached_drain": "g=4247..4250",
    "production_domain": "FrontendCallbackIndex newtype supplied by the parent; phase is explicit and never inferred from the numeric generation",
    "production_normal": "g=0..4246",
    "source_origin": "s=g-2 only for g>=2",
    "type10": "for g>=2, Type10 row0 is LFE[g-2], rows1..15 are component15[g] from P1.3-E2; E3 emits row0 input only and does not assemble Type10"
  },
  "commit_authority": "enclosing CallbackTransactionCoordinator root replacement only; no F04C/local E3 commit",
  "eof_drain": "process only actual callbacks, including the four production cached drains; stop after the fourth drain; no synthetic LFE or QMF flush",
  "error_translation": {
    "authority": "WASH_APPROVED_LOCAL_TRANSLATION_2026-10-18; no native/package error-code claim",
    "failure_result": "first applicable local outcome only; no proposal; committed state unchanged",
    "outcomes": [
      {
        "error_code": "F04_PIPELINE_RETIRED",
        "trigger": "retired owner/programme"
      },
      {
        "error_code": "F04_PROFILE_UNSUPPORTED",
        "trigger": "unsupported or mismatched selected profile"
      },
      {
        "error_code": "F04_CALLBACK_ORDER",
        "trigger": "stale/out-of-order generation or phase mismatch",
        "vector_error_label": "STALE_OR_OUT_OF_ORDER_GENERATION"
      },
      {
        "error_code": "F04_STATE_ERROR",
        "trigger": "spatial_audio before-generation/digest mismatch or invalid ring shape/cursor"
      },
      {
        "error_code": "F04_INPUT_ERROR",
        "trigger": "programme/attempt/transaction/provider/semantic-host mismatch or row shape/nonfinite input",
        "vector_error_label": "F04_INPUT_ERROR where supplied"
      }
    ],
    "priority": [
      "retired",
      "selected_profile",
      "callback",
      "owner_generation_digest",
      "other_lineage_and_input"
    ]
  },
  "fatal_behavior": "local validation returns the first mapped error with no proposal; any downstream or parent failure discards the staged LFE proposal with the other staged roots and retires the programme",
  "fixture_files": [
    "crates/jocforge-domain/tests/fixtures/direct_lfe/INPUT-ONLY-VECTORS.json",
    "crates/jocforge-domain/tests/fixtures/direct_lfe/G1-VECTOR-ERRATUM.json"
  ],
  "fixture_inventory": [
    {
      "bytes": 13306,
      "path": "crates/jocforge-domain/tests/fixtures/direct_lfe/INPUT-ONLY-VECTORS.json",
      "sha256": "52c2b1a3b52e53198e47e90e4d6e51e8435fbe660ad996b4005f741107a7eb7f"
    },
    {
      "bytes": 639,
      "path": "crates/jocforge-domain/tests/fixtures/direct_lfe/G1-VECTOR-ERRATUM.json",
      "sha256": "b9d8c32b497c2498e3ae1c2a2822fd5cee2dccd456b5dae68bf3fb5ba0f1f307"
    }
  ],
  "functional_scope": [
    {
      "functional_id": "F04C",
      "status": "SCOPED_SUBSET_ONLY",
      "supplement_sections": [
        "1 ingress and lineage",
        "2 three-slot state machine",
        "3 Type10 row0 projection"
      ]
    }
  ],
  "implementation_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/direct_lfe.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state.rs",
    "crates/jocforge-domain/src/spatial/spatial_control.rs"
  ],
  "implementation_inventory": [
    {
      "bytes": 17056,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/direct_lfe.rs",
      "sha256": "b30cb63addb5f8ddfe6bf03e65f84e34221929b8cb78180efc1e6c762c6b6bc5"
    },
    {
      "bytes": 35306,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state.rs",
      "sha256": "794c5980f2c6f29f8aad45768a6a8b14e4fc1992b2715171a5f747afdaf4f05b"
    },
    {
      "bytes": 26347,
      "path": "crates/jocforge-domain/src/spatial/spatial_control.rs",
      "sha256": "3c13f67f84698b32d568bf9fda223878eb3925a95c7fcbe30e0d0ef2f1b8fcd2"
    }
  ],
  "implementation_prohibitions": [
    "no raw38-to-LFE producer",
    "no hardcoded LFE host slot",
    "no regular-source mute transform",
    "no component producer or component-row assembly",
    "no Type10 full assembly",
    "no receiver/QMF or JOC/codec logic",
    "no synthetic EOF flush",
    "no local commit/abort/fatal authority",
    "no parent transaction",
    "no F04 full-chain/product acceptance promotion"
  ],
  "implementation_status": "LOCALLY_VERIFIED_DIRECT_LFE__UPSTREAM_PARENT_DEFERRED",
  "input_contract": {
    "cached_drain": "reuse the exact last NORMAL padded LFE row bytes with a new callback generation; do not reread ADM or regenerate samples",
    "host_slot": "typed host identity in 0..37; never hardcode a source/track slot",
    "lineage": [
      "programme",
      "attempt",
      "CallbackTransactionId",
      "FrontendCallbackIndex",
      "explicit CallbackPhase",
      "selected profile",
      "spatial_control before-generation and full digest",
      "provider lineage",
      "unique semantic LFE host identity"
    ],
    "mute": "no mute input; preserve explicit row bytes unchanged; regular-source mute does not enter this path",
    "normal_short_read": "upstream pads the full host38 frame with positive zero before this view is created; E3 receives the full padded row",
    "payload": "owned finite F32[1536], exact F32LE row bytes of 6144 bytes from host38[g][lfe_host][0..1535]",
    "producer_authority": "future spatial_control/F03W producer after validated ADM graph establishes exactly one semantic_role=lfe and bed_id=3 host slot",
    "type": "opaque DirectLfeInputView"
  },
  "mapping_document": "docs/implementation/DIRECT-LFE-RING-MAP.md",
  "mapping_status": "MAPPING_COMPLETE",
  "not_run": [
    "real ADM read",
    "production raw38-to-current_lfe sample audit",
    "whole callback transaction/fault injection",
    "receiver/QMF",
    "full song/decode/render/listening"
  ],
  "overall_p1_3_e_status_remains": "BLOCKED_ON_F04_DSP_RENDER_AND_PARENT",
  "owner_and_state": {
    "cold": {
      "input_frame_indexes": [
        null,
        null,
        null
      ],
      "model_samples_consumed": 0,
      "read_index": 1,
      "rows": [
        null,
        null,
        null
      ],
      "write_index": 0
    },
    "exact_fields": [
      "write_index:I64LE",
      "read_index:I64LE",
      "rows:T3<None|B(6144)> according to the approved runtime variants",
      "input_frame_indexes:T3<None|I64LE>",
      "model_samples_consumed:I64LE"
    ],
    "functional_owner": "F04C within the single spatial_audio programme root",
    "sample_count_unit": "model F32 samples; not ADM valid samples, bytes, frames, or output samples",
    "staged_result": "linear LfeStageProposal with complete ring after-state and optional immutable Type10LfeRow0Input; no local committed state or success authority",
    "state_pointer": "#/whole_state_by_boundary/cold/fields/5 (lfe_state)",
    "state_schema_bytes": 226344,
    "state_schema_file": "test-vectors/F04-WHOLE-SPATIAL-STATE-SCHEMA.json",
    "state_schema_sha256": "4fdab9f591821d7c50e7daca17bc98d4a5b0616e9cb45b5937cae47b26ed657c"
  },
  "planned_implementation_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/direct_lfe.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state.rs",
    "crates/jocforge-domain/src/spatial/spatial_control.rs"
  ],
  "planned_test_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/direct_lfe/tests.rs"
  ],
  "planned_test_vector_fixtures": [
    "crates/jocforge-domain/tests/fixtures/direct_lfe/INPUT-ONLY-VECTORS.json",
    "crates/jocforge-domain/tests/fixtures/direct_lfe/G1-VECTOR-ERRATUM.json"
  ],
  "profile_binding": "selected EncodingProfile only, validated against the spatial_audio owner; all other profiles remain UNSUPPORTED_PROFILE",
  "programme_root": "spatial_audio",
  "receipt": "no E3-specific committed receipt; a future complete F04C/CallbackTransaction owns selected-j05-owner-receipt.v1 and the global OwnerReceipt<SpatialAudio>",
  "reset_authority": "programme-origin full restart reconstructing the complete cold root; no resume within RETIRED attempt",
  "scope": "Consume one opaque F03W-owned direct-LFE input and stage only the three-slot delay-ring transition plus optional Type10 row0 input. Do not implement the raw38 producer, component rows, receiver/QMF, full F04 assembler, receipt wrapper, local commit, parent transaction, or codec/JOC logic.",
  "stage_authority": "F04C direct-LFE substage only; future F04 composer joins this proposal with the P1.3-E2 component proposal",
  "status": "LOCALLY_VERIFIED_DIRECT_LFE__UPSTREAM_PARENT_DEFERRED",
  "supplement": {
    "api_schema_bytes": 652,
    "api_schema_file": "API-SCHEMA.json",
    "api_schema_sha256": "611bacbcc774b217449387c062af86c87f6fda2c523f5db6d4fa703bc9b3073e",
    "g1_erratum_bytes": 639,
    "g1_erratum_file": "G1-VECTOR-ERRATUM.json",
    "g1_erratum_sha256": "b9d8c32b497c2498e3ae1c2a2822fd5cee2dccd456b5dae68bf3fb5ba0f1f307",
    "identity": "direct-lfe-ring-v1-with-g1-erratum",
    "manifest_sha256": "15da6c0cd0802079c1911e161eacb4cd1e1d4afd3e15a606f3daf1988a5f3595",
    "root": "outputs/IMPLEMENTER_SUPPLEMENTS/DIRECT-LFE-RING-V1",
    "spec_bytes": 6204,
    "spec_file": "SPEC.md",
    "spec_sha256": "44fbbd2bd707ca26488b885ee879c448c759e1227a69a78f3817353c69b2051d",
    "vectors_bytes": 13306,
    "vectors_file": "INPUT-ONLY-VECTORS.json",
    "vectors_sha256": "52c2b1a3b52e53198e47e90e4d6e51e8435fbe660ad996b4005f741107a7eb7f"
  },
  "task_id": "P1.3-E3-F04-DIRECT-LFE-RING",
  "test_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/direct_lfe/tests.rs"
  ],
  "test_inventory": [
    {
      "bytes": 19748,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/direct_lfe/tests.rs",
      "sha256": "974767ddea57685ddf6656a5e46308e47b2a92cefdb7e23827230d6c9f831c66"
    }
  ],
  "transition": {
    "failure": "no after proposal and no ring/cursor/index/sample-count advance; enclosing callback retires the programme and same-attempt retry is forbidden",
    "next_read": "(old_read+1)%3",
    "next_write": "(old_write+1)%3",
    "output_read": "read old.rows[next_read] and old.input_frame_indexes[next_read] before writing",
    "preflight_before_change": [
      "retired state",
      "selected profile",
      "callback generation and explicit phase",
      "programme/attempt/transaction/provider/host lineage",
      "spatial_audio before generation and full digest",
      "ring indices/shape",
      "payload exact shape and finite values"
    ],
    "sample_advance": "model_samples_consumed += 1536 exactly once after all preflight passes",
    "write_current": "write current_lfe[g] and typed callback generation g into next_write"
  },
  "vectors": {
    "actual_first_rule": "construct rows from the listed input recipe or exact prior cached row, compute bytes/digests and execute the ring; never seed state from expected values or hashes",
    "assertions": [
      "before/after three row digests and frame-index tuples",
      "write/read indices",
      "model sample count",
      "priming/output availability",
      "output origin and exact row bytes",
      "Type10 row0 and component15 origins",
      "failure state unchanged"
    ],
    "cached_rows": "g2..g5 reuse exact padded g1 bytes; do not rerun the impulse recipe",
    "case_ids": [
      "g0",
      "g1",
      "g2",
      "g3",
      "g4",
      "g5"
    ],
    "expected_values_are_assertions_only": true,
    "failure_case_ids": [
      "stale_generation",
      "short_shape",
      "nonfinite"
    ],
    "file": "INPUT-ONLY-VECTORS.json",
    "g1_erratum": {
      "default_u32": 0,
      "fresh_f32le_bytes": 6144,
      "fresh_sha256": "755447224cb411e85201061e13b79d0b020b08642933c5acb161abc0026624e7",
      "negative_zero_indexes": [],
      "override_index": 270,
      "override_u32": "0x3E000000",
      "positive_zero_padding_range": [
        731,
        1536
      ],
      "sample_count": 1536,
      "valid_samples": 731
    }
  },
  "verification": {
    "direct_lfe_test_count": 9,
    "f03w_raw38_producer": "NOT_IMPLEMENTED",
    "failure_vector_cases": 3,
    "full_repository_boundary_mutations_rejected": 629,
    "full_type10_assembler": "NOT_IMPLEMENTED",
    "local_commit": "NOT_IMPLEMENTED",
    "parent_transaction": "NOT_IMPLEMENTED",
    "receiver_qmf": "NOT_IMPLEMENTED",
    "success_vector_cases": 6,
    "targeted_implementation_mutations_rejected": 22,
    "targeted_mapping_mutations_rejected": 22
  }
}
```
