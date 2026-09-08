// pattern: Functional Core

use jocforge_core::{
    AdmPosition, BlockPartition, CompositionRequest, CompositionResolver, EmdfProfile, JocCarriage,
    JocContainer, JocProfile, LfeOwnership, composition_capability_matrix,
    package_source_derived_short_block_access_unit,
    package_source_derived_short_block_access_unit_with_plan,
    validate_short_multi_dependent_access_unit,
};

fn carrier(seed: f64) -> Vec<f64> {
    (0..1536)
        .map(|sample| (f64::from(sample) * 0.013 + seed).sin() * 0.1)
        .collect()
}

#[test]
fn short_multi_dependent_writer_keeps_one_shared_partition_and_final_owner() {
    let request = CompositionRequest {
        profile: JocProfile::Idx4,
        container: JocContainer::Raw,
        carriage: JocCarriage::EtsiAuxDependent,
        emdf_profile: EmdfProfile::EtsiStrict,
        bitrate_kbps: 2304,
        partitions: vec![BlockPartition::Two; 3],
        dependent_count: 2,
        lfe_ownership: LfeOwnership::DependentSupplement,
    };
    let plan = CompositionResolver::resolve(&request).expect("short multi-dependent plan");
    let ownership = plan.ownership.as_ref().expect("ownership");
    let i0_carriers = (0..5)
        .map(|index| carrier(f64::from(index)))
        .collect::<Vec<_>>();
    let dependent_carriers = ownership
        .dependents
        .iter()
        .enumerate()
        .map(|(dependent, dependent_plan)| {
            (0..dependent_plan.full_band_channel_count())
                .map(|channel| carrier(0.2 + dependent as f64 + channel as f64))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let dependent_lfe_carriers = vec![Some(carrier(9.0)), None];
    let matrix = vec![vec![0.1; 7]];
    let positions = vec![AdmPosition {
        azimuth: 0.0,
        elevation: 0.0,
        distance: 0.0,
    }];

    let bytes = package_source_derived_short_block_access_unit_with_plan(
        &plan,
        1,
        &matrix,
        &positions,
        &i0_carriers,
        None,
        &dependent_carriers,
        &dependent_lfe_carriers,
    )
    .expect("source-derived short multi-dependent bytes");
    let report = validate_short_multi_dependent_access_unit(
        &bytes,
        ownership,
        plan.block_partition.partitions(),
    )
    .expect("short multi-dependent structure");

    assert_eq!(report.syncframes, 9);
    assert_eq!(report.audio_blocks, 6);
    assert_eq!(report.samples, 1536);
    assert_eq!(report.metadata_owner_frame, 8);
    assert!(report.lfe_present);
    assert_eq!(report.complexity_index, 2);
}

#[test]
fn capability_matrix_classifies_cheap_orthogonal_cells_without_encoding_them() {
    let rows = composition_capability_matrix();

    assert!(rows.len() > 10_000);
    assert!(rows.iter().any(|row| {
        row.classification.verdict == jocforge_core::CapabilityVerdict::SupportedPass
    }));
    assert!(rows.iter().any(|row| {
        row.classification.verdict == jocforge_core::CapabilityVerdict::SemanticCeiling
    }));
    assert!(rows.iter().any(|row| {
        row.classification.verdict == jocforge_core::CapabilityVerdict::NotApplicable
    }));
    assert!(
        rows.iter()
            .all(|row| !row.classification.reason.trim().is_empty())
    );
}

#[test]
fn single_dependent_short_api_rejects_a_multi_dependent_plan() {
    let request = CompositionRequest {
        profile: JocProfile::Idx2,
        container: JocContainer::Raw,
        carriage: JocCarriage::EtsiAuxDependent,
        emdf_profile: EmdfProfile::EtsiStrict,
        bitrate_kbps: 1664,
        partitions: vec![BlockPartition::Three, BlockPartition::Three],
        dependent_count: 2,
        lfe_ownership: LfeOwnership::Absent,
    };
    let plan = CompositionResolver::resolve(&request).expect("multi-dependent plan");
    let matrix = vec![vec![0.1; 7]];
    let position = AdmPosition {
        azimuth: 0.0,
        elevation: 0.0,
        distance: 0.0,
    };
    let error = package_source_derived_short_block_access_unit(
        &plan,
        1,
        &matrix,
        &[position],
        &vec![vec![0.1; 1536]; 5],
        &vec![vec![0.2; 1536]; 2],
        None,
    )
    .expect_err("single-dependent API must not silently drop D1");
    assert!(error.contains("exactly one dependent"));
}
