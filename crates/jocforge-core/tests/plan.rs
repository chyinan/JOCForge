// pattern: Functional Core

use jocforge_core::{
    BlockPartition, CapabilityVerdict, CompositionRequest, CompositionResolver, EmdfProfile,
    JocCarriage, JocContainer, JocPlanError, JocProfile, LfeOwnership, resolve_encoding_plan,
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
    let error = CompositionResolver::resolve(&CompositionRequest {
        profile: JocProfile::Idx1,
        container: JocContainer::Raw,
        carriage: JocCarriage::I0SkipField,
        emdf_profile: EmdfProfile::DeployedCompat,
        bitrate_kbps: 1024,
        partitions: vec![BlockPartition::Six],
        dependent_count: 0,
        lfe_ownership: LfeOwnership::Absent,
    })
    .expect_err("idx1 skip-field carriage must remain not applicable");

    assert_eq!(
        error.classification.verdict,
        CapabilityVerdict::NotApplicable
    );
}

#[test]
fn compatibility_wrapper_maps_not_applicable_skip_field_to_an_actionable_error() {
    let error = resolve_encoding_plan(
        JocProfile::Idx1,
        JocCarriage::I0SkipField,
        EmdfProfile::DeployedCompat,
        1024,
    )
    .expect_err("idx1 skip-field carriage must remain fail-closed");

    assert!(matches!(
        error,
        JocPlanError::UnsupportedBlockPartition {
            profile: JocProfile::Idx1,
            carriage: JocCarriage::I0SkipField,
            status: jocforge_core::BlockPartitionStatus::PublicLegal,
        }
    ));
}

#[test]
fn skip_field_profile_and_dependent_axes_are_classified_by_applicability() {
    for (profile, dependent_count) in [(JocProfile::Idx1, 0), (JocProfile::Idx2, 1)] {
        let error = CompositionResolver::resolve(&CompositionRequest {
            profile,
            container: JocContainer::Raw,
            carriage: JocCarriage::I0SkipField,
            emdf_profile: EmdfProfile::EtsiStrict,
            bitrate_kbps: 1024,
            partitions: vec![BlockPartition::Six],
            dependent_count,
            lfe_ownership: LfeOwnership::Absent,
        })
        .expect_err("skip-field has no seven-channel/dependent topology");
        assert_eq!(
            error.classification.verdict,
            CapabilityVerdict::NotApplicable
        );
    }
}

#[test]
fn phase_profile_reuses_the_five_channel_skip_field_carriage() {
    let plan = CompositionResolver::resolve(&CompositionRequest {
        profile: JocProfile::Idx3,
        container: JocContainer::Raw,
        carriage: JocCarriage::I0SkipField,
        emdf_profile: EmdfProfile::EtsiStrict,
        bitrate_kbps: 1024,
        partitions: vec![BlockPartition::Six],
        dependent_count: 0,
        lfe_ownership: LfeOwnership::Absent,
    })
    .expect("idx3 is a five-input phase-signaled I0-only profile");
    assert_eq!(plan.spec.input_labels.len(), 5);
    assert!(plan.spec.phase_signaled);
}

#[test]
fn skip_field_short_canonical_and_mixed_partitions_have_distinct_results() {
    let canonical = CompositionResolver::resolve(&CompositionRequest {
        profile: JocProfile::Idx0,
        container: JocContainer::Raw,
        carriage: JocCarriage::I0SkipField,
        emdf_profile: EmdfProfile::DeployedCompat,
        bitrate_kbps: 1024,
        partitions: vec![BlockPartition::Three, BlockPartition::Three],
        dependent_count: 0,
        lfe_ownership: LfeOwnership::Absent,
    })
    .expect("skip-field canonical short partition");
    assert_eq!(
        canonical.block_partition.partitions(),
        &[BlockPartition::Three, BlockPartition::Three]
    );

    let mixed = CompositionResolver::resolve(&CompositionRequest {
        partitions: vec![
            BlockPartition::One,
            BlockPartition::Two,
            BlockPartition::Three,
        ],
        ..CompositionRequest {
            profile: JocProfile::Idx0,
            container: JocContainer::Raw,
            carriage: JocCarriage::I0SkipField,
            emdf_profile: EmdfProfile::EtsiStrict,
            bitrate_kbps: 1024,
            partitions: vec![BlockPartition::Six],
            dependent_count: 0,
            lfe_ownership: LfeOwnership::Absent,
        }
    })
    .expect_err("heterogeneous skip-field ownership remains ambiguous");
    assert_eq!(
        mixed.classification.verdict,
        CapabilityVerdict::PublicAmbiguous
    );
}

#[test]
fn skip_field_and_legacy_plans_carry_the_remaining_public_axes() {
    let skip_lfe = CompositionResolver::resolve(&CompositionRequest {
        profile: JocProfile::Idx0,
        container: JocContainer::Raw,
        carriage: JocCarriage::I0SkipField,
        emdf_profile: EmdfProfile::EtsiStrict,
        bitrate_kbps: 1024,
        partitions: vec![BlockPartition::Six],
        dependent_count: 0,
        lfe_ownership: LfeOwnership::Independent,
    })
    .expect("skip-field independent LFE");
    assert_eq!(skip_lfe.lfe_ownership, LfeOwnership::Independent);

    let deployed_legacy = CompositionResolver::resolve(&CompositionRequest {
        profile: JocProfile::Idx4,
        container: JocContainer::Raw,
        carriage: JocCarriage::LegacyAc3Dependent,
        emdf_profile: EmdfProfile::DeployedCompat,
        bitrate_kbps: 1280,
        partitions: vec![BlockPartition::Six],
        dependent_count: 1,
        lfe_ownership: LfeOwnership::Absent,
    })
    .expect("legacy deployed EMDF");
    assert_eq!(deployed_legacy.emdf_profile, EmdfProfile::DeployedCompat);

    for lfe_ownership in [
        LfeOwnership::Independent,
        LfeOwnership::DependentSupplement,
        LfeOwnership::DependentReplacement,
    ] {
        CompositionResolver::resolve(&CompositionRequest {
            profile: JocProfile::Idx1,
            container: JocContainer::Raw,
            carriage: JocCarriage::LegacyAc3Dependent,
            emdf_profile: EmdfProfile::EtsiStrict,
            bitrate_kbps: 1280,
            partitions: vec![BlockPartition::Six],
            dependent_count: 1,
            lfe_ownership,
        })
        .expect("legacy LFE ownership plan");
    }

    let skip_capacity = CompositionResolver::resolve(&CompositionRequest {
        profile: JocProfile::Idx0,
        container: JocContainer::Raw,
        carriage: JocCarriage::I0SkipField,
        emdf_profile: EmdfProfile::EtsiStrict,
        bitrate_kbps: 1088,
        partitions: vec![BlockPartition::Three, BlockPartition::Three],
        dependent_count: 0,
        lfe_ownership: LfeOwnership::Absent,
    })
    .expect_err("skip-field rates above its 1024 kbps frame are capacity-bound");
    assert_eq!(
        skip_capacity.classification.verdict,
        CapabilityVerdict::RejectedCapacity
    );

    let legacy_capacity = CompositionResolver::resolve(&CompositionRequest {
        profile: JocProfile::Idx0,
        container: JocContainer::Raw,
        carriage: JocCarriage::LegacyAc3Dependent,
        emdf_profile: EmdfProfile::DeployedCompat,
        bitrate_kbps: 1216,
        partitions: vec![BlockPartition::Six],
        dependent_count: 1,
        lfe_ownership: LfeOwnership::Absent,
    })
    .expect_err("legacy idx0 lower rates remain capacity-bound");
    assert_eq!(
        legacy_capacity.classification.verdict,
        CapabilityVerdict::RejectedCapacity
    );
}

#[test]
fn authoritative_matrix_has_no_unimplemented_public_cells() {
    let rows = jocforge_core::composition_capability_matrix();
    let gaps = rows
        .iter()
        .filter(|row| row.classification.verdict == CapabilityVerdict::SpecLegalNotImplemented)
        .count();
    assert_eq!(gaps, 0, "authoritative matrix still contains {gaps} gaps");
}

#[test]
fn authoritative_matrix_preserves_the_zero_closure_verdict_counts() {
    let rows = jocforge_core::composition_capability_matrix();
    let count = |verdict| {
        rows.iter()
            .filter(|row| row.classification.verdict == verdict)
            .count()
    };
    assert_eq!(count(CapabilityVerdict::SupportedPass), 2393);
    assert_eq!(count(CapabilityVerdict::PublicAmbiguous), 2990);
    assert_eq!(count(CapabilityVerdict::NotApplicable), 110825);
    assert_eq!(count(CapabilityVerdict::SemanticCeiling), 6240);
    assert_eq!(count(CapabilityVerdict::RejectedCapacity), 10152);
    assert_eq!(count(CapabilityVerdict::Invalid), 7800);
    assert_eq!(count(CapabilityVerdict::Reserved), 0);
    assert_eq!(count(CapabilityVerdict::SpecLegalNotImplemented), 0);
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
fn central_resolver_keeps_single_dependent_strict_rates_on_the_closed_lattice() {
    let error = CompositionResolver::resolve(&CompositionRequest {
        profile: JocProfile::Idx2,
        container: JocContainer::Raw,
        carriage: JocCarriage::EtsiAuxDependent,
        emdf_profile: EmdfProfile::EtsiStrict,
        bitrate_kbps: 1088,
        partitions: vec![BlockPartition::Six],
        dependent_count: 1,
        lfe_ownership: LfeOwnership::Absent,
    })
    .expect_err("single dependent 1088 kbps is outside the closed strict lattice");
    assert_eq!(
        error.classification.verdict,
        CapabilityVerdict::RejectedCapacity
    );
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
fn resolver_admits_short_partitions_on_skip_field_carriage() {
    let plan = resolve_encoding_plan_with_partition(
        JocProfile::Idx0,
        JocCarriage::I0SkipField,
        EmdfProfile::DeployedCompat,
        1024,
        &[BlockPartition::One; 6],
    )
    .expect("skip-field short blocks");

    assert_eq!(plan.block_partition.audio_blocks(), 6);
}

#[test]
fn composition_resolver_carries_dependent_and_lfe_ownership_into_one_plan() {
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

    let plan = CompositionResolver::resolve(&request).expect("legal orthogonal plan");

    assert_eq!(plan.dependent_count, 2);
    assert_eq!(plan.lfe_ownership, LfeOwnership::DependentSupplement);
    assert_eq!(plan.block_partition.partitions(), &[BlockPartition::Two; 3]);
    let ownership = plan.ownership.as_ref().expect("ownership plan");
    assert_eq!(ownership.final_joc_owner, 1);
    assert!(ownership.dependents[0].has_lfe());
    assert!(!ownership.independent_lfe);
}

#[test]
fn composition_resolver_rejects_cmaf_short_and_dependent_extensions() {
    let request = CompositionRequest {
        profile: JocProfile::Idx2,
        container: JocContainer::Cmaf,
        carriage: JocCarriage::EtsiAuxDependent,
        emdf_profile: EmdfProfile::EtsiStrict,
        bitrate_kbps: 1664,
        partitions: vec![BlockPartition::Three, BlockPartition::Three],
        dependent_count: 1,
        lfe_ownership: LfeOwnership::Absent,
    };

    let error = CompositionResolver::resolve(&request).expect_err("CMAF short is closed");

    assert_eq!(
        error.classification.verdict,
        CapabilityVerdict::NotApplicable
    );
    assert!(error.classification.reason.contains("CMAF"));
}

#[test]
fn composition_resolver_distinguishes_semantic_ceiling_and_invalid_lfe() {
    let ceiling = CompositionRequest {
        profile: JocProfile::Idx0,
        container: JocContainer::Raw,
        carriage: JocCarriage::EtsiAuxDependent,
        emdf_profile: EmdfProfile::EtsiStrict,
        bitrate_kbps: 4864,
        partitions: vec![BlockPartition::Six],
        dependent_count: 6,
        lfe_ownership: LfeOwnership::Absent,
    };
    let ceiling_error = CompositionResolver::resolve(&ceiling).expect_err("idx0 D5 is ceiling");
    assert_eq!(
        ceiling_error.classification.verdict,
        CapabilityVerdict::SemanticCeiling
    );

    let invalid_lfe = CompositionRequest {
        dependent_count: 0,
        lfe_ownership: LfeOwnership::DependentSupplement,
        ..ceiling
    };
    let lfe_error = CompositionResolver::resolve(&invalid_lfe).expect_err("LFE needs a dependent");
    assert_eq!(lfe_error.classification.verdict, CapabilityVerdict::Invalid);
}

#[test]
fn composition_resolver_keeps_mixed_partitions_publicly_ambiguous() {
    let request = CompositionRequest {
        profile: JocProfile::Idx1,
        container: JocContainer::Raw,
        carriage: JocCarriage::EtsiAuxDependent,
        emdf_profile: EmdfProfile::EtsiStrict,
        bitrate_kbps: 1664,
        partitions: vec![
            BlockPartition::One,
            BlockPartition::Two,
            BlockPartition::Three,
        ],
        dependent_count: 1,
        lfe_ownership: LfeOwnership::Absent,
    };

    let error = CompositionResolver::resolve(&request).expect_err("mixed partition is ambiguous");

    assert_eq!(
        error.classification.verdict,
        CapabilityVerdict::PublicAmbiguous
    );
}

#[test]
fn composition_resolver_marks_legacy_short_as_not_applicable() {
    let request = CompositionRequest {
        profile: JocProfile::Idx1,
        container: JocContainer::Raw,
        carriage: JocCarriage::LegacyAc3Dependent,
        emdf_profile: EmdfProfile::EtsiStrict,
        bitrate_kbps: 1280,
        partitions: vec![BlockPartition::Three, BlockPartition::Three],
        dependent_count: 1,
        lfe_ownership: LfeOwnership::Absent,
    };

    let error = CompositionResolver::resolve(&request).expect_err("legacy short is not applicable");

    assert_eq!(
        error.classification.verdict,
        CapabilityVerdict::NotApplicable
    );
}

#[test]
fn container_specific_short_rules_precede_mixed_partition_ambiguity() {
    for carriage in [
        JocCarriage::EtsiAuxDependent,
        JocCarriage::LegacyAc3Dependent,
    ] {
        let error = CompositionResolver::resolve(&CompositionRequest {
            profile: JocProfile::Idx0,
            container: JocContainer::Cmaf,
            carriage,
            emdf_profile: EmdfProfile::EtsiStrict,
            bitrate_kbps: 1664,
            partitions: vec![
                BlockPartition::One,
                BlockPartition::Two,
                BlockPartition::Three,
            ],
            dependent_count: 1,
            lfe_ownership: LfeOwnership::Absent,
        })
        .expect_err("CMAF mixed partitions are not applicable");
        assert_eq!(
            error.classification.verdict,
            CapabilityVerdict::NotApplicable
        );
    }
}
