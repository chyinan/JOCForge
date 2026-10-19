# Reader target updates map

由 IMPLEMENTATION-MAP.json 生成，不得手工修改。

Mapping status: `MAPPING_COMPLETE`. Implementation status: `MAPPING_COMPLETE_IMPLEMENTATION_READY`.

F05R56 previews marker/history, the selected Type11 encode/decode boundary, 16 reader56 records, and the 16x6 target payload. The persistent state is exactly the nine fields in `reader56-target-state-schema.v2`; targets and receipts are callback-local and contribute zero canonical state bytes.

The callback clock is `FrontendCallbackIndex g`; callbacks 0 and 1 advance marker/history and target snapshot only. For `g>=2`, the separate `ReaderInvocationIndex q` is `g-2`; delivered row generations remain attached per row. The stage returns a proposal and a pending immutable `ReaderTargetUpdateReceipt`; only the enclosing `spatial_control` transaction may commit state and admit that receipt.

Receipt consumers may use only the admitted same-generation receipt. This mapping does not authorize F05JC/J05, F05L queue/body production, parent transaction implementation, or whole-programme/product acceptance.

## Machine-readable contract

```json
{
  "clock": {
    "covered_clauses": [
      "g is zero-based FrontendCallbackIndex; g=0,1 advance marker/history and target snapshot only",
      "reader begins only for g>=2; q is ReaderInvocationIndex(g-2)",
      "history producer generation is carried independently per delivered regular row; cold generation is -1",
      "direct history row provenance is current g; regular rows preserve delivered producer generation",
      "one successful g>=2 callback proposes one receipt with reader invocation q and all 16 row generations",
      "callback g is advanced only by the enclosing spatial_control transaction; this stage has no commit"
    ],
    "domains": [
      "FrontendCallbackIndex",
      "ReaderInvocationIndex",
      "HistoryProducerGeneration"
    ]
  },
  "error_outcomes": [
    {
      "caller_action": "Return STAGE_FATAL with no receipt, payload, proposal, or commit; preserve committed bytes; require full cold reset.",
      "category": "state schema or lineage",
      "error_code": "R56T_STATE_ERROR",
      "programme_state_after": "RETIRED",
      "trigger": "persistent state shape, canonical identity, or before-state lineage is invalid"
    },
    {
      "caller_action": "Return STAGE_FATAL with no receipt, payload, proposal, or commit; require full cold reset.",
      "category": "programme lifecycle",
      "error_code": "R56T_PIPELINE_RETIRED",
      "programme_state_after": "RETIRED",
      "trigger": "stage is invoked after the attempt has retired"
    },
    {
      "caller_action": "Return STAGE_FATAL with no receipt, payload, proposal, or commit; preserve committed bytes and generation.",
      "category": "clock and generation",
      "error_code": "R56T_CALLBACK_ORDER",
      "programme_state_after": "RETIRED",
      "trigger": "callback g is stale, skipped, or inconsistent with the committed state generation"
    },
    {
      "caller_action": "Return STAGE_FATAL with no receipt, payload, proposal, or commit; preserve committed bytes and generation.",
      "category": "input shape or value",
      "error_code": "R56T_INPUT_ERROR",
      "programme_state_after": "RETIRED",
      "trigger": "snapshot, matrix, source flags, overlap, mixer, or reader rows violate the selected input contract"
    },
    {
      "caller_action": "Reject before state replacement; return no receipt, target payload, proposal, or commit.",
      "category": "selected-profile preflight",
      "error_code": "R56T_PROFILE_UNSUPPORTED",
      "programme_state_after": "RETIRED",
      "trigger": "Type11 branch, reader row, component, coordinate, trim, or target panner value is outside the selected profile"
    },
    {
      "caller_action": "Return STAGE_FATAL with no receipt, payload, proposal, or commit; preserve committed bytes and generation.",
      "category": "numeric intermediate",
      "error_code": "R56T_NUMERIC_FAILURE",
      "programme_state_after": "RETIRED",
      "trigger": "required finite R32 or mathematical-function result becomes nonfinite or cannot be represented"
    }
  ],
  "functional_id": "F05R56",
  "implementation_files": [
    "crates/jocforge-domain/src/joc/oamd/reader_target.rs"
  ],
  "implementation_status": "MAPPING_COMPLETE_IMPLEMENTATION_READY",
  "inventory_status": "INVENTORIED",
  "mapping_status": "MAPPING_COMPLETE",
  "owner": "spatial_control",
  "parameters": {
    "kind": "IDENTITY_VALIDATED_SELECTED_PROVIDERS",
    "provider_files": [
      "parameters/reader56-target-profile-v1.json",
      "parameters/reader56-main-panner-v1.json",
      "parameters/oamd-id11-live-v1.json",
      "parameters/oamd-coordinate-grid-q15.json",
      "parameters/meter-oar-secondary8-math-v1.json"
    ]
  },
  "phase_scope": {
    "commit_authority": "spatial_control parent transaction only; F05R56 exposes a proposal and has no local commit",
    "consumer_boundary": "F04C/F04R/F05JC may use only the exact same-generation receipt after parent admission; no producer rerun or duplicate target state",
    "exclusions": [
      "No F05JC or J05 execution",
      "No parent CallbackTransaction or spatial_control commit implementation",
      "No F05L ID11 body/queue owner implementation",
      "No real ADM cold replay, full-programme producer, whole receiver, native rollback, codec, writer, listening, W04, or product acceptance"
    ],
    "persistent_state_schema": "reader56-target-state-schema.v2; exactly nine fields; target payload and receipt are not persistent state",
    "receipt_payload": "FiniteF32[16][6], row order [L,R,C,LFE,Ls,Rs], 384-byte F32LE payload, semantic hash over canonical sorted compact JSON including payload hex",
    "receipt_schema": "reader-target-update-receipt.v1",
    "receipt_type": "ReaderTargetUpdateReceipt",
    "scope": "F05R56 selected ReaderTargetState transition and callback-local ReaderTargetUpdateReceipt production only",
    "stage_authority": "F05R56 previews marker/history, Type11 wire roundtrip, reader56 and target rows"
  },
  "planned_phase": "Phase 4",
  "spec_files": [
    "specs/05-reader56-target-updates-composition.md",
    "specs/05-fit-marker-and-metadata-history.md",
    "specs/05-oamd-id11-live-producer.md",
    "specs/05-oamd-row-and-timing-wire.md"
  ],
  "test_modules": [
    "crates/jocforge-domain/src/joc/oamd/reader_target/tests.rs"
  ],
  "vectors": {
    "authoritative_vectors": [
      "test-vectors/READER56-TARGET-UPDATES-VERIFICATION.json",
      "test-vectors/READER56-TARGET-STATE-SCHEMA.json",
      "outputs/IMPLEMENTER_SUPPLEMENTS/READER-TARGET-UPDATES-V1/READER-TARGET-RECEIPT-INPUTS.json",
      "outputs/IMPLEMENTER_SUPPLEMENTS/READER-TARGET-UPDATES-V1/READER-TARGET-RECEIPT-VECTORS.json"
    ],
    "case_pointers": [
      "R56T-CONT-0..7: continuous marker/history callbacks; g0/g1 have no reader receipt; g2..g7 have one receipt each",
      "reader_materializer_cases: previous-state-aware 16x56 records; expected records are postcompute comparisons only",
      "target_leaf_cases: all 16x6 F32 words; expected target words are postcompute comparisons only",
      "receipt vectors: schema, lineage, hashes, finite target payload and semantic digest postcompute checks",
      "rejection_cases and receipt rejection cases: first-failure and zero-output/zero-state-change assertions"
    ],
    "kind": "INPUT_ONLY_RECIPES_WITH_POSTCOMPUTATION_ASSERTIONS"
  }
}
```
