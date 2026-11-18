use std::sync::Arc;

use super::*;
use crate::{
    codec::owner::Flat7xD0SourceGenerationReceipt,
    frontend::{
        PackageManifestEvidence, ProviderFileEvidence,
        selected_adm_graph::{SelectedAdmGraph, parse_selected_adm_graph},
    },
    joc::coefficient_fitting::{CoefficientFitState, stage_seven_carrier_coefficient_fit_from_qmf},
    joc::coefficient_payload::Flat7xCoefficientRows,
    joc::coefficient_representation::{JocCoefficientRowLayout, RepresentationRetryProvider},
    joc::programme_state::{SelectedJocState, stage_selected_joc_source_with_seven_carrier_fit},
    joc::{coefficient_fitting::fit_seven_carrier_window_set, fit_score::FitScoreProvider},
    profile::{
        EncodingProfile, Flat7xI0D0TargetPlan, Flat7xProvisionalSourceRoleMap,
        JocReconstructionLayout, build_flat7x_provisional_source_role_map,
        project_flat7x_source_pcm_rows, resolve_selected_profile,
    },
    spatial::{
        ValidatedF04ParameterPackageManifest,
        source_flags::{SourceFlagsAttempt, SourceFlagsProgrammeId},
        spatial_audio_state::{SelectedJocQmfProjection, SpatialAudioMachine},
        validate_f04_parameter_package_manifest, validate_qmf_mathematical_fft_provider,
        validate_qmf_receiver_functional_provider,
    },
    timeline::{CodecSourceIndex, FrontendCallbackIndex},
    transaction::{AttemptId, CallbackLineageIdentity, CallbackTransactionId, ProgrammeId},
};

const MANIFEST_BYTES: &[u8] = include_bytes!("../../../tests/fixtures/f01e_provider/MANIFEST.json");
const QMF_FUNCTIONAL_BYTES: &[u8] =
    include_bytes!("../../../tests/fixtures/f04_parameters/qmf-receiver-functional-v1.json");
const QMF_FFT_BYTES: &[u8] =
    include_bytes!("../../../tests/fixtures/f04_parameters/qmf-mathematical-fft-v1.json");
const JOC_FUNCTIONAL_BYTES: &[u8] = include_bytes!(
    "../../../tests/fixtures/synthetic_qmf_fit_checkpoints/approved-joc-functional.json"
);
const AXML: &[u8] =
    include_bytes!("../../../tests/fixtures/selected_adm_graph/source_metadata.xml");
const CHNA: &[u8] =
    include_bytes!("../../../tests/fixtures/selected_adm_graph/source_track_associations.bin");

const PCM_SAMPLES_PER_SOURCE: usize = 1_536;
const PHYSICAL_ROWS: usize = 38;
const CHNA_HEADER_BYTES: usize = 4;
const CHNA_ENTRY_BYTES: usize = 40;

fn validated_providers() -> (QmfReceiverFunctionalProvider, QmfMathematicalFftProvider) {
    let manifest_evidence = PackageManifestEvidence::new(Arc::from(MANIFEST_BYTES), None);
    let manifest: ValidatedF04ParameterPackageManifest =
        validate_f04_parameter_package_manifest(&manifest_evidence).unwrap();
    let functional = validate_qmf_receiver_functional_provider(
        &ProviderFileEvidence::new(Arc::from(QMF_FUNCTIONAL_BYTES), None),
        &manifest,
    )
    .unwrap();
    let fft = validate_qmf_mathematical_fft_provider(
        &ProviderFileEvidence::new(Arc::from(QMF_FFT_BYTES), None),
        &manifest,
    )
    .unwrap();
    (functional, fft)
}

fn selected_graph() -> SelectedAdmGraph {
    let end = CHNA_HEADER_BYTES + PHYSICAL_ROWS * CHNA_ENTRY_BYTES;
    let mut chna = CHNA[..end].to_vec();
    let count = u16::try_from(PHYSICAL_ROWS).unwrap().to_le_bytes();
    chna[..2].copy_from_slice(&count);
    chna[2..4].copy_from_slice(&count);
    parse_selected_adm_graph(AXML, &chna).unwrap()
}

fn selected_role_map() -> Flat7xProvisionalSourceRoleMap {
    build_flat7x_provisional_source_role_map(
        &selected_graph(),
        Flat7xI0D0TargetPlan::approved_contract(),
    )
    .unwrap()
}

fn source_identity(
    source: u32,
    callback: u32,
    analysis_generation: u64,
) -> SevenCarrierQmfSourceIdentity {
    SevenCarrierQmfSourceIdentity::new(
        CallbackLineageIdentity::new(
            ProgrammeId::new("seven-carrier-qmf-test").unwrap(),
            AttemptId::new(1),
            FrontendCallbackIndex::new(callback),
        ),
        CallbackTransactionId::new(&format!("seven-carrier-source-{source}")).unwrap(),
        CodecSourceIndex::new(u64::from(source)),
        AnalysisOwnerGeneration::from_parent_snapshot(analysis_generation),
    )
    .unwrap()
}

fn projected_input<'a>(
    role_map: &Flat7xProvisionalSourceRoleMap,
    physical_rows: &'a [Vec<u32>],
) -> SevenCarrierPcmInput<'a> {
    project_flat7x_source_pcm_rows(role_map, physical_rows)
        .unwrap()
        .seven_carrier_input()
}

#[test]
fn qmf_windows_keep_seven_lane_histories_and_previous_current_slot_order() {
    let role_map = selected_role_map();
    let mut first_source_rows = vec![vec![0_u32; PCM_SAMPLES_PER_SOURCE]; PHYSICAL_ROWS];
    let left_row = usize::from(role_map.joc_carrier_rows()[0].row().physical_host().value());
    first_source_rows[left_row][PCM_SAMPLES_PER_SOURCE - 64..].fill(1.0_f32.to_bits());
    let first_input = projected_input(&role_map, &first_source_rows);
    let second_source_rows = vec![vec![0_u32; PCM_SAMPLES_PER_SOURCE]; PHYSICAL_ROWS];
    let second_input = projected_input(&role_map, &second_source_rows);
    let (functional, fft) = validated_providers();

    let first = stage_seven_carrier_windows(
        first_input,
        source_identity(0, 2, 1),
        &SevenCarrierQmfState::cold(),
        &functional,
        &fft,
    )
    .unwrap();
    assert_eq!(first.windows().role_bindings(), role_map.joc_carrier_rows());
    for binding in role_map.joc_carrier_rows() {
        let windows = first.windows().windows_for_role(binding.role()).unwrap();
        assert!(windows[..24].iter().flatten().all(|word| *word == 0));
        if binding.role() == Flat7xJocCarrierRole::Left {
            assert!(
                windows[24..]
                    .iter()
                    .flatten()
                    .any(|word| f32::from_bits(*word) != 0.0)
            );
        } else {
            assert!(
                windows
                    .iter()
                    .flatten()
                    .all(|word| f32::from_bits(*word) == 0.0)
            );
        }
    }

    let second = stage_seven_carrier_windows(
        second_input,
        source_identity(1, 3, 2),
        first.after_state(),
        &functional,
        &fft,
    )
    .unwrap();
    let left_windows = second
        .windows()
        .windows_for_role(Flat7xJocCarrierRole::Left)
        .unwrap();
    assert_eq!(
        &left_windows[..24],
        &first
            .windows()
            .windows_for_role(Flat7xJocCarrierRole::Left)
            .unwrap()[24..]
    );
    assert!(
        left_windows[24..]
            .iter()
            .flatten()
            .any(|word| f32::from_bits(*word) != 0.0)
    );
    for role in [
        Flat7xJocCarrierRole::Right,
        Flat7xJocCarrierRole::Center,
        Flat7xJocCarrierRole::LeftSurround,
        Flat7xJocCarrierRole::RightSurround,
        Flat7xJocCarrierRole::LeftRearSurround,
        Flat7xJocCarrierRole::RightRearSurround,
    ] {
        assert!(
            second
                .windows()
                .windows_for_role(role)
                .unwrap()
                .iter()
                .flatten()
                .all(|word| f32::from_bits(*word) == 0.0)
        );
    }
    let canonical_state = second.after_state().canonical_payload_bytes().unwrap();
    assert!(canonical_state.starts_with(b"SEVQMF01"));
    assert_eq!(
        canonical_state,
        second.after_state().canonical_payload_bytes().unwrap()
    );
}

#[test]
fn qmf_window_stage_rejects_a_skipped_source_generation() {
    let role_map = selected_role_map();
    let physical_rows = vec![vec![0_u32; PCM_SAMPLES_PER_SOURCE]; PHYSICAL_ROWS];
    let input = projected_input(&role_map, &physical_rows);
    let (functional, fft) = validated_providers();
    let first = stage_seven_carrier_windows(
        input,
        source_identity(0, 2, 1),
        &SevenCarrierQmfState::cold(),
        &functional,
        &fft,
    )
    .unwrap();

    assert_eq!(
        stage_seven_carrier_windows(
            input,
            source_identity(2, 4, 3),
            first.after_state(),
            &functional,
            &fft,
        ),
        Err(SevenCarrierWindowError::SourceSequenceMismatch)
    );
}

#[test]
fn seven_carrier_window_fit_emits_fifteen_by_seven_by_twelve_rows() {
    let role_map = selected_role_map();
    let physical_rows = vec![vec![0_u32; PCM_SAMPLES_PER_SOURCE]; PHYSICAL_ROWS];
    let input = projected_input(&role_map, &physical_rows);
    let (functional, fft) = validated_providers();
    let qmf = stage_seven_carrier_windows(
        input,
        source_identity(0, 2, 1),
        &SevenCarrierQmfState::cold(),
        &functional,
        &fft,
    )
    .unwrap();
    let target_windows = vec![0_u32; 15 * 48 * 64 * 2];
    let weights = [1.0_f32.to_bits(); 48];
    let score = FitScoreProvider::constant_for_test(0.0).unwrap();
    let fit =
        fit_seven_carrier_window_set(&target_windows, qmf.windows(), &weights, &score).unwrap();

    assert_eq!(fit.full_words().len(), 15 * 7 * 12);
    assert_eq!(fit.sparse_words().len(), 15 * 12);
    assert_eq!(fit.selected_channels().len(), 15 * 12);
    assert!(fit.selected_channels().iter().all(|channel| *channel < 7));
    assert_eq!(
        fit.row_layout(),
        crate::joc::coefficient_representation::JocCoefficientRowLayout::SevenChannel
    );
    assert_eq!(fit.payload_downmix_index().value(), 1);
    assert_eq!(fit.qmf_source_identity().source_index().value(), 0);
    assert!(!fit.production_allowed());

    let payload_input = fit.coefficient_payload_input(128, 0, 511, None).unwrap();
    let flat7x = resolve_selected_profile(
        &EncodingProfile::selected_with_joc_reconstruction_layout(JocReconstructionLayout::Flat7x),
    )
    .unwrap();
    let outcome = Flat7xCoefficientRows::new(&payload_input)
        .unwrap()
        .write_payload(flat7x)
        .unwrap();
    assert_eq!(outcome.object_count().value(), 15);
    assert!(outcome.payload().is_some());
}

#[test]
fn flat7x_fit_crosses_selected_dispatch_and_writes_seven_row_id14() {
    let role_map = selected_role_map();
    let physical_rows = vec![vec![0_u32; PCM_SAMPLES_PER_SOURCE]; PHYSICAL_ROWS];
    let input = projected_input(&role_map, &physical_rows);
    let (functional, fft) = validated_providers();
    let before_qmf = SevenCarrierQmfState::cold();
    let qmf = stage_seven_carrier_windows(
        input,
        source_identity(0, 2, 1),
        &before_qmf,
        &functional,
        &fft,
    )
    .unwrap();
    let programme_id = ProgrammeId::new("seven-carrier-qmf-test").unwrap();
    let attempt = AttemptId::new(1);
    let callback = FrontendCallbackIndex::new(2);
    let callback_lineage = CallbackLineageIdentity::new(programme_id.clone(), attempt, callback);
    let spatial = SpatialAudioMachine::new_cold(
        EncodingProfile::selected_with_joc_reconstruction_layout(JocReconstructionLayout::Flat7x),
    )
    .unwrap();
    let target_projection = SelectedJocQmfProjection::identity_only_for_test(
        SourceFlagsProgrammeId::new(programme_id.as_str()).unwrap(),
        SourceFlagsAttempt::new(attempt.value()),
        callback_lineage,
        0,
        spatial.digest(),
        callback,
        CodecSourceIndex::new(0),
    )
    .with_target_object_windows_for_test(vec![0_u32; 15 * 48 * 64 * 2]);
    let profile = resolve_selected_profile(
        &EncodingProfile::selected_with_joc_reconstruction_layout(JocReconstructionLayout::Flat7x),
    )
    .unwrap();
    let score = FitScoreProvider::constant_for_test(0.0).unwrap();
    let fit = stage_seven_carrier_coefficient_fit_from_qmf(
        profile,
        &CoefficientFitState::cold(),
        &target_projection,
        qmf.windows(),
        qmf.after_state(),
        &functional,
        &score,
    )
    .unwrap();
    assert!(fit.after().seven_carrier_qmf_state().is_some());
    assert_eq!(fit.rows().full_words().len(), 15 * 7 * 12);
    assert_eq!(
        fit.rows().row_layout(),
        JocCoefficientRowLayout::SevenChannel
    );
    assert_eq!(fit.rows().payload_downmix_index().value(), 1);

    let payload_input = fit
        .rows()
        .coefficient_payload_input(128, 0, 372, None)
        .unwrap();
    assert_eq!(payload_input.downmix_index, 1);
    assert_eq!(payload_input.objects.len(), 15);
    assert!(payload_input.objects.iter().all(|object| {
        object.present
            && object.points.len() == 1
            && object.points[0].channel_rows.len() == 7
            && object.points[0]
                .channel_rows
                .iter()
                .all(|row| row.len() == 12)
    }));

    // This test deliberately crosses the production selected-dispatch function. Its body is
    // written by stage_joc_payload_with_seven_carrier_fit, not a test-only payload helper.
    let fit_source = fit.qmf_source_identity();
    let producer_transaction_id = fit_source.source_transaction_id().clone();
    let source_generation_receipt = Flat7xD0SourceGenerationReceipt::synthetic_for_test(
        producer_transaction_id.clone(),
        1,
        target_projection.transaction_id().clone(),
        target_projection.source_index(),
        fit_source.analysis_generation().value(),
    );
    // The provider's pinned identity is over LF bytes; normalize Windows autocrlf checkout bytes.
    let joc_functional_lf = std::str::from_utf8(JOC_FUNCTIONAL_BYTES)
        .unwrap()
        .replace("\r\n", "\n");
    let retry_provider =
        RepresentationRetryProvider::from_approved_bytes(joc_functional_lf.as_bytes()).unwrap();
    let selected_before = SelectedJocState::cold();
    let selected_after = stage_selected_joc_source_with_seven_carrier_fit(
        &selected_before,
        &target_projection,
        &producer_transaction_id,
        &fit,
        &source_generation_receipt,
        &retry_provider,
        profile,
    )
    .unwrap();
    let id14_body = &selected_after.id14_body().body().bytes;
    assert!(id14_body.len() >= 2);
    assert_eq!(id14_body[0] >> 5, 1); // actual ID14 reconstruction/downmix index
    let object_count_minus_one =
        (u16::from(id14_body[0] & 0x1f) << 1) | u16::from(id14_body[1] >> 7);
    assert_eq!(object_count_minus_one + 1, 15);

    let selected_joc = selected_after.after();
    let canonical = selected_joc.canonical_bytes().unwrap();
    assert!(canonical.starts_with(b"JOCSEL01"));
    assert!(canonical.windows(8).any(|window| window == b"J7QMF001"));
    assert!(canonical.len() > 22_790);
    assert_eq!(selected_joc.gtc_owner_snapshot().unwrap().generation(), 1);
}
