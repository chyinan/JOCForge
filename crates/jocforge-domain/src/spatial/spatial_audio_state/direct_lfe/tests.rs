// pattern: Imperative Shell
use super::super::LfeDelayRingState;
use super::*;
use crate::spatial::post_window_fanout::CallbackPhase;
use crate::{
    frontend::{
        PackageManifestEvidence, ProviderFileEvidence, validate_f01e_package_manifest,
        validate_f01e_provider, validate_f03_package_manifest, validate_f03_target_cold_provider,
    },
    profile::EncodingProfile,
    spatial::{
        source_flags::{
            HostRole, HostRouteRecord, SourceFlagsAttempt, SourceFlagsProgrammeId,
            ValidatedSemanticRoute,
        },
        spatial_audio_state::{SpatialAudioJoinMetadata, SpatialAudioMachine},
        spatial_control::{SpatialControlJoinMetadata, SpatialControlMachine},
    },
    timeline::FrontendCallbackIndex,
    transaction::{AttemptId, CallbackLineageIdentity},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fmt::Write as FmtWrite, sync::Arc};

const MANIFEST_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/f01e_provider/MANIFEST.json");
const F01E_PROVIDER_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/f01e_provider/f01e-selected-constructor-v1.json");
const F03_PROVIDER_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/target_cold/f03-selected-target-cold-v1.json");
const VECTOR_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/direct_lfe/INPUT-ONLY-VECTORS.json");
const G1_ERRATUM_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/direct_lfe/G1-VECTOR-ERRATUM.json");
const SUCCESS_IDS: [&str; 6] = ["g0", "g1", "g2", "g3", "g4", "g5"];

struct TestContext {
    spatial_audio: SpatialAudioMachine,
    spatial_control: SpatialControlMachine,
    profile: EncodingProfile,
}

fn context() -> TestContext {
    let profile = EncodingProfile::selected();
    let manifest = PackageManifestEvidence::new(Arc::from(MANIFEST_BYTES), None);
    let f01e_manifest = validate_f01e_package_manifest(&manifest).unwrap();
    let f01e = validate_f01e_provider(
        &ProviderFileEvidence::new(Arc::from(F01E_PROVIDER_BYTES), None),
        &f01e_manifest,
    )
    .unwrap();
    let f03_manifest = validate_f03_package_manifest(&manifest).unwrap();
    let f03 = validate_f03_target_cold_provider(
        &ProviderFileEvidence::new(Arc::from(F03_PROVIDER_BYTES), None),
        &f03_manifest,
    )
    .unwrap();
    TestContext {
        spatial_audio: SpatialAudioMachine::new_cold(profile).unwrap(),
        spatial_control: SpatialControlMachine::new_cold(profile, &f01e, &f03).unwrap(),
        profile,
    }
}

fn route(lfe_host: u8) -> ValidatedSemanticRoute {
    let mut destination = 11_u8;
    let records = (0..38_u8)
        .map(|host| {
            if host == lfe_host {
                HostRouteRecord::new(host, HostRole::Lfe, true, None).unwrap()
            } else {
                let record =
                    HostRouteRecord::new(host, HostRole::Regular, true, Some(destination)).unwrap();
                destination += 1;
                record
            }
        })
        .collect();
    ValidatedSemanticRoute::new(records).unwrap()
}

fn direct_lfe_programme_id() -> SourceFlagsProgrammeId {
    crate::transaction::test_programme_id("direct-lfe-test-programme")
}

fn direct_lfe_transaction_id(callback_index: FrontendCallbackIndex) -> CallbackLineageIdentity {
    CallbackLineageIdentity::new(direct_lfe_programme_id(), AttemptId::new(7), callback_index)
}

fn input_view(
    context: &TestContext,
    callback_index: FrontendCallbackIndex,
    phase: CallbackPhase,
    samples: Vec<f32>,
    transaction_label: &str,
) -> Result<DirectLfeInputView, DirectLfeError> {
    let programme_id = if transaction_label == "foreign-transaction" {
        crate::transaction::test_programme_id("foreign-direct-lfe-programme")
    } else {
        direct_lfe_programme_id()
    };
    DirectLfeInputView::try_from_spatial_control(
        SpatialControlLfeIngressAuthority::fixture(),
        programme_id.clone(),
        SourceFlagsAttempt::new(7),
        CallbackLineageIdentity::new(programme_id, AttemptId::new(7), callback_index),
        callback_index,
        phase,
        context.profile,
        &context.spatial_control,
        &route(17),
        samples,
    )
}

fn spatial_audio_join(
    context: &TestContext,
    callback_index: FrontendCallbackIndex,
) -> SpatialAudioJoinMetadata {
    SpatialAudioJoinMetadata::new(
        direct_lfe_programme_id(),
        SourceFlagsAttempt::new(7),
        direct_lfe_transaction_id(callback_index),
        callback_index,
        context.spatial_audio.generation(),
        context.spatial_audio.digest(),
        context.profile,
    )
}

fn spatial_control_join(
    context: &TestContext,
    callback_index: FrontendCallbackIndex,
) -> SpatialControlJoinMetadata {
    SpatialControlJoinMetadata::fixture(
        &context.spatial_control,
        direct_lfe_transaction_id(callback_index),
        callback_index,
        context.spatial_control.generation(),
        None,
        context.profile,
    )
}

fn vectors() -> Value {
    serde_json::from_slice(VECTOR_BYTES).expect("washed Direct-LFE vector fixture is valid JSON")
}

fn phase(value: &str) -> CallbackPhase {
    match value {
        "NORMAL" => CallbackPhase::Normal,
        "CACHED_DRAIN" => CallbackPhase::CachedDrain,
        unknown => panic!("unregistered callback phase: {unknown}"),
    }
}

fn row_from_recipe(recipe: &Value, cached_g1: Option<&[u8]>) -> Vec<f32> {
    if recipe == "ZERO" {
        return vec![0.0; 1536];
    }
    if recipe == "CACHED_EXACT_G1" {
        return cached_g1
            .expect("cached drain follows the g1 normal row")
            .chunks_exact(4)
            .map(|bytes| f32::from_bits(u32::from_le_bytes(bytes.try_into().unwrap())))
            .collect();
    }
    let default_word = recipe["default_u32"].as_u64().unwrap() as u32;
    let mut words = vec![default_word; 1536];
    for override_word in recipe["overrides"].as_array().unwrap() {
        let sample_index = override_word["sample_index"].as_u64().unwrap() as usize;
        let word = override_word["f32_u32"].as_u64().unwrap() as u32;
        words[sample_index] = word;
    }
    assert_eq!(recipe["negative_zero_indexes"], json!([]));
    words.into_iter().map(f32::from_bits).collect()
}

fn row_bytes(samples: &[f32]) -> Vec<u8> {
    samples
        .iter()
        .flat_map(|sample| sample.to_bits().to_le_bytes())
        .collect()
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut result = String::with_capacity(64);
    for byte in digest {
        write!(&mut result, "{byte:02x}").unwrap();
    }
    result
}

fn reconstruct_g1_erratum(erratum: &Value) -> Vec<u8> {
    let word_count = erratum["sample_count"].as_u64().unwrap() as usize;
    let default_word = erratum["default_f32_u32"].as_u64().unwrap() as u32;
    let mut bytes = Vec::with_capacity(word_count * 4);
    for _ in 0..word_count {
        bytes.extend_from_slice(&default_word.to_le_bytes());
    }
    for override_word in erratum["overrides"].as_array().unwrap() {
        let sample_index = override_word["sample_index"].as_u64().unwrap() as usize;
        let word = override_word["f32_u32"].as_u64().unwrap() as u32;
        let offset = sample_index * 4;
        bytes[offset..offset + 4].copy_from_slice(&word.to_le_bytes());
    }
    bytes
}

fn ring_snapshot(state: &LfeDelayRingState) -> Value {
    let row_hashes: Vec<Option<String>> = state
        .rows
        .iter()
        .map(|row| row.as_ref().map(|bytes| sha256(bytes)))
        .collect();
    let indexes: Vec<Option<u32>> = state
        .input_frame_indexes
        .iter()
        .map(|index| index.map(FrontendCallbackIndex::value))
        .collect();
    json!({
        "write_index": state.write_index,
        "read_index": state.read_index,
        "row_f32le_sha256": row_hashes,
        "input_frame_indexes": indexes,
        "model_samples_consumed": state.model_samples_consumed,
    })
}

#[test]
fn direct_lfe_g0_through_g5_match_actual_first_vectors() {
    let context = context();
    let vectors = vectors();
    let g1_erratum: Value = serde_json::from_slice(G1_ERRATUM_BYTES).unwrap();
    let cases = vectors["cases"].as_array().unwrap();
    assert_eq!(cases.len(), SUCCESS_IDS.len());
    let mut state = LfeDelayRingState::cold();
    let mut cached_g1: Option<Vec<u8>> = None;

    for (generation, case) in cases.iter().enumerate() {
        assert_eq!(case["id"], SUCCESS_IDS[generation]);
        let input = &case["input"];
        assert_eq!(input["generation"].as_u64(), Some(generation as u64));
        let callback_index = FrontendCallbackIndex::new(u32::try_from(generation).unwrap());
        let callback_phase = phase(input["phase"].as_str().unwrap());
        let samples = row_from_recipe(&input["recipe"], cached_g1.as_deref());
        let actual_input_bytes = row_bytes(&samples);
        assert_eq!(actual_input_bytes.len(), 6144);
        assert_eq!(
            sha256(&actual_input_bytes),
            input["current_lfe_f32le_sha256"].as_str().unwrap()
        );
        if generation == 1 {
            let reconstructed = reconstruct_g1_erratum(&g1_erratum);
            assert_eq!(actual_input_bytes, reconstructed);
            assert_eq!(
                actual_input_bytes.len(),
                g1_erratum["expected_bytes"].as_u64().unwrap() as usize
            );
            assert_eq!(
                sha256(&actual_input_bytes),
                g1_erratum["expected_sha256"].as_str().unwrap()
            );
            assert!(actual_input_bytes[731 * 4..].iter().all(|byte| *byte == 0));
            cached_g1 = Some(actual_input_bytes.clone());
        }

        assert_eq!(ring_snapshot(&state), case["expected_before"]);
        let input = input_view(
            &context,
            callback_index,
            callback_phase,
            samples,
            "direct-lfe-test-transaction",
        )
        .unwrap();
        let proposal = stage_lfe_ring_transition(
            &state,
            callback_index,
            callback_phase,
            input,
            context.spatial_audio.generation(),
            context.spatial_audio.digest(),
        )
        .expect("approved ring transition succeeds");
        let expected_output = &case["expected_output"];
        let expected_available = expected_output["output_available"].as_bool().unwrap();
        assert_eq!(proposal.type10_row0.is_some(), expected_available);
        if let Some(row0) = proposal.type10_row0.as_ref() {
            assert_eq!(
                Some(row0.origin_generation.value()),
                expected_output["output_origin_generation"]
                    .as_u64()
                    .map(|value| u32::try_from(value).unwrap())
            );
            assert_eq!(
                sha256(&row0.row_bytes),
                expected_output["output_f32le_sha256"].as_str().unwrap()
            );
            assert_eq!(row0.callback_generation, callback_index);
        }

        assert_eq!(ring_snapshot(&proposal.after), case["expected_after"]);
        let expected_type10 = &case["expected_type10"];
        assert_eq!(
            expected_type10["available"].as_bool(),
            Some(expected_available)
        );
        assert_eq!(
            expected_type10["rows_1_15_origin_generation"]
                .as_u64()
                .map(|value| u32::try_from(value).unwrap()),
            expected_available.then_some(u32::try_from(generation).unwrap())
        );
        state = proposal.after;
    }

    assert_eq!(
        state.model_samples_consumed,
        vectors["summary"]["terminal_model_samples_consumed"]
            .as_i64()
            .unwrap()
    );
}

#[test]
fn direct_lfe_failure_vectors_reject_without_mutating_committed_owners() {
    let context = context();
    let vectors = vectors();
    let before_spatial_audio = context.spatial_audio.digest();
    let before_spatial_control = context.spatial_control.digest();

    for failure in vectors["failure_cases"].as_array().unwrap() {
        let case_id = failure["id"].as_str().unwrap();
        let result = match case_id {
            "stale_generation" => {
                let current = FrontendCallbackIndex::new(0);
                let stale = input_view(
                    &context,
                    FrontendCallbackIndex::new(1),
                    CallbackPhase::Normal,
                    vec![0.0; 1536],
                    "direct-lfe-test-transaction",
                )
                .unwrap();
                produce_direct_lfe_stage(
                    &context.spatial_audio,
                    &spatial_audio_join(&context, current),
                    &spatial_control_join(&context, current),
                    &context.spatial_control,
                    CallbackPhase::Normal,
                    stale,
                )
                .map(|_| ())
            }
            "short_shape" => input_view(
                &context,
                FrontendCallbackIndex::new(0),
                CallbackPhase::Normal,
                vec![0.0; 1535],
                "direct-lfe-test-transaction",
            )
            .map(|_| ()),
            "nonfinite" => {
                let mut samples = vec![0.0; 1536];
                samples[0] = f32::INFINITY;
                input_view(
                    &context,
                    FrontendCallbackIndex::new(0),
                    CallbackPhase::Normal,
                    samples,
                    "direct-lfe-test-transaction",
                )
                .map(|_| ())
            }
            unknown => panic!("unregistered failure vector: {unknown}"),
        };
        let expected_local_error = if case_id == "stale_generation" {
            "F04_CALLBACK_ORDER"
        } else {
            failure["expected_error"].as_str().unwrap()
        };
        assert_eq!(
            result.err().map(DirectLfeError::code),
            Some(expected_local_error)
        );
        assert_eq!(failure["state_unchanged"].as_bool(), Some(true));
        assert_eq!(context.spatial_audio.generation(), 0);
        assert_eq!(context.spatial_audio.digest(), before_spatial_audio);
        assert_eq!(context.spatial_control.generation(), 0);
        assert_eq!(context.spatial_control.digest(), before_spatial_control);
    }
}

#[test]
fn successful_direct_lfe_stage_is_discardable_and_has_no_local_commit() {
    let context = context();
    let callback_index = FrontendCallbackIndex::new(0);
    let before_digest = context.spatial_audio.digest();
    let input = input_view(
        &context,
        callback_index,
        CallbackPhase::Normal,
        vec![0.0; 1536],
        "direct-lfe-test-transaction",
    )
    .unwrap();
    let proposal = produce_direct_lfe_stage(
        &context.spatial_audio,
        &spatial_audio_join(&context, callback_index),
        &spatial_control_join(&context, callback_index),
        &context.spatial_control,
        CallbackPhase::Normal,
        input,
    )
    .expect("matching lineage produces a linear proposal");
    assert_eq!(proposal.after.model_samples_consumed, 1536);
    assert!(proposal.type10_row0.is_none());
    assert_eq!(proposal.lineage.lfe_host.value(), 17);
    assert_eq!(context.spatial_audio.generation(), 0);
    assert_eq!(context.spatial_audio.digest(), before_digest);
}

#[test]
fn direct_lfe_profile_mismatch_rejects_without_staging() {
    let context = context();
    let before_digest = context.spatial_audio.digest();
    let result = DirectLfeInputView::try_from_spatial_control(
        SpatialControlLfeIngressAuthority::fixture(),
        direct_lfe_programme_id(),
        SourceFlagsAttempt::new(7),
        direct_lfe_transaction_id(FrontendCallbackIndex::new(0)),
        FrontendCallbackIndex::new(0),
        CallbackPhase::Normal,
        EncodingProfile::selected_with_carriage(crate::profile::EmdfCarriageProfile::EtsiEmdf),
        &context.spatial_control,
        &route(17),
        vec![0.0; 1536],
    );
    assert_eq!(
        result.err().map(DirectLfeError::code),
        Some("F04_PROFILE_UNSUPPORTED")
    );
    assert_eq!(context.spatial_audio.generation(), 0);
    assert_eq!(context.spatial_audio.digest(), before_digest);
}

#[test]
fn direct_lfe_transaction_mismatch_rejects_without_staging() {
    let context = context();
    let callback_index = FrontendCallbackIndex::new(0);
    let before_digest = context.spatial_audio.digest();
    let input = input_view(
        &context,
        callback_index,
        CallbackPhase::Normal,
        vec![0.0; 1536],
        "foreign-transaction",
    )
    .unwrap();
    let result = produce_direct_lfe_stage(
        &context.spatial_audio,
        &spatial_audio_join(&context, callback_index),
        &spatial_control_join(&context, callback_index),
        &context.spatial_control,
        CallbackPhase::Normal,
        input,
    );
    assert_eq!(
        result.err().map(DirectLfeError::code),
        Some("F04_INPUT_ERROR")
    );
    assert_eq!(context.spatial_audio.generation(), 0);
    assert_eq!(context.spatial_audio.digest(), before_digest);
}

#[test]
fn direct_lfe_invalid_ring_cursor_rejects_before_input_lineage() {
    let mut context = context();
    context.spatial_audio.committed.root.lfe_state.write_index = 3;
    let callback_index = FrontendCallbackIndex::new(0);
    let input = input_view(
        &context,
        callback_index,
        CallbackPhase::Normal,
        vec![0.0; 1536],
        "foreign-transaction",
    )
    .unwrap();
    let result = produce_direct_lfe_stage(
        &context.spatial_audio,
        &spatial_audio_join(&context, callback_index),
        &spatial_control_join(&context, callback_index),
        &context.spatial_control,
        CallbackPhase::Normal,
        input,
    );
    assert_eq!(
        result.err().map(DirectLfeError::code),
        Some("F04_STATE_ERROR")
    );
}

#[test]
fn direct_lfe_in_range_but_incoherent_ring_cursors_reject() {
    let mut context = context();
    context.spatial_audio.committed.root.lfe_state.read_index = 2;
    let callback_index = FrontendCallbackIndex::new(0);
    let input = input_view(
        &context,
        callback_index,
        CallbackPhase::Normal,
        vec![0.0; 1536],
        "direct-lfe-test-transaction",
    )
    .unwrap();
    let result = produce_direct_lfe_stage(
        &context.spatial_audio,
        &spatial_audio_join(&context, callback_index),
        &spatial_control_join(&context, callback_index),
        &context.spatial_control,
        CallbackPhase::Normal,
        input,
    );
    assert_eq!(
        result.err().map(DirectLfeError::code),
        Some("F04_STATE_ERROR")
    );
}

#[test]
fn direct_lfe_sample_count_matches_completed_frontend_callbacks() {
    let mut context = context();
    context
        .spatial_audio
        .committed
        .root
        .lfe_state
        .model_samples_consumed = 1536;
    let callback_index = FrontendCallbackIndex::new(0);
    let input = input_view(
        &context,
        callback_index,
        CallbackPhase::Normal,
        vec![0.0; 1536],
        "direct-lfe-test-transaction",
    )
    .unwrap();
    let result = produce_direct_lfe_stage(
        &context.spatial_audio,
        &spatial_audio_join(&context, callback_index),
        &spatial_control_join(&context, callback_index),
        &context.spatial_control,
        CallbackPhase::Normal,
        input,
    );
    assert_eq!(
        result.err().map(DirectLfeError::code),
        Some("F04_STATE_ERROR")
    );
}

#[test]
fn direct_lfe_phase_is_an_explicit_input_not_inferred_from_generation() {
    let context = context();
    let input = input_view(
        &context,
        FrontendCallbackIndex::new(0),
        CallbackPhase::CachedDrain,
        vec![0.0; 1536],
        "direct-lfe-test-transaction",
    )
    .unwrap();
    let result = stage_lfe_ring_transition(
        &LfeDelayRingState::cold(),
        FrontendCallbackIndex::new(0),
        CallbackPhase::Normal,
        input,
        0,
        context.spatial_audio.digest(),
    );
    assert_eq!(
        result.err().map(DirectLfeError::code),
        Some("F04_CALLBACK_ORDER")
    );
}
