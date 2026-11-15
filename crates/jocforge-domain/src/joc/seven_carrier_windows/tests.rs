use std::sync::Arc;

use super::*;
use crate::{
    frontend::{
        PackageManifestEvidence, ProviderFileEvidence,
        selected_adm_graph::{SelectedAdmGraph, parse_selected_adm_graph},
    },
    profile::{
        Flat7xI0D0TargetPlan, Flat7xProvisionalSourceRoleMap,
        build_flat7x_provisional_source_role_map, project_flat7x_source_pcm_rows,
    },
    spatial::{
        ValidatedF04ParameterPackageManifest, validate_f04_parameter_package_manifest,
        validate_qmf_mathematical_fft_provider, validate_qmf_receiver_functional_provider,
    },
    timeline::{CodecSourceIndex, FrontendCallbackIndex},
    transaction::{AttemptId, CallbackLineageIdentity, CallbackTransactionId, ProgrammeId},
};

const MANIFEST_BYTES: &[u8] = include_bytes!("../../../tests/fixtures/f01e_provider/MANIFEST.json");
const QMF_FUNCTIONAL_BYTES: &[u8] =
    include_bytes!("../../../tests/fixtures/f04_parameters/qmf-receiver-functional-v1.json");
const QMF_FFT_BYTES: &[u8] =
    include_bytes!("../../../tests/fixtures/f04_parameters/qmf-mathematical-fft-v1.json");
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
