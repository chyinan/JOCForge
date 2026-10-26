// pattern: Imperative Shell
use super::{CurrentFeaturesError, CurrentFeaturesPacket, produce_current_features};
use crate::{
    frontend::{
        PackageManifestEvidence, ProviderFileEvidence, ValidatedF01EColdProvider,
        ValidatedF03TargetColdProvider, ValidatedFeatureFft4096MathProvider,
        ValidatedFeatureProcessorMathProvider, validate_current_features_package_manifest,
        validate_f01e_package_manifest, validate_f01e_provider, validate_f03_package_manifest,
        validate_f03_target_cold_provider, validate_feature_fft4096_math_provider,
        validate_feature_processor_math_provider,
    },
    profile::{EmdfCarriageProfile, EncodingProfile},
    spatial::{
        SourceFlagsPacket, SpatialControlJoinMetadata, SpatialControlMachine,
        frontend_stages::gate_current_features,
        post_window_fanout::{
            CallbackPhase, CurrentFeaturesAudioView, PAYLOAD_BYTE_COUNT, PostWindowFanoutError,
            tests::{current_features_audio_view_fixture, render_proposal_fixture},
        },
        source_flags::{
            SourceFlagsAttempt, SourceFlagsParentLineage, current_features_test_packet,
        },
    },
    timeline::FrontendCallbackIndex,
    transaction::{AttemptId, CallbackLineageIdentity, ProgrammeId},
};
use serde_json::Value;
use std::sync::Arc;

const VECTORS: &[u8] =
    include_bytes!("../../../tests/fixtures/current_features/INPUT-ONLY-LITERAL-VECTORS.json");
const MANIFEST: &[u8] = include_bytes!("../../../tests/fixtures/f01e_provider/MANIFEST.json");
const F01E_PROVIDER: &[u8] =
    include_bytes!("../../../tests/fixtures/f01e_provider/f01e-selected-constructor-v1.json");
const F03_PROVIDER: &[u8] =
    include_bytes!("../../../tests/fixtures/target_cold/f03-selected-target-cold-v1.json");
const FEATURE_PROCESSOR: &[u8] =
    include_bytes!("../../../tests/fixtures/current_features/feature-processor-math-v1.json");
const FEATURE_FFT: &[u8] =
    include_bytes!("../../../tests/fixtures/current_features/feature-fft4096-math-v1.json");
const ATTEMPT_ID: AttemptId = AttemptId::new(0);

fn programme_id() -> ProgrammeId {
    crate::transaction::test_programme_id("CURRENT_FEATURES_TEST")
}

fn transaction_id(callback_index: FrontendCallbackIndex) -> CallbackLineageIdentity {
    transaction_id_for(programme_id(), ATTEMPT_ID, callback_index)
}

fn transaction_id_for(
    programme_id: ProgrammeId,
    attempt_id: AttemptId,
    callback_index: FrontendCallbackIndex,
) -> CallbackLineageIdentity {
    CallbackLineageIdentity::new(programme_id, attempt_id, callback_index)
}

fn frontend_callback_index(value: u64) -> FrontendCallbackIndex {
    FrontendCallbackIndex::new(u32::try_from(value).expect("fixture callback index fits U32"))
}

const POSITIVE_CASES: [&str; 15] = [
    "cold_suppress",
    "warm_reset_11",
    "consumer_reset_priority_01",
    "carry_recompute_scalar",
    "read_zero_threshold_equality",
    "read_impulse_no_extra_window",
    "read_dyadic_comb",
    "two_sources_packet_order",
    "read_repeat_blend_continuation",
    "read_12x_amplitude_jump",
    "read_zero_with_warm_history",
    "threshold_below",
    "threshold_first_nonjump",
    "threshold_above",
    "reset_then_read_replay",
];

const NEGATIVE_CASES: [&str; 9] = [
    "read_missing_audio",
    "read_wrong_audio_extent",
    "source_count_47",
    "wrong_flag_value",
    "before_vector_23",
    "NaN_before",
    "NaN_read_audio",
    "stale_callback",
    "finite_audio_intermediate_overflow",
];
const NEGATIVE_ERROR_LABELS: [&str; 9] = [
    "READ_AUDIO_MISSING",
    "RENDER_PAYLOAD_SHAPE",
    "RENDER_PAYLOAD_SHAPE",
    "FLAG_DOMAIN",
    "BEFORE_VECTOR_SHAPE",
    "NONFINITE_BEFORE",
    "RENDER_NONFINITE_PAYLOAD",
    "STALE_CALLBACK",
    "NONFINITE_INTERMEDIATE",
];

struct TestContext {
    machine: SpatialControlMachine,
    processor: ValidatedFeatureProcessorMathProvider,
    fft: ValidatedFeatureFft4096MathProvider,
}

struct PreparedCase {
    context: TestContext,
    source_flags: SourceFlagsPacket,
    post_window_audio: Option<CurrentFeaturesAudioView>,
    render_proposal_error: Option<PostWindowFanoutError>,
    callback_index: FrontendCallbackIndex,
    transaction_id: CallbackLineageIdentity,
}

fn vectors() -> Value {
    serde_json::from_slice(VECTORS).expect("fixture JSON identity is checked by the boundary gate")
}

fn providers() -> (
    ValidatedF01EColdProvider,
    ValidatedF03TargetColdProvider,
    ValidatedFeatureProcessorMathProvider,
    ValidatedFeatureFft4096MathProvider,
) {
    let manifest = PackageManifestEvidence::new(Arc::from(MANIFEST), None);
    let f01e_manifest = validate_f01e_package_manifest(&manifest).unwrap();
    let f01e = validate_f01e_provider(
        &ProviderFileEvidence::new(Arc::from(F01E_PROVIDER), None),
        &f01e_manifest,
    )
    .unwrap();
    let f03_manifest = validate_f03_package_manifest(&manifest).unwrap();
    let f03 = validate_f03_target_cold_provider(
        &ProviderFileEvidence::new(Arc::from(F03_PROVIDER), None),
        &f03_manifest,
    )
    .unwrap();

    let current_features_manifest = validate_current_features_package_manifest(&manifest).unwrap();
    let processor = validate_feature_processor_math_provider(
        &ProviderFileEvidence::new(Arc::from(FEATURE_PROCESSOR), None),
        &current_features_manifest,
    )
    .unwrap();
    let fft = validate_feature_fft4096_math_provider(
        &ProviderFileEvidence::new(Arc::from(FEATURE_FFT), None),
        &current_features_manifest,
    )
    .unwrap();
    (f01e, f03, processor, fft)
}

fn context() -> TestContext {
    let (f01e, f03, processor, fft) = providers();
    let machine =
        SpatialControlMachine::new_cold(EncodingProfile::selected(), &f01e, &f03).unwrap();
    TestContext {
        machine,
        processor,
        fft,
    }
}

fn bits(value: &Value) -> u32 {
    u32::try_from(value.as_u64().unwrap()).unwrap()
}

fn prepare_case(case: &Value, invalid_before: Option<&str>) -> PreparedCase {
    let mut context = context();
    let input = &case["input"];
    let callback_value = input
        .get("callback_index")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let callback_index = frontend_callback_index(
        input
            .get("expected_callback_index")
            .and_then(Value::as_u64)
            .unwrap_or(callback_value),
    );

    let mut before_vector = vec![0_u32; 48 * 24];
    let mut before_scalar = vec![0_u32; 48];
    if let Some(before) = input.get("before").and_then(Value::as_object) {
        for (source, state) in before {
            let source = source.parse::<usize>().unwrap();
            if let Some(vector) = state.get("vector_u32").and_then(Value::as_array) {
                let start = source * 24;
                before_vector.splice(start..start + vector.len(), vector.iter().map(bits));
            }
            if let Some(scalar) = state.get("scalar_u32") {
                before_scalar[source] = bits(scalar);
            }
        }
    }
    if invalid_before == Some("before_vector_23") {
        before_vector.truncate(23);
    } else if invalid_before == Some("NaN_before") {
        before_vector[0] = 0x7fc0_0000;
    }
    context
        .machine
        .set_current_features_test_history(&before_vector, &before_scalar);

    let mut flags = [[1_u8, 1_u8]; 48];
    if let Some(overrides) = input.get("flags").and_then(Value::as_object) {
        for (source, pair) in overrides {
            let source = source.parse::<usize>().unwrap();
            let pair = pair.as_array().unwrap();
            flags[source] = [
                pair[0].as_u64().unwrap() as u8,
                pair[1].as_u64().unwrap() as u8,
            ];
        }
    }
    let packet_callback_index = frontend_callback_index(callback_value);
    let transaction_id = transaction_id(packet_callback_index);
    let source_flags_parent =
        SourceFlagsParentLineage::new(transaction_id.clone(), &context.machine).unwrap();
    let source_flags = current_features_test_packet(flags, source_flags_parent);

    let audio_input = input.get("audio").and_then(Value::as_object);
    let requested_source_count = input
        .get("source_count")
        .and_then(Value::as_u64)
        .map(|count| usize::try_from(count).unwrap());
    let has_audio_recipe = audio_input.is_some_and(|audio| !audio.is_empty());
    let is_missing_audio_vector = case["case_id"].as_str() == Some("read_missing_audio");
    let (post_window_audio, render_proposal_error) =
        if !is_missing_audio_vector || has_audio_recipe || requested_source_count.is_some() {
            let mut rows = vec![vec![0_u32; 3072]; requested_source_count.unwrap_or(48)];
            if let Some(audio) = audio_input.filter(|audio| !audio.is_empty()) {
                for (source, recipe) in audio {
                    let source = source.parse::<usize>().unwrap();
                    if let Some(samples) = recipe.get("samples").and_then(Value::as_u64) {
                        rows[source].resize(usize::try_from(samples).unwrap(), 0);
                    }
                    if recipe.get("kind").and_then(Value::as_str) == Some("DYADIC_COMB17") {
                        for (index, word) in rows[source].iter_mut().enumerate() {
                            let centered = i32::try_from(index % 17).unwrap() - 8;
                            *word = ((centered as f32) / 4096.0).to_bits();
                        }
                    }
                    if let Some(points) = recipe.get("points_u32").and_then(Value::as_array) {
                        for point in points {
                            rows[source][usize::try_from(point[0].as_u64().unwrap()).unwrap()] =
                                bits(&point[1]);
                        }
                    }
                }
            }
            let spatial_control_join = Arc::new(SpatialControlJoinMetadata::fixture(
                &context.machine,
                transaction_id.clone(),
                packet_callback_index,
                context.machine.generation(),
                None,
                context.machine.profile(),
            ));
            let payload = rows.into_iter().flatten().map(f32::from_bits).collect();
            match current_features_audio_view_fixture(
                programme_id(),
                ATTEMPT_ID,
                spatial_control_join,
                CallbackPhase::Normal,
                payload,
            ) {
                Ok(view) => (Some(view), None),
                Err(error) => (None, Some(error)),
            }
        } else {
            (None, None)
        };

    PreparedCase {
        context,
        source_flags,
        post_window_audio,
        render_proposal_error,
        callback_index,
        transaction_id,
    }
}

fn produce(
    case: &Value,
    invalid_before: Option<&str>,
) -> Result<super::CurrentFeaturesProduced, CurrentFeaturesError> {
    let prepared = prepare_case(case, invalid_before);
    if let Some(error) = prepared.render_proposal_error {
        panic!("positive CurrentFeatures vector rejected by render boundary: {error:?}");
    }
    produce_current_features(
        &prepared.context.machine,
        &programme_id(),
        ATTEMPT_ID,
        &prepared.transaction_id,
        prepared.callback_index,
        CallbackPhase::Normal,
        prepared.source_flags,
        prepared.post_window_audio,
        &prepared.context.processor,
        &prepared.context.fft,
    )
}

fn flags_packet(
    machine: &SpatialControlMachine,
    transaction_id: &CallbackLineageIdentity,
    _callback_index: FrontendCallbackIndex,
    flags: [[u8; 2]; 48],
) -> SourceFlagsPacket {
    current_features_test_packet(
        flags,
        SourceFlagsParentLineage::new(transaction_id.clone(), machine).unwrap(),
    )
}

fn decode_hex(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16).unwrap();
            let low = (pair[1] as char).to_digit(16).unwrap();
            ((high << 4) | low) as u8
        })
        .collect()
}

fn words_as_u32le(words: impl IntoIterator<Item = u32>) -> Vec<u8> {
    words.into_iter().flat_map(u32::to_le_bytes).collect()
}

fn error_label(error: CurrentFeaturesError) -> &'static str {
    match error {
        CurrentFeaturesError::ReadAudioMissing => "READ_AUDIO_MISSING",
        CurrentFeaturesError::AudioShape => "READ_AUDIO_SHAPE",
        CurrentFeaturesError::SourceShape => "SOURCE_SHAPE",
        CurrentFeaturesError::FlagDomain => "FLAG_DOMAIN",
        CurrentFeaturesError::BeforeVectorShape => "BEFORE_VECTOR_SHAPE",
        CurrentFeaturesError::BeforeScalarShape => "BEFORE_SCALAR_SHAPE",
        CurrentFeaturesError::NonFiniteBefore => "NONFINITE_BEFORE",
        CurrentFeaturesError::StaleCallback => "STALE_CALLBACK",
        CurrentFeaturesError::NonFiniteIntermediate => "NONFINITE_INTERMEDIATE",
        CurrentFeaturesError::UnsupportedProfile => "PROFILE_BINDING",
        CurrentFeaturesError::ProviderMismatch => "PROVIDER_BINDING",
        CurrentFeaturesError::ParentLineageMismatch => "LINEAGE_BINDING",
        CurrentFeaturesError::RootIdentityMismatch => "ROOT_BINDING",
        CurrentFeaturesError::RenderBoundaryMismatch => "RENDER_BOUNDARY",
        CurrentFeaturesError::RenderPhaseMismatch => "RENDER_PHASE",
    }
}

fn fanout_error_label(error: PostWindowFanoutError) -> &'static str {
    match error {
        PostWindowFanoutError::PayloadShape => "RENDER_PAYLOAD_SHAPE",
        PostWindowFanoutError::NonFinitePayload => "RENDER_NONFINITE_PAYLOAD",
        PostWindowFanoutError::CallbackIdentityMismatch => "RENDER_CALLBACK_IDENTITY",
    }
}

#[test]
fn all_fifteen_current_features_positive_vectors_match_actual_output_first() {
    let vectors = vectors();
    assert_eq!(vectors["positive_cases"].as_array().unwrap().len(), 15);
    let mut verified_words = 0;
    for case_id in POSITIVE_CASES {
        let case = vectors["positive_cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["case_id"] == case_id)
            .unwrap();
        let actual = produce(case, None).unwrap();
        let expected = &case["expected"];
        let actual_vector = words_as_u32le(
            actual
                .feature_state_proposal
                .source_vector_bits
                .iter()
                .copied(),
        );
        let actual_scalar = words_as_u32le(
            actual
                .feature_state_proposal
                .source_scalar_bits
                .iter()
                .copied(),
        );
        let actual_packet =
            words_as_u32le(actual.packet.test_payload_bits().iter().flatten().copied());
        for (actual_bytes, expected_name) in [
            (actual_vector, "after_source_vector_u32le"),
            (actual_scalar, "after_source_scalar_u32le"),
            (actual_packet, "current_features_band_major_u32le"),
        ] {
            let expected_bytes = decode_hex(expected[expected_name]["hex"].as_str().unwrap());
            assert_eq!(actual_bytes, expected_bytes, "{case_id}/{expected_name}");
            verified_words += actual_bytes.len() / 4;
        }
        assert_eq!(
            actual.feature_state_proposal.source_vector_bits.len(),
            48 * 24
        );
        assert_eq!(actual.feature_state_proposal.source_scalar_bits.len(), 48);
        assert_eq!(
            actual.packet.test_callback_index(),
            FrontendCallbackIndex::new(0)
        );
    }
    assert_eq!(verified_words, 35_280);
}

#[test]
fn current_features_preserves_shared_render_and_after_proposal_identities() {
    let case = vectors()["positive_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["case_id"] == "read_impulse_no_extra_window")
        .unwrap()
        .clone();
    let prepared = prepare_case(&case, None);
    let view = prepared.post_window_audio.as_ref().unwrap();
    let expected_render_stage_identity = Arc::clone(view.render_stage_identity());
    let expected_after_proposal_identity = Arc::clone(
        view.spatial_control_after_proposal_identity()
            .expect("fixture carries a complete after-proposal identity"),
    );
    let expected_payload_sha256 = view.payload_sha256();

    let produced = produce_current_features(
        &prepared.context.machine,
        &programme_id(),
        ATTEMPT_ID,
        &transaction_id(prepared.callback_index),
        prepared.callback_index,
        CallbackPhase::Normal,
        prepared.source_flags,
        prepared.post_window_audio,
        &prepared.context.processor,
        &prepared.context.fft,
    )
    .unwrap();

    assert_eq!(produced.lineage.callback_phase, CallbackPhase::Normal);
    assert_eq!(
        produced.lineage.audio_payload_sha256,
        Some(expected_payload_sha256)
    );
    assert_eq!(
        produced.lineage.audio_payload_byte_count,
        Some(PAYLOAD_BYTE_COUNT)
    );
    assert!(Arc::ptr_eq(
        produced.lineage.render_stage_identity.as_ref().unwrap(),
        &expected_render_stage_identity
    ));
    assert!(Arc::ptr_eq(
        produced
            .lineage
            .spatial_control_after_proposal_identity
            .as_ref()
            .unwrap(),
        &expected_after_proposal_identity
    ));
}

#[test]
fn all_nine_current_features_error_vectors_reject_without_partial_output() {
    let vectors = vectors();
    assert_eq!(vectors["negative_cases"].as_array().unwrap().len(), 9);
    for (case_id, expected_label) in NEGATIVE_CASES.into_iter().zip(NEGATIVE_ERROR_LABELS) {
        let case = vectors["negative_cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["case_id"] == case_id)
            .unwrap();
        let prepared = prepare_case(case, Some(case_id));
        let before_generation = prepared.context.machine.generation();
        let before_digest = prepared.context.machine.digest();
        if let Some(render_error) = prepared.render_proposal_error {
            let expected_render_error = match case_id {
                "read_wrong_audio_extent" | "source_count_47" => {
                    PostWindowFanoutError::PayloadShape
                }
                "NaN_read_audio" => PostWindowFanoutError::NonFinitePayload,
                _ => panic!("unexpected render proposal error in {case_id}"),
            };
            assert_eq!(render_error, expected_render_error, "{case_id}");
            assert_eq!(
                fanout_error_label(render_error),
                expected_label,
                "{case_id}"
            );
            assert_eq!(prepared.context.machine.generation(), before_generation);
            assert_eq!(prepared.context.machine.digest(), before_digest);
            assert!(case["expected"]["packet"].is_null(), "{case_id}");
            assert_eq!(case["expected"]["state_commit_count"].as_u64(), Some(0));
            continue;
        }
        let expected_error = match case_id {
            "read_missing_audio" => CurrentFeaturesError::ReadAudioMissing,
            "wrong_flag_value" => CurrentFeaturesError::FlagDomain,
            "before_vector_23" => CurrentFeaturesError::BeforeVectorShape,
            "NaN_before" => CurrentFeaturesError::NonFiniteBefore,
            "stale_callback" => CurrentFeaturesError::StaleCallback,
            "finite_audio_intermediate_overflow" => CurrentFeaturesError::NonFiniteIntermediate,
            _ => panic!("{case_id} should fail at the render proposal boundary"),
        };
        let result = produce_current_features(
            &prepared.context.machine,
            &programme_id(),
            ATTEMPT_ID,
            &transaction_id(prepared.callback_index),
            prepared.callback_index,
            CallbackPhase::Normal,
            prepared.source_flags,
            prepared.post_window_audio,
            &prepared.context.processor,
            &prepared.context.fft,
        );
        let Err(error) = result else {
            panic!("{case_id} unexpectedly produced a capability");
        };
        assert_eq!(prepared.context.machine.generation(), before_generation);
        assert_eq!(prepared.context.machine.digest(), before_digest);
        let actual_error = error;
        assert_eq!(actual_error, expected_error, "{case_id}");
        assert_eq!(error_label(actual_error), expected_label, "{case_id}");
        assert!(case["expected"]["packet"].is_null(), "{case_id}");
        assert_eq!(case["expected"]["state_commit_count"].as_u64(), Some(0));
    }
}

#[test]
fn current_features_rejects_parent_render_root_and_flags_lineage_mismatches() {
    let callback = FrontendCallbackIndex::new(0);
    let mismatches = [
        ("programme", CurrentFeaturesError::ParentLineageMismatch),
        ("attempt", CurrentFeaturesError::ParentLineageMismatch),
        ("transaction", CurrentFeaturesError::ParentLineageMismatch),
        ("callback", CurrentFeaturesError::StaleCallback),
        ("profile", CurrentFeaturesError::UnsupportedProfile),
        (
            "root_generation",
            CurrentFeaturesError::RootIdentityMismatch,
        ),
        ("root_digest", CurrentFeaturesError::RootIdentityMismatch),
        (
            "flags_transaction",
            CurrentFeaturesError::ParentLineageMismatch,
        ),
    ];
    for (mismatch, expected_error) in mismatches {
        let current_owner = context();
        let mut packet_programme = programme_id();
        let mut packet_attempt = ATTEMPT_ID;
        let mut packet_transaction = transaction_id(callback);
        let mut packet_callback = callback;
        let mut packet_profile = current_owner.machine.profile();
        let mut packet_generation = current_owner.machine.generation();
        let mut packet_before_digest = None;
        let mut flags_transaction = transaction_id(callback);
        match mismatch {
            "programme" => {
                packet_programme = crate::transaction::test_programme_id("OTHERPROGRAMME01");
            }
            "attempt" => packet_attempt = AttemptId::new(1),
            "transaction" => {
                packet_transaction = transaction_id_for(
                    crate::transaction::test_programme_id("OTHERPROGRAMME01"),
                    ATTEMPT_ID,
                    callback,
                );
            }
            "callback" => packet_callback = FrontendCallbackIndex::new(1),
            "profile" => {
                packet_profile =
                    EncodingProfile::selected_with_carriage(EmdfCarriageProfile::EtsiEmdf);
            }
            "root_generation" => packet_generation += 1,
            "root_digest" => packet_before_digest = Some([0x55; 32]),
            "flags_transaction" => {
                flags_transaction = transaction_id_for(
                    crate::transaction::test_programme_id("OTHERPROGRAMME01"),
                    ATTEMPT_ID,
                    callback,
                );
            }
            _ => unreachable!(),
        }
        let spatial_control_join = Arc::new(SpatialControlJoinMetadata::fixture(
            &current_owner.machine,
            packet_transaction,
            packet_callback,
            packet_generation,
            packet_before_digest,
            packet_profile,
        ));
        let audio_view = current_features_audio_view_fixture(
            packet_programme,
            packet_attempt,
            spatial_control_join,
            CallbackPhase::Normal,
            vec![0.0; 48 * 3072],
        )
        .unwrap();
        let flags = flags_packet(
            &current_owner.machine,
            &flags_transaction,
            callback,
            [[1, 1]; 48],
        );
        let before_generation = current_owner.machine.generation();
        let before_digest = current_owner.machine.digest();
        let result = produce_current_features(
            &current_owner.machine,
            &programme_id(),
            ATTEMPT_ID,
            &transaction_id(callback),
            callback,
            CallbackPhase::Normal,
            flags,
            Some(audio_view),
            &current_owner.processor,
            &current_owner.fft,
        );
        let Err(error) = result else {
            panic!("{mismatch} mismatch unexpectedly produced a capability");
        };
        assert_eq!(error, expected_error, "{mismatch}");
        assert_eq!(current_owner.machine.generation(), before_generation);
        assert_eq!(current_owner.machine.digest(), before_digest);
    }
}

#[test]
fn current_features_rejects_a_mismatched_explicit_render_phase_without_staging() {
    let context = context();
    let callback = FrontendCallbackIndex::new(0);
    let join = Arc::new(SpatialControlJoinMetadata::fixture(
        &context.machine,
        transaction_id(callback),
        callback,
        context.machine.generation(),
        None,
        context.machine.profile(),
    ));
    let proposal = render_proposal_fixture(
        programme_id(),
        ATTEMPT_ID,
        join,
        CallbackPhase::CachedDrain,
        vec![0.0; 48 * 3072],
    )
    .unwrap();
    let (audio_view, spatial_view) = proposal.fanout();
    let flags = flags_packet(
        &context.machine,
        &transaction_id(callback),
        callback,
        [[1, 1]; 48],
    );
    let before_generation = context.machine.generation();
    let before_digest = context.machine.digest();

    let result = produce_current_features(
        &context.machine,
        &programme_id(),
        ATTEMPT_ID,
        &transaction_id(callback),
        callback,
        CallbackPhase::Normal,
        flags,
        Some(audio_view),
        &context.processor,
        &context.fft,
    );

    let Err(error) = result else {
        panic!("a mismatched explicit render phase unexpectedly produced a capability");
    };
    assert_eq!(error, CurrentFeaturesError::RenderPhaseMismatch);
    assert_eq!(context.machine.generation(), before_generation);
    assert_eq!(context.machine.digest(), before_digest);
    drop(spatial_view);
}

#[test]
fn current_features_accepts_missing_view_for_reset_sources() {
    let context = context();
    let callback = FrontendCallbackIndex::new(0);
    let transaction = transaction_id(callback);
    let programme = programme_id();
    let parent = SourceFlagsParentLineage::new(transaction.clone(), &context.machine).unwrap();
    let source_flags = current_features_test_packet([[0, 1]; 48], parent);
    let before_generation = context.machine.generation();
    let before_digest = context.machine.digest();

    let result = produce_current_features(
        &context.machine,
        &programme,
        SourceFlagsAttempt::new(0),
        &transaction,
        callback,
        CallbackPhase::Normal,
        source_flags,
        None,
        &context.processor,
        &context.fft,
    );

    let Ok(produced) = result else {
        panic!("all-reset sources unexpectedly required a PostWindow view");
    };
    assert!(
        produced
            .feature_state_proposal
            .source_vector_bits
            .iter()
            .all(|bits| *bits == 0)
    );
    assert!(
        produced
            .feature_state_proposal
            .source_scalar_bits
            .iter()
            .all(|bits| *bits == 0)
    );
    assert!(produced.lineage.render_stage_identity.is_none());
    assert!(
        produced
            .lineage
            .spatial_control_after_proposal_identity
            .is_none()
    );
    assert!(produced.lineage.audio_payload_sha256.is_none());
    assert!(produced.lineage.audio_payload_byte_count.is_none());
    assert_eq!(context.machine.generation(), before_generation);
    assert_eq!(context.machine.digest(), before_digest);
}

#[test]
fn current_features_accepts_missing_view_for_carry_sources() {
    let context = context();
    let callback = FrontendCallbackIndex::new(0);
    let transaction = transaction_id(callback);
    let programme = programme_id();
    let source_flags = flags_packet(&context.machine, &transaction, callback, [[1, 0]; 48]);
    let before_generation = context.machine.generation();
    let before_digest = context.machine.digest();

    let result = produce_current_features(
        &context.machine,
        &programme,
        SourceFlagsAttempt::new(0),
        &transaction,
        callback,
        CallbackPhase::Normal,
        source_flags,
        None,
        &context.processor,
        &context.fft,
    );

    let Ok(produced) = result else {
        panic!("all-carry sources unexpectedly required a PostWindow view");
    };
    assert!(
        produced
            .feature_state_proposal
            .source_vector_bits
            .iter()
            .all(|bits| *bits == 0)
    );
    assert!(
        produced
            .feature_state_proposal
            .source_scalar_bits
            .iter()
            .all(|bits| *bits == 0)
    );
    assert!(produced.lineage.render_stage_identity.is_none());
    assert!(
        produced
            .lineage
            .spatial_control_after_proposal_identity
            .is_none()
    );
    assert!(produced.lineage.audio_payload_sha256.is_none());
    assert!(produced.lineage.audio_payload_byte_count.is_none());
    assert_eq!(context.machine.generation(), before_generation);
    assert_eq!(context.machine.digest(), before_digest);
}

#[test]
fn current_features_requires_audio_for_a_single_read_in_mixed_flags() {
    let context = context();
    let callback = FrontendCallbackIndex::new(0);
    let flags_transaction = transaction_id(callback);
    let programme = programme_id();
    let mut flags = [[0, 1]; 48];
    flags[17] = [0, 0];
    let source_flags = flags_packet(&context.machine, &flags_transaction, callback, flags);
    let before_generation = context.machine.generation();
    let before_digest = context.machine.digest();

    let result = produce_current_features(
        &context.machine,
        &programme,
        SourceFlagsAttempt::new(0),
        &flags_transaction,
        callback,
        CallbackPhase::Normal,
        source_flags,
        None,
        &context.processor,
        &context.fft,
    );

    let Err(error) = result else {
        panic!("one READ source unexpectedly succeeded without a PostWindow view");
    };
    assert_eq!(error, CurrentFeaturesError::ReadAudioMissing);
    assert_eq!(context.machine.generation(), before_generation);
    assert_eq!(context.machine.digest(), before_digest);
}

#[test]
fn current_features_validates_stale_view_for_all_reset_sources() {
    let context = context();
    let callback = FrontendCallbackIndex::new(0);
    let transaction = transaction_id(callback);
    let stale_join = Arc::new(SpatialControlJoinMetadata::fixture(
        &context.machine,
        transaction.clone(),
        callback,
        context.machine.generation() + 1,
        None,
        context.machine.profile(),
    ));
    let audio_view = current_features_audio_view_fixture(
        programme_id(),
        ATTEMPT_ID,
        stale_join,
        CallbackPhase::Normal,
        vec![0.0; 48 * 3072],
    )
    .unwrap();
    let source_flags = flags_packet(&context.machine, &transaction, callback, [[0, 1]; 48]);
    let before_generation = context.machine.generation();
    let before_digest = context.machine.digest();

    let result = produce_current_features(
        &context.machine,
        &programme_id(),
        ATTEMPT_ID,
        &transaction,
        callback,
        CallbackPhase::Normal,
        source_flags,
        Some(audio_view),
        &context.processor,
        &context.fft,
    );

    let Err(error) = result else {
        panic!("all-reset sources accepted a stale PostWindow lineage");
    };
    assert_eq!(error, CurrentFeaturesError::RootIdentityMismatch);
    assert_eq!(context.machine.generation(), before_generation);
    assert_eq!(context.machine.digest(), before_digest);
}

#[test]
fn feature_gate_uses_independent_flags_and_preserves_candidate_features() {
    let context = context();
    let callback_index = frontend_callback_index(0);
    let transaction = transaction_id(callback_index);
    let parent = SourceFlagsParentLineage::new(transaction.clone(), &context.machine).unwrap();
    let mut flags = [[1, 1]; 48];
    flags[0] = [0, 0];
    flags[1] = [1, 0];
    flags[2] = [1, 0];
    flags[5] = [1, 0];
    let source_flags = current_features_test_packet(flags, parent);

    let mut bands = vec![[0_u32; 48]; 24];
    bands[0][1] = 0x3f00_0000;
    bands[0][2] = 0x3eff_ffff;
    bands[0][3] = 0x3f80_0000;
    bands[0][5] = 0x3e80_0000;
    bands[1][5] = 0x3e80_0000;
    let packet =
        CurrentFeaturesPacket::from_producer(callback_index, transaction, bands.into_boxed_slice());

    let gate = gate_current_features(&packet, &source_flags).unwrap();

    assert_eq!(&*gate.candidate_ids, &[1, 5]);
    assert_eq!(&*gate.processing_ids, &[0]);
    assert_eq!(&*gate.response_source_ids, &[0, 1, 5]);
    assert_eq!(gate.candidate_features.len(), 24);
    assert!(gate.candidate_features.iter().all(|band| band.len() == 2));
    assert_eq!(gate.candidate_features[0][0], 0x3f00_0000);
    assert_eq!(gate.candidate_features[0][1], 0x3e80_0000);
    assert_eq!(gate.candidate_features[1][1], 0x3e80_0000);
}
