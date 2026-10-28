use super::*;
use crate::{
    frontend::{
        PackageManifestEvidence, ProviderFileEvidence, ValidatedFeatureFft4096MathProvider,
        ValidatedFeatureProcessorMathProvider, validate_current_features_package_manifest,
        validate_f01e_package_manifest, validate_f01e_provider, validate_f03_package_manifest,
        validate_f03_target_cold_provider, validate_feature_fft4096_math_provider,
        validate_feature_processor_math_provider,
    },
    meter::{
        installed48_renderer::{
            MeterInstalled48Input, MeterInstalled48Proposal, MeterInstalled48State,
            stage_meter_installed48,
        },
        sample_block_queue::{MeterCallbackIndex, MeterSourceSampleCount},
    },
    profile::{EncodingProfile, resolve_selected_profile},
    spatial::candidate_selector::SelectorStatus,
    spatial::{
        SpatialControlMachine,
        frontend_stages::stage_frontend_features_from_installed48,
        installed48_activity::{
            PrimaryActivityInput, PrimaryActivityProposal, PrimaryInstalled48ActivityProducer,
        },
        post_window_fanout::CallbackPhase,
        selected_p15_geometry::generate_selected_p15_geometry,
        source_flags::{
            BeforeRegularHistory, CurrentHostPcm, PrefixWriterSet, SourceFlagsInput,
            SourceFlagsParentLineage, project_normal_control_snapshot_source_controls,
        },
        source_geometry_outer::stage_normal_source_geometry,
        spatial_audio_state::{SpatialAudioDigest, SpatialAudioMachine},
        spatial_control::SpatialControlDigest,
        validate_component_parameter_provider, validate_f04_parameter_package_manifest,
        validate_window_q15_provider,
    },
    timeline::AdmSampleIndex,
};
use serde_json::Value;
use std::sync::Arc;

const MANIFEST_BYTES: &[u8] = include_bytes!("../../../tests/fixtures/f01e_provider/MANIFEST.json");
const F01E_PROVIDER_BYTES: &[u8] =
    include_bytes!("../../../tests/fixtures/f01e_provider/f01e-selected-constructor-v1.json");
const F03_PROVIDER_BYTES: &[u8] =
    include_bytes!("../../../tests/fixtures/target_cold/f03-selected-target-cold-v1.json");
const FEATURE_PROCESSOR_BYTES: &[u8] =
    include_bytes!("../../../tests/fixtures/current_features/feature-processor-math-v1.json");
const FEATURE_FFT_BYTES: &[u8] =
    include_bytes!("../../../tests/fixtures/current_features/feature-fft4096-math-v1.json");
const WINDOW_Q15_BYTES: &[u8] =
    include_bytes!("../../../tests/fixtures/f04_parameters/window-q15.json");
const COMPONENT_PARAMETERS_BYTES: &[u8] = include_bytes!(
    "../../../tests/fixtures/f04_parameters/component-parameter-mathematical-v1.json"
);
const SELECTOR_VECTORS_BYTES: &[u8] =
    include_bytes!("../../../tests/fixtures/f04_component/unit-vectors.json");

struct CallbackOwners {
    spatial_control: SpatialControlMachine,
    spatial_audio: SpatialAudioMachine,
    reader_target: ReaderTargetState,
    processor: ValidatedFeatureProcessorMathProvider,
    fft: ValidatedFeatureFft4096MathProvider,
    window: ValidatedWindowQ15Provider,
    parameters: ValidatedComponentParameterProvider,
}

fn callback_owners() -> CallbackOwners {
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
    let current_features_manifest = validate_current_features_package_manifest(&manifest).unwrap();
    let processor = validate_feature_processor_math_provider(
        &ProviderFileEvidence::new(Arc::from(FEATURE_PROCESSOR_BYTES), None),
        &current_features_manifest,
    )
    .unwrap();
    let fft = validate_feature_fft4096_math_provider(
        &ProviderFileEvidence::new(Arc::from(FEATURE_FFT_BYTES), None),
        &current_features_manifest,
    )
    .unwrap();
    let f04_manifest = validate_f04_parameter_package_manifest(&manifest).unwrap();
    let window = validate_window_q15_provider(
        &ProviderFileEvidence::new(Arc::from(WINDOW_Q15_BYTES), None),
        &f04_manifest,
    )
    .unwrap();
    let parameters = validate_component_parameter_provider(
        &ProviderFileEvidence::new(Arc::from(COMPONENT_PARAMETERS_BYTES), None),
        &f04_manifest,
        &profile,
    )
    .unwrap();
    CallbackOwners {
        spatial_control: SpatialControlMachine::new_cold(profile, &f01e, &f03).unwrap(),
        spatial_audio: SpatialAudioMachine::new_cold(profile).unwrap(),
        reader_target: ReaderTargetState::cold(),
        processor,
        fft,
        window,
        parameters,
    }
}

fn vector_ids(value: &Value) -> Vec<u32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| u32::try_from(item.as_u64().unwrap()).unwrap())
        .collect()
}

fn vector_matrix(value: &Value) -> Vec<Vec<u32>> {
    value.as_array().unwrap().iter().map(vector_ids).collect()
}

fn selector_input_from_vector(value: &Value) -> CandidateSelectorInput {
    CandidateSelectorInput {
        candidate_ids: vector_ids(&value["candidate_ids"]),
        feature_u32: vector_matrix(&value["feature_u32"]),
        positions_u32: value["positions_u32"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| {
                let words = vector_ids(row);
                [words[0], words[1], words[2]]
            })
            .collect(),
        width_u32: vector_ids(&value["width_u32"]),
        attributes_u32: vector_ids(&value["attributes_u32"]),
        response_u32: vector_matrix(&value["response_u32"]),
        previous_ids: vector_ids(&value["previous_ids"]),
    }
}

fn stage_activity_and_installed48(
    fixture: &super::super::source_geometry_outer::tests::F01bF01eFixture,
) -> (PrimaryActivityProposal, MeterInstalled48Proposal) {
    let producer = PrimaryInstalled48ActivityProducer::new(&fixture.source_map).unwrap();
    let before = producer.cold_state().unwrap();
    let raw_pcm = vec![vec![0_u32; 1_536]; 38];
    let activity = producer
        .produce_next(
            &fixture.source_map,
            &before,
            PrimaryActivityInput::NormalRead {
                reader_relative_sample: AdmSampleIndex::new(0),
                pcm_word_rows: raw_pcm.clone(),
            },
        )
        .unwrap();
    let source_binding = fixture.source_map.source_binding();
    let installed_before = MeterInstalled48State::cold(
        *source_binding.source_receipt_sha256(),
        MeterSourceSampleCount::new(source_binding.source_samples().value()).unwrap(),
    );
    let installed48 = stage_meter_installed48(
        &installed_before,
        MeterInstalled48Input {
            source_sha256: *source_binding.source_receipt_sha256(),
            callback_index: MeterCallbackIndex::new(0),
            source_cursor_before: AdmSampleIndex::new(0),
            host_input_samples: 1_536,
            source_padding_samples: 0,
            raw_pcm_rows: &raw_pcm,
            activity: activity.output(),
            membership_route: activity.membership_route(),
            source_map: &fixture.source_map,
            controls: &fixture.primary_controls,
            diffusion_parameters: None,
        },
    )
    .unwrap();
    (activity, installed48)
}

fn run_callback(
    warm_candidate_source_count: usize,
) -> (
    SpatialProductionCallbackProposal,
    CallbackOwners,
    SpatialControlDigest,
    SpatialAudioDigest,
    [u8; 32],
) {
    let fixture = super::super::source_geometry_outer::tests::f01b_f01e_fixture();
    let mut owners = callback_owners();
    if warm_candidate_source_count > 0 {
        let mut feature_vector = vec![0_u32; 48 * 24];
        for source in 11..11 + warm_candidate_source_count {
            for band in 0..24 {
                feature_vector[source * 24 + band] = 1.0_f32.to_bits();
            }
        }
        owners
            .spatial_control
            .set_current_features_test_history(&feature_vector, &[0; 48]);
    }
    let before_control = owners.spatial_control.digest();
    let before_audio = owners.spatial_audio.digest();
    let before_reader = owners.reader_target.canonical_sha256();
    let (activity, installed48) = stage_activity_and_installed48(&fixture);
    let resolved_profile = resolve_selected_profile(&owners.spatial_control.profile()).unwrap();
    let source_projection = project_normal_control_snapshot_source_controls(
        &fixture.normal,
        &fixture.graph,
        resolved_profile,
    )
    .unwrap();
    let source_control_clock = source_projection.clock();
    let before_history =
        BeforeRegularHistory::from_owner(&owners.spatial_control, source_projection.route())
            .unwrap();
    let prefix_writers =
        PrefixWriterSet::new(activity.output().prefix_writer_indices().to_vec()).unwrap();
    let parent = SourceFlagsParentLineage::new(
        fixture.normal.transaction_id().clone(),
        &owners.spatial_control,
    )
    .unwrap();
    let source_flags_input = SourceFlagsInput::from_primary_regular_source_controls(
        CurrentHostPcm::new(vec![vec![0.0_f32; 1_536]; 38]).unwrap(),
        Some(before_history),
        source_projection,
        prefix_writers,
        parent,
    )
    .unwrap();
    let frontend = stage_frontend_features_from_installed48(
        &owners.spatial_control,
        fixture.normal.transaction_id().clone(),
        activity,
        installed48,
        source_flags_input,
        CallbackPhase::Normal,
        &owners.processor,
        &owners.fft,
    )
    .unwrap();
    let normal_source_geometry =
        stage_normal_source_geometry(&fixture.graph, &fixture.source_map, fixture.normal).unwrap();
    let p15_geometry = generate_selected_p15_geometry(0x159b, true).unwrap();
    let proposal = continue_spatial_production_callback_from_normal_stage(
        SpatialProductionNormalCallbackInputs {
            frontend,
            normal_source_geometry,
            source_control_clock,
            source_map: &fixture.source_map,
            p15_geometry: &p15_geometry,
            spatial_control: &owners.spatial_control,
            spatial_audio: &owners.spatial_audio,
            reader_target: &owners.reader_target,
            window: &owners.window,
            parameters: &owners.parameters,
        },
    )
    .unwrap();
    (
        proposal,
        owners,
        before_control,
        before_audio,
        before_reader,
    )
}

fn assert_stages_share_one_callback(proposal: &SpatialProductionCallbackProposal) {
    let callback = proposal.callback_index();
    assert_eq!(proposal.response().callback_index(), callback);
    assert_eq!(
        proposal.response().transaction_id(),
        proposal.transaction_id()
    );
    assert_eq!(proposal.target_context().callback_index(), callback);
    assert_eq!(
        proposal.target_context().transaction_id(),
        proposal.transaction_id()
    );
    assert_eq!(proposal.compact_matrix().value_generation(), callback);
    assert_eq!(
        proposal.compact_matrix().transaction_id(),
        proposal.transaction_id()
    );
    assert_eq!(proposal.component_rows().callback_index(), callback);
    assert_eq!(
        proposal.component_rows().transaction_id(),
        proposal.transaction_id()
    );
    assert_eq!(
        proposal.reader_target.next_state().callback_index(),
        FrontendCallbackIndex::new(callback.value() + 1)
    );
    assert_eq!(proposal.canonical_dense_d_f32_bits().len(), 15 * 48);
}

#[test]
fn empty_selection_keeps_empty_c_d_and_stages_all_pending_owners_once() {
    let (proposal, owners, before_control, before_audio, before_reader) = run_callback(0);
    assert_stages_share_one_callback(&proposal);
    assert!(proposal.selector().is_none());
    assert!(proposal.selected_targets().xyz_words.is_empty());
    assert!(proposal.compact_matrix().destinations().is_empty());
    assert!(proposal.compact_matrix().c_rows_u32().is_empty());
    assert!(
        proposal
            .canonical_dense_d_f32_bits()
            .iter()
            .all(|word| *word == 0)
    );
    assert_eq!(
        proposal.component_rows().callback_index(),
        FrontendCallbackIndex::new(0)
    );
    assert_eq!(proposal.component.blocks_processed(), 24);
    assert!(proposal.reader_target.pending_receipt().is_none());
    assert_eq!(owners.spatial_control.digest(), before_control);
    assert_eq!(owners.spatial_audio.digest(), before_audio);
    assert_eq!(owners.reader_target.canonical_sha256(), before_reader);
}

#[test]
fn all_zero_active_c_keeps_d_and_advances_component_without_committing_owners() {
    let (proposal, owners, before_control, before_audio, before_reader) = run_callback(1);
    assert_stages_share_one_callback(&proposal);
    let selector = proposal
        .selector()
        .expect("warm feature creates a candidate");
    assert_eq!(selector.selected_count(), 1);
    assert_eq!(selector.status(), SelectorStatus::CandidatesExhausted);
    assert!(proposal.response().processing_ids().is_empty());
    assert_eq!(proposal.compact_matrix().destinations().len(), 1);
    assert_eq!(proposal.compact_matrix().c_rows_u32().len(), 1);
    assert!(
        proposal
            .compact_matrix()
            .c_rows_u32()
            .iter()
            .flatten()
            .all(|word| *word == 0)
    );
    assert!(
        proposal
            .canonical_dense_d_f32_bits()
            .iter()
            .all(|word| *word == 0)
    );
    assert!(
        proposal
            .component_rows()
            .canonical_rows_f32_bits()
            .iter()
            .all(|word| *word == 0)
    );
    assert_eq!(proposal.component.blocks_processed(), 24);
    assert_eq!(owners.spatial_control.digest(), before_control);
    assert_eq!(owners.spatial_audio.digest(), before_audio);
    assert_eq!(owners.reader_target.canonical_sha256(), before_reader);
}

#[test]
fn capacity_reached_runs_u04_and_preserves_selected_identity_through_installation() {
    let (proposal, owners, before_control, before_audio, before_reader) = run_callback(37);
    assert_stages_share_one_callback(&proposal);
    let selector = proposal
        .selector()
        .expect("regular source feature history admits candidates");
    assert_eq!(selector.status(), SelectorStatus::CapacityReached);
    assert_eq!(selector.selected_count(), 15);
    let refinement = proposal
        .u04_refinement()
        .expect("CAPACITY_REACHED executes the approved U04 producer");
    assert_eq!(refinement.selected_target_ids(), selector.selected_ids());
    assert_eq!(
        refinement.selected_target_attributes(),
        selector.selected_attributes()
    );
    assert_eq!(refinement.projected_xyz_u32().len(), 15);
    assert_eq!(proposal.selected_targets().ids_u32, selector.selected_ids());
    assert_eq!(
        proposal.selected_targets().attributes_u32,
        selector.selected_attributes()
    );
    assert_eq!(proposal.installation().next_state().active_flags, [1; 15]);
    assert_eq!(proposal.component.blocks_processed(), 24);
    assert_eq!(owners.spatial_control.digest(), before_control);
    assert_eq!(owners.spatial_audio.digest(), before_audio);
    assert_eq!(owners.reader_target.canonical_sha256(), before_reader);
}

#[test]
fn callback_u04_runs_only_for_capacity_and_preserves_selector_domains() {
    let vectors: Value = serde_json::from_slice(SELECTOR_VECTORS_BYTES).unwrap();
    let cases = vectors["cases"].as_array().unwrap();
    for (case_id, expected_u04) in [("U-SEL-CAPACITY", true), ("U-SEL-COMPLEMENT", false)] {
        let case = cases.iter().find(|case| case["id"] == case_id).unwrap();
        let input = selector_input_from_vector(&case["input"]);
        let result = candidate_selector(input.clone()).unwrap();
        assert_eq!(
            result.status() == SelectorStatus::CapacityReached,
            expected_u04
        );
        let initial_selected_positions = result
            .selected_initial_ordinals()
            .iter()
            .map(|ordinal| input.positions_u32[usize::try_from(*ordinal).unwrap()])
            .collect::<Vec<_>>();
        let (refinement, selected) = finish_selection_with_u04(&input, &result).unwrap();
        assert_eq!(refinement.is_some(), expected_u04);
        assert_eq!(selected.ids_u32, result.selected_ids());
        assert_eq!(selected.attributes_u32, result.selected_attributes());
        if let Some(refinement) = refinement {
            assert_eq!(refinement.source_ids(), input.candidate_ids);
            assert_eq!(refinement.selected_target_ids(), result.selected_ids());
            assert_eq!(
                refinement.selected_target_attributes(),
                result.selected_attributes()
            );
            assert_eq!(selected.xyz_words, refinement.projected_xyz_u32());
        } else {
            assert_eq!(selected.xyz_words, initial_selected_positions);
        }
    }
}
