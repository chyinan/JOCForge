# Vector 映射

由 IMPLEMENTATION-MAP.json 生成，不得手工修改。

| functional_id | kind | authoritative_vectors | case_pointers | inventory_status |
|---|---|---|---|---|
| STRC | — | — | — | INVENTORIED |
| GTC | — | — | — | INVENTORIED |
| XSTJ | RULE_AND_PROPERTY_TEST | — | `J03` | INVENTORIED |
| F01 | — | — | — | INVENTORIED |
| F01R | — | — | — | INVENTORIED |
| F01G | — | — | — | INVENTORIED |
| F01B | — | — | — | INVENTORIED |
| F01GL | — | — | — | INVENTORIED |
| F01C | — | — | — | INVENTORIED |
| F01E | — | — | — | INVENTORIED |
| F02 | RULE_AND_PROPERTY_TEST | — | `U-TIME-1`<br>`U-TIME-1535`<br>`U-TIME-1536`<br>`U-TIME-1537`<br>`U-TIME-6522906` | INVENTORIED |
| F03 | — | — | — | INVENTORIED |
| F03G | — | — | — | INVENTORIED |
| F03C | — | — | — | INVENTORIED |
| F03U | — | — | — | INVENTORIED |
| F03F | — | — | — | INVENTORIED |
| F03FC | — | — | — | INVENTORIED |
| F03R | — | — | — | INVENTORIED |
| F03P | — | — | — | INVENTORIED |
| F03N | — | — | — | INVENTORIED |
| F03S | — | — | — | INVENTORIED |
| F03FD | — | — | — | INVENTORIED |
| F03W | — | — | — | INVENTORIED |
| F04 | — | — | — | INVENTORIED |
| F04C | — | — | — | INVENTORIED |
| F04A | — | — | — | INVENTORIED |
| F04F | — | — | — | INVENTORIED |
| F04FM | — | — | — | INVENTORIED |
| F04FCM | — | — | — | INVENTORIED |
| F04G | — | — | — | INVENTORIED |
| F04GA | — | — | — | INVENTORIED |
| F04P | — | — | — | INVENTORIED |
| F04PG | — | — | — | INVENTORIED |
| F04R | DIRECT_INPUT_ONLY_RECIPES_AND_POSTEXECUTION_LITERALS_CONTINUOUS_CHAIN_PLUS_MODEL_FIRST_WHOLE_CHAIN | `crates/jocforge-domain/tests/fixtures/receiver_pipeline/F04R-DIRECT-INPUT-VECTORS.json`<br>`crates/jocforge-domain/tests/fixtures/receiver_pipeline/F04R-DIRECT-ACTUAL-LITERALS.json`<br>`test-vectors/unit/vectors.json`<br>`test-vectors/F04-WHOLE-SPATIAL-CHAIN-VERIFICATION.json` | `case1_cold_normal`<br>`case2_warm_normal`<br>`case3_cached_drain`<br>`missing_provider -> F04R_PROVIDER_MISSING`<br>`provider_identity -> F04R_PROVIDER_IDENTITY`<br>`stale_target_updates -> F04R_LINEAGE_TARGET_UPDATES`<br>`invalid_model_state -> F04R_STATE_SHAPE`<br>`invalid_temporal_cursor -> F04R_TEMPORAL_CURSOR`<br>`type10_shape -> F04R_TYPE10_SHAPE`<br>`target_shape -> F04R_TARGET_SHAPE`<br>`nonfinite_input -> F04R_INPUT_NONFINITE`<br>`numeric_nonfinite -> F04R_NUMERIC_NONFINITE`<br>`six F04 whole-chain model-first assertions remain non-authoritative for direct F04R execution` | INVENTORIED |
| F04D | INPUT_ONLY_RECIPES_AND_POSTCOMPUTATION_LITERALS | `outputs/IMPLEMENTER_SUPPLEMENTS/F04-DESCRIPTOR-CONTROL-V1/F04D-DIRECT-INPUT-VECTORS.json`<br>`outputs/IMPLEMENTER_SUPPLEMENTS/F04-DESCRIPTOR-CONTROL-V1/F04D-DIRECT-ACTUAL-LITERALS.json` | `cold_zero`<br>`head_tail_lfe_excluded`<br>`threshold_equal_hold_decrement`<br>`lookup_last_ordinal`<br>`attack_and_nontrivial_ramp`<br>`old_tail_peak_carry_current_tail_zero`<br>`8 direct error cases; see supplement identity-bound file` | INVENTORIED |
| F04Q | — | — | — | INVENTORIED |
| F04RB | DIRECT_MODE0_INPUT_ONLY_AND_POSTCOMPUTATION_LITERALS_PLUS_MODEL_FIRST_WHOLE_BINDING | `outputs/IMPLEMENTER_SUPPLEMENTS/F04RB-MODE0-FITTING-V1/F04RB-MODE0-INPUT-VECTORS.json`<br>`outputs/IMPLEMENTER_SUPPLEMENTS/F04RB-MODE0-FITTING-V1/F04RB-MODE0-ACTUAL-LITERALS.json`<br>`test-vectors/QMF-RECEIVER-WHOLE-BINDING-VERIFICATION.json`<br>`test-vectors/F04-WHOLE-SPATIAL-CHAIN-VERIFICATION.json` | `cold_normal_g2`<br>`warm_normal_g3`<br>`cached_drain_g4`<br>`provider_missing_precedes_stale`<br>`provider_identity_invalid`<br>`stale_lineage`<br>`descriptor_projection_mismatch`<br>`state_delay_shape`<br>`s09_shape`<br>`target_shape`<br>`nonfinite_input`<br>`finite_numeric_overflow`<br>`QRB-CONT-0..4 are whole-binding model-first assertions, not isolated implementation results` | INVENTORIED |
| F04QF | — | — | — | INVENTORIED |
| F04QP | — | — | — | INVENTORIED |
| F05 | — | — | — | INVENTORIED |
| F05Q | — | — | — | INVENTORIED |
| F05QH | — | — | — | INVENTORIED |
| F05J | — | — | — | INVENTORIED |
| F05JC | — | — | — | INVENTORIED |
| F05JL | — | — | — | INVENTORIED |
| F05JS | — | — | — | INVENTORIED |
| F05O | — | — | — | INVENTORIED |
| F05T | — | — | — | INVENTORIED |
| F05H | — | — | — | INVENTORIED |
| F05L | — | — | — | INVENTORIED |
| F05R56 | INPUT_ONLY_RECIPES_WITH_POSTCOMPUTATION_ASSERTIONS | `test-vectors/READER56-TARGET-UPDATES-VERIFICATION.json`<br>`test-vectors/READER56-TARGET-STATE-SCHEMA.json`<br>`outputs/IMPLEMENTER_SUPPLEMENTS/READER-TARGET-UPDATES-V1/READER-TARGET-RECEIPT-INPUTS.json`<br>`outputs/IMPLEMENTER_SUPPLEMENTS/READER-TARGET-UPDATES-V1/READER-TARGET-RECEIPT-VECTORS.json`<br>`outputs/IMPLEMENTER_SUPPLEMENTS/PANNER-TRIGONOMETRY-V1/KAT.json` | `R56T-CONT-0..7: continuous marker/history callbacks; g0/g1 have no reader receipt; g2..g7 have one receipt each`<br>`reader_materializer_cases: previous-state-aware 16x56 records; expected records are postcompute comparisons only`<br>`target_leaf_cases: all 16x6 F32 words; expected target words are postcompute comparisons only`<br>`receipt vectors: schema, lineage, hashes, finite target payload and semantic digest postcompute checks`<br>`rejection_cases: BAD_CALLBACK, BAD_READER_GATE, OLD_ROW_COUNT, NEW_ROW_BYTES, SNAPSHOT_KEYS, RETIRED, COMPONENT6_PROFILE`<br>`extra guards: state invocation mismatch and nonfinite numeric intermediate`<br>`panner_trigonometry_kat: exact-rational direct-angle and equal-power words plus enclosure step counts; no host libm`<br>`fixture_integrity: fresh snapshot, matrix, overlap and 15x3072 mixer inputs are hashed after generation; marker energy consumes only the first 1536 mixer samples; hashes never seed the producer` | INVENTORIED |
| F06 | — | — | — | INVENTORIED |
| F06L | — | — | — | INVENTORIED |
| F06I | — | — | — | INVENTORIED |
| F06W | — | — | — | INVENTORIED |
| F06KG | — | — | — | INVENTORIED |
| F06TP | — | — | — | INVENTORIED |
| F06ND | — | — | — | INVENTORIED |
| F06NR | — | — | — | INVENTORIED |
| F06NI | — | — | — | INVENTORIED |
| F06NF | — | — | — | INVENTORIED |
| F06WH | — | — | — | INVENTORIED |
| F06U | — | — | — | INVENTORIED |
| F06DG | — | — | — | INVENTORIED |
| F06D | — | — | — | INVENTORIED |
| F06M | — | — | — | INVENTORIED |
| F06OG | — | — | — | INVENTORIED |
| F06OM | — | — | — | INVENTORIED |
| F06OP | — | — | — | INVENTORIED |
| F06OE | — | — | — | INVENTORIED |
| F06QT | — | — | — | INVENTORIED |
| F06QM | — | — | — | INVENTORIED |
| F06QS | — | — | — | INVENTORIED |
| F06OD | — | — | — | INVENTORIED |
| F06OS | — | — | — | INVENTORIED |
| F06OB | — | — | — | INVENTORIED |
| F06OR | — | — | — | INVENTORIED |
| F06OC | — | — | — | INVENTORIED |
| F06OF | — | — | — | INVENTORIED |
| F07 | — | — | — | INVENTORIED |
| F07A | — | — | — | INVENTORIED |
| F08 | — | — | — | INVENTORIED |
| F08NZ | — | — | — | INVENTORIED |
| F09 | — | — | — | INVENTORIED |
| F09E | — | — | — | INVENTORIED |
| F09PA | — | — | — | INVENTORIED |
| F09G | — | — | — | INVENTORIED |
| F09LB | — | — | — | INVENTORIED |
| F10 | — | — | — | INVENTORIED |
| F11 | — | — | — | INVENTORIED |
| F12 | — | — | — | INVENTORIED |
| F12S | — | — | — | INVENTORIED |
| F12SC | — | — | — | INVENTORIED |
| F12A | — | — | — | INVENTORIED |
| F13 | — | — | — | INVENTORIED |
| F13F | — | — | — | INVENTORIED |
| F13T | — | — | — | INVENTORIED |
| F13C | — | — | — | INVENTORIED |
| F13D2 | — | — | — | INVENTORIED |
| F14 | — | — | — | INVENTORIED |
| F14K | — | — | — | INVENTORIED |
| F15 | — | — | — | INVENTORIED |
| F15P | — | — | — | INVENTORIED |
| OWNJ | — | — | — | INVENTORIED |

## Direct sub-slice vectors

由 IMPLEMENTATION-MAP.json 生成，不得手工修改。

| slice_id | kind | authoritative_vectors | case_pointers | local_fixture |
|---|---|---|---|---|
| F05O/oamd-rowdata-body-encoder | INPUT_ONLY_UNIT_VECTORS_PLUS_SPEC_DERIVED_TRANSITION_CASES | `test-vectors/unit/vectors.json` | `U-OAMD-XYZ`<br>`U-OAMD-ESCAPED`<br>`U-OAMD-ROWS`<br>`U-OAMD-COUNT-31`<br>`U-OAMD-COUNT-32`<br>`U-OAMD-COUNT-159`<br>`U-OAMD-TIME-TABLE`<br>`U-OAMD-TIME-EXPLICIT`<br>`XYZ transition neighbors and halfway ties derived from the published integer formula` | `crates/jocforge-domain/tests/fixtures/oamd_wire/unit-vectors.json`<br>8428fd8a52cb018c00fff87c7324d4dbcc47e5e1720c76b79476d45e6bd21129 |
