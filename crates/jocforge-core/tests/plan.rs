// pattern: Functional Core

use jocforge_core::{
    BlockPartition, EmdfProfile, JocCarriage, JocPlanError, JocProfile, resolve_encoding_plan,
    resolve_encoding_plan_with_partition,
};

#[test]
fn resolver_returns_a_complete_profile_carriage_rate_plan() {
    let plan = resolve_encoding_plan(
        JocProfile::Idx4,
        JocCarriage::EtsiAuxDependent,
        EmdfProfile::EtsiStrict,
        1664,
    )
    .expect("idx4 strict plan");

    assert_eq!(plan.profile, JocProfile::Idx4);
    assert_eq!(plan.carriage, JocCarriage::EtsiAuxDependent);
    assert_eq!(plan.emdf_profile, EmdfProfile::EtsiStrict);
    assert_eq!(plan.rate.total_bitrate_kbps, 1664);
    assert_eq!(plan.spec.downmix_index, 4);
    assert_eq!(plan.spec.input_labels.len(), 7);
    assert!(plan.spec.phase_signaled);
}

#[test]
fn resolver_rejects_skip_field_for_profiles_without_a_closed_topology() {
    let error = resolve_encoding_plan(
        JocProfile::Idx1,
        JocCarriage::I0SkipField,
        EmdfProfile::DeployedCompat,
        1024,
    )
    .expect_err("idx1 skip-field carriage must remain fail-closed");

    assert_eq!(
        error,
        JocPlanError::UnsupportedCarriage {
            profile: JocProfile::Idx1,
            carriage: JocCarriage::I0SkipField,
        }
    );
}

#[test]
fn resolver_keeps_the_actionable_total_bitrate_error_category() {
    let error = resolve_encoding_plan(
        JocProfile::Idx4,
        JocCarriage::EtsiAuxDependent,
        EmdfProfile::EtsiStrict,
        896,
    )
    .expect_err("paired I0/D0 capacity is below the clean minimum");

    assert!(error.to_string().contains("unsupported total bitrate"));
}

#[test]
fn resolver_admits_only_the_validated_legacy_ac3_flat7x_plan() {
    let plan = resolve_encoding_plan(
        JocProfile::Idx1,
        JocCarriage::LegacyAc3Dependent,
        EmdfProfile::EtsiStrict,
        1280,
    )
    .expect("legacy AC-3 Flat-7.X plan");
    assert_eq!(plan.rate.carrier_bitrate_kbps, 640);
    assert_eq!(plan.rate.d0_bitrate_kbps, 640);

    let phase_plan = resolve_encoding_plan(
        JocProfile::Idx4,
        JocCarriage::LegacyAc3Dependent,
        EmdfProfile::EtsiStrict,
        1280,
    )
    .expect("legacy AC-3 phase profile plan");
    assert_eq!(phase_plan.profile, JocProfile::Idx4);
}

#[test]
fn resolver_carries_a_short_partition_and_exact_sample_ranges() {
    let plan = resolve_encoding_plan_with_partition(
        JocProfile::Idx1,
        JocCarriage::EtsiAuxDependent,
        EmdfProfile::EtsiStrict,
        1664,
        &[BlockPartition::Three, BlockPartition::Three],
    )
    .expect("3+3 raw short-block plan");

    assert_eq!(
        plan.block_partition.partitions(),
        &[BlockPartition::Three, BlockPartition::Three]
    );
    assert_eq!(
        plan.block_partition.frame_ranges(9_216),
        vec![
            jocforge_core::ShortBlockFrameRange {
                syncframe_index: 0,
                start_sample: 9_216,
                end_sample: 9_984,
                audio_blocks: 3,
            },
            jocforge_core::ShortBlockFrameRange {
                syncframe_index: 1,
                start_sample: 9_984,
                end_sample: 10_752,
                audio_blocks: 3,
            },
        ]
    );
}

#[test]
fn resolver_rejects_short_partitions_on_non_raw_etsi_carriage() {
    let error = resolve_encoding_plan_with_partition(
        JocProfile::Idx0,
        JocCarriage::I0SkipField,
        EmdfProfile::DeployedCompat,
        1024,
        &[BlockPartition::One; 6],
    )
    .expect_err("skip-field short blocks must remain fail-closed");

    assert!(error.to_string().contains("short-block partition"));
}
