// pattern: Imperative Shell

use super::*;
use crate::spatial::spatial_audio_state::fitting_mode0::{
    Mode0FittingProviderInput, assemble_selected_joc_qmf, stage_mode0_fitting_from_receiver,
};
use crate::{
    frontend::{PackageManifestEvidence, ProviderFileEvidence},
    profile::EncodingProfile,
    spatial::{
        DescriptorMathematicalProvider, QmfMathematicalFftProvider, QmfReceiverFunctionalProvider,
        ValidatedWindowQ15Provider,
        post_window_fanout::CallbackPhase,
        source_flags::SourceFlagsAttempt,
        spatial_audio_state::{F04StateEncoder, QmfWindowState, SpatialAudioMachine},
        validate_descriptor_mathematical_provider, validate_f04_parameter_package_manifest,
        validate_qmf_mathematical_fft_provider, validate_qmf_receiver_functional_provider,
        validate_window_q15_provider,
    },
    timeline::{FrontendCallbackIndex, ReceiverCallbackCount},
    transaction::CallbackLineageIdentity,
};
use std::sync::Arc;

const MANIFEST_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/f01e_provider/MANIFEST.json");
const QMF_FUNCTIONAL_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/f04_parameters/qmf-receiver-functional-v1.json");
const QMF_FFT_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/f04_parameters/qmf-mathematical-fft-v1.json");
const WINDOW_Q15_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/f04_parameters/window-q15.json");
const DESCRIPTOR_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/f04_parameters/descriptor-mathematical.json");

#[derive(Clone, Copy)]
struct DirectCase {
    callback_index: u64,
    receiver_generation: u64,
    phase: CallbackPhase,
    type10: &'static [u8],
    target_updates: &'static [u8],
    preconditioned: &'static [u8],
    lane3_before_postfilter: &'static [u8],
    lane3_after_postfilter: &'static [u8],
    s09_input: &'static [u8],
    descriptor_input: &'static [u8],
    final_six: &'static [u8],
    after_model_state: &'static [u8],
}

const DIRECT_CASES: [DirectCase; 3] = [
    DirectCase {
        callback_index: 2,
        receiver_generation: 1,
        phase: CallbackPhase::Normal,
        type10: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case1_cold_normal_type10_f32le.bin"
        ),
        target_updates: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case1_cold_normal_target_updates_f32le.bin"
        ),
        preconditioned: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case1_cold_normal_preconditioned_f32le.bin"
        ),
        lane3_before_postfilter: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case1_cold_normal_lane3_before_postfilter_f32le.bin"
        ),
        lane3_after_postfilter: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case1_cold_normal_lane3_after_postfilter_f32le.bin"
        ),
        s09_input: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case1_cold_normal_s09_input_f32le.bin"
        ),
        descriptor_input: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case1_cold_normal_descriptor_input_f32le.bin"
        ),
        final_six: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case1_cold_normal_final_six_f32le.bin"
        ),
        after_model_state: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case1_cold_normal_after_model_state.bin"
        ),
    },
    DirectCase {
        callback_index: 3,
        receiver_generation: 2,
        phase: CallbackPhase::Normal,
        type10: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case2_warm_normal_type10_f32le.bin"
        ),
        target_updates: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case2_warm_normal_target_updates_f32le.bin"
        ),
        preconditioned: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case2_warm_normal_preconditioned_f32le.bin"
        ),
        lane3_before_postfilter: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case2_warm_normal_lane3_before_postfilter_f32le.bin"
        ),
        lane3_after_postfilter: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case2_warm_normal_lane3_after_postfilter_f32le.bin"
        ),
        s09_input: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case2_warm_normal_s09_input_f32le.bin"
        ),
        descriptor_input: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case2_warm_normal_descriptor_input_f32le.bin"
        ),
        final_six: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case2_warm_normal_final_six_f32le.bin"
        ),
        after_model_state: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case2_warm_normal_after_model_state.bin"
        ),
    },
    DirectCase {
        callback_index: 4,
        receiver_generation: 3,
        phase: CallbackPhase::CachedDrain,
        type10: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case3_cached_drain_type10_f32le.bin"
        ),
        target_updates: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case3_cached_drain_target_updates_f32le.bin"
        ),
        preconditioned: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case3_cached_drain_preconditioned_f32le.bin"
        ),
        lane3_before_postfilter: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case3_cached_drain_lane3_before_postfilter_f32le.bin"
        ),
        lane3_after_postfilter: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case3_cached_drain_lane3_after_postfilter_f32le.bin"
        ),
        s09_input: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case3_cached_drain_s09_input_f32le.bin"
        ),
        descriptor_input: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case3_cached_drain_descriptor_input_f32le.bin"
        ),
        final_six: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case3_cached_drain_final_six_f32le.bin"
        ),
        after_model_state: include_bytes!(
            "../../../../tests/fixtures/receiver_pipeline/case3_cached_drain_after_model_state.bin"
        ),
    },
];

struct ValidatedProviders {
    validated_manifest_sha256: String,
    approved_source_manifest_sha256: String,
    functional: QmfReceiverFunctionalProvider,
    blend: ValidatedWindowQ15Provider,
    fft: QmfMathematicalFftProvider,
    descriptor: DescriptorMathematicalProvider,
}

impl ValidatedProviders {
    fn input(&self) -> SpatialAudioReceiverProviderInput<'_> {
        SpatialAudioReceiverProviderInput {
            functional: Some(&self.functional),
            blend: Some(&self.blend),
            fft: Some(&self.fft),
            descriptor: Some(&self.descriptor),
            validated_manifest_sha256: Some(&self.validated_manifest_sha256),
            selected_manifest_sha256: Some(&self.approved_source_manifest_sha256),
        }
    }
}

fn validated_providers() -> ValidatedProviders {
    validated_providers_from_manifest(
        MANIFEST_BYTES,
        QMF_FUNCTIONAL_BYTES,
        WINDOW_Q15_BYTES,
        QMF_FFT_BYTES,
        DESCRIPTOR_BYTES,
        "shared F01E manifest fixture",
    )
}

fn validated_receiver_subset_providers() -> ValidatedProviders {
    const SUBSET_MANIFEST: &[u8] = include_bytes!(
        "../../../../../../outputs/IMPLEMENTER_SUPPLEMENTS/F04-RECEIVER-APPROVED-PARAMETERS-V1/MANIFEST.json"
    );
    const FUNCTIONAL: &[u8] = include_bytes!(
        "../../../../../../outputs/IMPLEMENTER_SUPPLEMENTS/F04-RECEIVER-APPROVED-PARAMETERS-V1/parameters/qmf-receiver-functional-v1.json"
    );
    const WINDOW: &[u8] = include_bytes!(
        "../../../../../../outputs/IMPLEMENTER_SUPPLEMENTS/F04-RECEIVER-APPROVED-PARAMETERS-V1/parameters/window-q15.json"
    );
    const FFT: &[u8] = include_bytes!(
        "../../../../../../outputs/IMPLEMENTER_SUPPLEMENTS/F04-RECEIVER-APPROVED-PARAMETERS-V1/parameters/qmf-mathematical-fft-v1.json"
    );
    const DESCRIPTOR: &[u8] = include_bytes!(
        "../../../../../../outputs/IMPLEMENTER_SUPPLEMENTS/F04-RECEIVER-APPROVED-PARAMETERS-V1/parameters/descriptor-mathematical.json"
    );
    validated_providers_from_manifest(
        SUBSET_MANIFEST,
        FUNCTIONAL,
        WINDOW,
        FFT,
        DESCRIPTOR,
        "Wash F04 receiver subset manifest",
    )
}

fn validated_providers_from_manifest(
    manifest_bytes: &[u8],
    functional_bytes: &[u8],
    window_bytes: &[u8],
    fft_bytes: &[u8],
    descriptor_bytes: &[u8],
    source_name: &str,
) -> ValidatedProviders {
    let manifest = validate_f04_parameter_package_manifest(&PackageManifestEvidence::new(
        Arc::from(manifest_bytes),
        Some(source_name.to_owned()),
    ))
    .expect("the F04 provider manifest validates");
    let functional = validate_qmf_receiver_functional_provider(
        &ProviderFileEvidence::new(
            Arc::from(functional_bytes),
            Some("QMF receiver functional fixture".to_owned()),
        ),
        &manifest,
    )
    .expect("selected QMF receiver provider validates");
    let blend = validate_window_q15_provider(
        &ProviderFileEvidence::new(
            Arc::from(window_bytes),
            Some("window Q15 fixture".to_owned()),
        ),
        &manifest,
    )
    .expect("selected window provider validates");
    let fft = validate_qmf_mathematical_fft_provider(
        &ProviderFileEvidence::new(
            Arc::from(fft_bytes),
            Some("QMF mathematical FFT fixture".to_owned()),
        ),
        &manifest,
    )
    .expect("selected FFT provider validates");
    let descriptor = validate_descriptor_mathematical_provider(
        &ProviderFileEvidence::new(
            Arc::from(descriptor_bytes),
            Some("descriptor mathematical fixture".to_owned()),
        ),
        &manifest,
    )
    .expect("selected descriptor provider validates");
    ValidatedProviders {
        validated_manifest_sha256: manifest.manifest_sha256().to_hex(),
        approved_source_manifest_sha256: manifest.approved_source_manifest_sha256().to_owned(),
        functional,
        blend,
        fft,
        descriptor,
    }
}

#[test]
fn receiver_accepts_the_wash_subset_with_its_approved_parent_identity() {
    let providers = validated_receiver_subset_providers();
    assert_eq!(
        providers.validated_manifest_sha256,
        "161147c7959083738f5426b2280457454b08b73ee58a075322c33880cecc818b"
    );
    assert_eq!(
        providers.approved_source_manifest_sha256,
        "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a"
    );
    let proposal = run_case(DIRECT_CASES[0], ModelState::cold(), &providers);
    assert_eq!(
        encode_model_state(&proposal.after),
        DIRECT_CASES[0].after_model_state
    );
}

fn parent(callback_index: u64) -> SpatialAudioJoinMetadata {
    let profile = EncodingProfile::selected();
    let state = SpatialAudioMachine::new_cold(profile).expect("selected spatial-audio root");
    let programme_id = crate::transaction::test_programme_id("f04r-direct-test");
    let attempt = SourceFlagsAttempt::new(7);
    let callback_index = FrontendCallbackIndex::new(u32::try_from(callback_index).unwrap());
    SpatialAudioJoinMetadata::new(
        programme_id.clone(),
        attempt,
        CallbackLineageIdentity::new(programme_id, attempt, callback_index),
        callback_index,
        state.generation(),
        state.digest(),
        profile,
    )
}

fn f32_words(bytes: &[u8]) -> Vec<f32> {
    assert_eq!(bytes.len() % 4, 0);
    bytes
        .chunks_exact(4)
        .map(|word| f32::from_bits(u32::from_le_bytes(word.try_into().unwrap())))
        .collect()
}

fn assert_f32_fixture(label: &str, actual: &[f32], expected_bytes: &[u8]) {
    assert_eq!(expected_bytes.len(), actual.len() * 4);
    for (index, (actual, expected)) in actual
        .iter()
        .zip(expected_bytes.chunks_exact(4))
        .enumerate()
    {
        assert_eq!(
            actual.to_bits(),
            u32::from_le_bytes(expected.try_into().unwrap()),
            "{label} F32LE word {index}"
        );
    }
}

fn assert_finite_fixture(actual: &[FiniteF32], expected_bytes: &[u8]) {
    assert_eq!(expected_bytes.len(), actual.len() * 4);
    for (index, (actual, expected)) in actual
        .iter()
        .zip(expected_bytes.chunks_exact(4))
        .enumerate()
    {
        assert_eq!(
            actual.to_bits(),
            u32::from_le_bytes(expected.try_into().unwrap()),
            "F32LE word {index}"
        );
    }
}

fn encode_model_state(state: &ModelState) -> Vec<u8> {
    let mut encoder = F04StateEncoder::new();
    state
        .encode(&mut encoder)
        .expect("ModelState has canonical field order");
    encoder.finish()
}

fn stage_case(
    case: DirectCase,
    before: ModelState,
    providers: SpatialAudioReceiverProviderInput<'_>,
    type10_values: &[f32],
    target_values: &[f32],
    target_generation: ReceiverCallbackCount,
    include_target_receipt: bool,
) -> Result<SpatialAudioReceiverProposal, SpatialAudioReceiverError> {
    let parent = parent(case.callback_index);
    let callback_index = FrontendCallbackIndex::new(u32::try_from(case.callback_index).unwrap());
    let generation = ReceiverCallbackCount::new(case.receiver_generation);
    let type10 = Type10ReceiverInput::from_direct_vector_for_test(
        &parent,
        case.phase,
        callback_index,
        generation,
        type10_values,
    );
    let target_updates = include_target_receipt.then(|| {
        TargetUpdatesReceipt::from_direct_vector_for_test(
            &parent,
            case.phase,
            callback_index,
            target_generation,
            target_values,
        )
    });
    stage_spatial_audio_receiver(SpatialAudioReceiverStageInput {
        parent: &parent,
        phase: case.phase,
        receiver_callback_count_before: ReceiverCallbackCount::new(case.callback_index - 2),
        before,
        type10,
        target_updates,
        providers,
    })
}

fn assert_stage_error(
    case: DirectCase,
    before: ModelState,
    providers: SpatialAudioReceiverProviderInput<'_>,
    type10_values: &[f32],
    target_values: &[f32],
    expected_code: &str,
) {
    assert_eq!(
        stage_case(
            case,
            before,
            providers,
            type10_values,
            target_values,
            ReceiverCallbackCount::new(case.receiver_generation),
            true,
        )
        .unwrap_err()
        .code(),
        expected_code
    );
}

fn run_case(
    case: DirectCase,
    before: ModelState,
    providers: &ValidatedProviders,
) -> SpatialAudioReceiverProposal {
    let type10 = f32_words(case.type10);
    let targets = f32_words(case.target_updates);
    stage_case(
        case,
        before,
        providers.input(),
        &type10,
        &targets,
        ReceiverCallbackCount::new(case.receiver_generation),
        true,
    )
    .expect("direct F04R case succeeds")
}

#[test]
fn direct_receiver_kat_compares_literals_after_one_continuous_actual_state_chain() {
    let providers = validated_providers();
    let mut before = ModelState::cold();

    for case in DIRECT_CASES {
        let proposal = run_case(case, before, &providers);
        for object in 0..15 {
            let expected_start = (object + 1) * FRAME_SAMPLES;
            assert_eq!(
                proposal.target_qmf_input[object].as_ref(),
                &proposal.trace.preconditioned[expected_start..expected_start + FRAME_SAMPLES]
            );
        }
        assert_eq!(
            u64::from(proposal.callback_index.value()),
            proposal.source_index.value() + 2
        );
        assert_eq!(proposal.dispatch.value(), proposal.source_index.value() + 1);
        assert_eq!(
            proposal.dispatch.generated_writer().unwrap().value(),
            proposal.source_index.value()
        );
        assert_eq!(
            proposal.descriptor_target_gain_word,
            u32::from_le_bytes(proposal.after.descriptor_state.gain.0)
        );
        assert!(proposal.descriptor_selected_index >= 128);
        assert!(proposal.s09_return.iter().all(|row| row.len() == 1_536));
        assert_f32_fixture(
            "preconditioned",
            &proposal.trace.preconditioned,
            case.preconditioned,
        );
        assert_f32_fixture(
            "lane3 before postfilter",
            &proposal.trace.lane3_before_postfilter,
            case.lane3_before_postfilter,
        );
        assert_f32_fixture(
            "lane3 after postfilter",
            &proposal.trace.lane3_after_postfilter,
            case.lane3_after_postfilter,
        );
        assert_f32_fixture("s09 input", &proposal.trace.s09_input, case.s09_input);
        assert_f32_fixture(
            "descriptor input",
            &proposal.trace.descriptor_input,
            case.descriptor_input,
        );
        assert_finite_fixture(
            &proposal
                .final_six
                .iter()
                .flatten()
                .copied()
                .collect::<Vec<_>>(),
            case.final_six,
        );
        assert_eq!(encode_model_state(&proposal.after), case.after_model_state);
        before = proposal.after;
    }
}

#[test]
fn actual_receiver_output_feeds_one_qmf_projection_without_receiver_reexecution() {
    let providers = validated_providers();
    let receiver_before = ModelState::cold();
    let fitting_before = QmfWindowState::cold();
    let parent = parent(2);
    let receiver = run_case(DIRECT_CASES[0], ModelState::cold(), &providers);
    let fitting = stage_mode0_fitting_from_receiver(
        &parent,
        &fitting_before,
        &receiver_before,
        &receiver,
        Mode0FittingProviderInput::Ready {
            functional: &providers.functional,
            fft: &providers.fft,
            descriptor: &providers.descriptor,
        },
    )
    .expect("actual receiver values satisfy fitting-QMF lineage");
    let proposal =
        assemble_selected_joc_qmf(receiver, fitting).expect("same-generation QMF values assemble");
    let projection = &proposal.projection;
    assert_eq!(projection.callback_index().value(), 2);
    assert_eq!(projection.source_index().value(), 0);
    assert_eq!(projection.dispatch().value(), 1);
    assert_eq!(projection.receiver_generation_after().value(), 1);
    assert!(projection.descriptor_selected_index() >= 128);
    assert!(f32::from_bits(projection.descriptor_target_gain_word()).is_finite());
    assert!(projection.s09_return().iter().all(|row| row.len() == 1_536));
    assert!(projection.final_six().iter().all(|row| row.len() == 1_536));
    for object in 0..15 {
        for slot in 0..48 {
            for bin in 0..64 {
                assert_eq!(
                    projection.target_object_complex(object, slot, bin),
                    projection.target_packed_complex(object / 3, slot, (object % 3) * 64 + bin)
                );
            }
        }
    }
    assert!(projection.target_packed_complex(5, 0, 0).is_none());
    assert!(projection.carrier_complex(5, 0, 0).is_none());
    assert_eq!(proposal.receiver_after.analysis_history.len(), 5);
    assert_eq!(proposal.fitting_after.previous_targets.len(), 15);
}

fn run_case_with_expected_after(
    case: DirectCase,
    before: ModelState,
    providers: &ValidatedProviders,
    expected_after: &[u8],
) -> (ModelState, bool) {
    let proposal = run_case(case, before, providers);
    let actual_after = encode_model_state(&proposal.after);
    let expected_matches = actual_after == expected_after;
    (proposal.after, expected_matches)
}

#[test]
fn mutating_expected_after_literals_cannot_change_the_computed_next_case_state() {
    let providers = validated_providers();
    let mut poisoned_expected = DIRECT_CASES[0].after_model_state.to_vec();
    poisoned_expected[0] ^= 0x01;

    let (actual_after_original_check, original_matches) = run_case_with_expected_after(
        DIRECT_CASES[0],
        ModelState::cold(),
        &providers,
        DIRECT_CASES[0].after_model_state,
    );
    let (actual_after_poisoned_check, poisoned_matches) = run_case_with_expected_after(
        DIRECT_CASES[0],
        ModelState::cold(),
        &providers,
        &poisoned_expected,
    );
    assert!(original_matches);
    assert!(!poisoned_matches);
    assert_eq!(
        encode_model_state(&actual_after_original_check),
        encode_model_state(&actual_after_poisoned_check)
    );

    let next = run_case(DIRECT_CASES[1], actual_after_poisoned_check, &providers);
    assert_eq!(
        encode_model_state(&next.after),
        DIRECT_CASES[1].after_model_state
    );
}

#[test]
fn each_direct_error_case_maps_to_its_approved_outcome() {
    let providers = validated_providers();
    let case = DIRECT_CASES[0];

    let mut type10 = f32_words(case.type10);
    let mut targets = f32_words(case.target_updates);

    let missing = SpatialAudioReceiverProviderInput {
        functional: None,
        blend: Some(&providers.blend),
        fft: Some(&providers.fft),
        descriptor: Some(&providers.descriptor),
        validated_manifest_sha256: Some(&providers.validated_manifest_sha256),
        selected_manifest_sha256: Some(&providers.approved_source_manifest_sha256),
    };
    assert_eq!(
        stage_case(
            case,
            ModelState::cold(),
            missing,
            &type10,
            &targets,
            ReceiverCallbackCount::new(case.receiver_generation),
            true,
        )
        .unwrap_err()
        .code(),
        "F04R_PROVIDER_MISSING"
    );

    let mut wrong_identity = providers.input();
    wrong_identity.selected_manifest_sha256 = Some("wrong-manifest-identity");
    assert_eq!(
        stage_case(
            case,
            ModelState::cold(),
            wrong_identity,
            &type10,
            &targets,
            ReceiverCallbackCount::new(case.receiver_generation),
            true,
        )
        .unwrap_err()
        .code(),
        "F04R_PROVIDER_IDENTITY"
    );

    assert_eq!(
        stage_case(
            case,
            ModelState::cold(),
            providers.input(),
            &type10,
            &targets,
            ReceiverCallbackCount::new(99),
            true,
        )
        .unwrap_err()
        .code(),
        "F04R_LINEAGE_TARGET_UPDATES"
    );

    let mut invalid_model_state = ModelState::cold();
    invalid_model_state.post_filter_state.bytes = Box::new([]);
    assert_eq!(
        stage_case(
            case,
            invalid_model_state,
            providers.input(),
            &type10,
            &targets,
            ReceiverCallbackCount::new(case.receiver_generation),
            true,
        )
        .unwrap_err()
        .code(),
        "F04R_STATE_SHAPE"
    );

    let mut invalid_cursor = ModelState::cold();
    invalid_cursor.temporal_states[0].index_28 = 2 * invalid_cursor.temporal_states[0].span + 1;
    assert_eq!(
        stage_case(
            case,
            invalid_cursor,
            providers.input(),
            &type10,
            &targets,
            ReceiverCallbackCount::new(case.receiver_generation),
            true,
        )
        .unwrap_err()
        .code(),
        "F04R_TEMPORAL_CURSOR"
    );

    assert_eq!(
        stage_case(
            case,
            ModelState::cold(),
            providers.input(),
            &type10[..1],
            &targets,
            ReceiverCallbackCount::new(case.receiver_generation),
            true,
        )
        .unwrap_err()
        .code(),
        "F04R_TYPE10_SHAPE"
    );

    assert_eq!(
        stage_case(
            case,
            ModelState::cold(),
            providers.input(),
            &type10,
            &targets[..1],
            ReceiverCallbackCount::new(case.receiver_generation),
            true,
        )
        .unwrap_err()
        .code(),
        "F04R_TARGET_SHAPE"
    );

    type10[0] = f32::NAN;
    assert_eq!(
        stage_case(
            case,
            ModelState::cold(),
            providers.input(),
            &type10,
            &targets,
            ReceiverCallbackCount::new(case.receiver_generation),
            true,
        )
        .unwrap_err()
        .code(),
        "F04R_INPUT_NONFINITE"
    );

    type10.fill(1.0);
    targets.fill(f32::MAX);
    assert_eq!(
        stage_case(
            case,
            ModelState::cold(),
            providers.input(),
            &type10,
            &targets,
            ReceiverCallbackCount::new(case.receiver_generation),
            true,
        )
        .unwrap_err()
        .code(),
        "F04R_NUMERIC_NONFINITE"
    );
}

#[test]
fn competing_direct_errors_follow_the_approved_first_failure_order() {
    let providers = validated_providers();
    let case = DIRECT_CASES[0];
    let mut type10 = f32_words(case.type10);
    let mut targets = f32_words(case.target_updates);
    type10[0] = f32::NAN;
    targets.fill(f32::MAX);

    let mut missing_provider = providers.input();
    missing_provider.functional = None;
    missing_provider.selected_manifest_sha256 = Some("wrong-manifest-identity");
    let stale_generation = ReceiverCallbackCount::new(99);
    assert_stage_error(
        case,
        invalid_model_state(),
        missing_provider,
        &type10[..1],
        &targets[..1],
        "F04R_PROVIDER_MISSING",
    );

    let mut wrong_identity = providers.input();
    wrong_identity.selected_manifest_sha256 = Some("wrong-manifest-identity");
    assert_stage_error(
        case,
        ModelState::cold(),
        wrong_identity,
        &type10[..1],
        &targets[..1],
        "F04R_PROVIDER_IDENTITY",
    );

    assert_eq!(
        stage_case(
            case,
            invalid_model_state(),
            providers.input(),
            &type10[..1],
            &targets[..1],
            stale_generation,
            true,
        )
        .unwrap_err()
        .code(),
        "F04R_LINEAGE_TARGET_UPDATES"
    );

    assert_stage_error(
        case,
        invalid_model_state(),
        providers.input(),
        &type10[..1],
        &targets[..1],
        "F04R_STATE_SHAPE",
    );

    let mut invalid_cursor = ModelState::cold();
    invalid_cursor.temporal_states[0].index_28 = 2 * invalid_cursor.temporal_states[0].span + 1;
    assert_stage_error(
        case,
        invalid_cursor,
        providers.input(),
        &type10[..1],
        &targets[..1],
        "F04R_TEMPORAL_CURSOR",
    );

    assert_stage_error(
        case,
        ModelState::cold(),
        providers.input(),
        &type10[..1],
        &targets[..1],
        "F04R_TYPE10_SHAPE",
    );

    assert_stage_error(
        case,
        ModelState::cold(),
        providers.input(),
        &type10,
        &targets[..1],
        "F04R_TARGET_SHAPE",
    );

    assert_stage_error(
        case,
        ModelState::cold(),
        providers.input(),
        &type10,
        &targets,
        "F04R_INPUT_NONFINITE",
    );
}

fn invalid_model_state() -> ModelState {
    let mut state = ModelState::cold();
    state.post_filter_state.bytes = Box::new([]);
    state
}
