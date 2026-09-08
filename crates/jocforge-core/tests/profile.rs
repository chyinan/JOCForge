// pattern: Functional Core

use jocforge_core::{
    AdmObjectKind, AdmPosition, EmdfProfile, JocCarriage, JocProfile, profile_spec,
    semantic_slot_for_label, semantic_slot_for_position, semantic_slot_for_profile_label,
    semantic_slot_for_source_channel,
};

#[test]
fn mandatory_profiles_have_distinct_public_input_orders() {
    let flat = profile_spec(JocProfile::Idx1);
    let height = profile_spec(JocProfile::Idx2);

    assert_eq!(flat.downmix_index, 1);
    assert_eq!(flat.input_labels, ["L", "R", "C", "Ls", "Rs", "Lrs", "Rrs"]);
    assert_eq!(height.downmix_index, 2);
    assert_eq!(
        height.input_labels,
        ["L", "R", "C", "Ls", "Rs", "Tfl", "Tfr"]
    );
    assert_ne!(flat.input_labels, height.input_labels);
    assert_ne!(flat.channel_map, height.channel_map);
    assert_eq!(profile_spec(JocProfile::Idx0).d0_source_indices, [0, 1]);
    assert_eq!(flat.d0_source_indices, [5, 6]);
    assert_eq!(height.d0_source_indices, [5, 6]);
}

#[test]
fn phase_profiles_preserve_base_topology_without_extra_phase_dsp() {
    let phase_five = profile_spec(JocProfile::Idx3);
    let phase_height = profile_spec(JocProfile::Idx4);

    assert_eq!(phase_five.downmix_index, 3);
    assert_eq!(phase_five.input_labels, ["L", "R", "C", "Ls", "Rs"]);
    assert!(phase_five.phase_signaled);
    assert_eq!(phase_five.base_profile, JocProfile::Idx0);

    assert_eq!(phase_height.downmix_index, 4);
    assert_eq!(
        phase_height.input_labels,
        ["L", "R", "C", "Ls", "Rs", "Tfl", "Tfr"]
    );
    assert!(phase_height.phase_signaled);
    assert_eq!(phase_height.base_profile, JocProfile::Idx2);
    assert_eq!(phase_height.d0_source_indices, [5, 6]);
}

#[test]
fn strict_profiles_are_the_defaults_and_carriage_is_independent() {
    assert_eq!(EmdfProfile::default(), EmdfProfile::EtsiStrict);
    assert_eq!(JocCarriage::default(), JocCarriage::EtsiAuxDependent);
    assert_ne!(JocCarriage::EtsiAuxDependent, JocCarriage::I0SkipField);
    assert_ne!(EmdfProfile::EtsiStrict, EmdfProfile::DeployedCompat);
}

#[test]
fn audio_semantic_identity_gate_covers_flat7x_carriers_and_lfe_boundary() {
    let labels = [
        "RC_L", "RC_R", "RC_C", "RC_Lss", "RC_Rss", "RC_Lrs", "RC_Rrs",
    ];
    assert_eq!(
        labels.map(semantic_slot_for_label),
        [
            Some(0),
            Some(1),
            Some(2),
            Some(3),
            Some(4),
            Some(5),
            Some(6)
        ]
    );
    assert_eq!(semantic_slot_for_label("RC_LFE"), None);
    assert_eq!(
        [
            semantic_slot_for_position(jocforge_core::AdmPosition {
                azimuth: -1.0,
                elevation: -1.0,
                distance: 0.0,
            }),
            semantic_slot_for_position(jocforge_core::AdmPosition {
                azimuth: 0.0,
                elevation: -1.0,
                distance: 0.0,
            }),
            semantic_slot_for_position(jocforge_core::AdmPosition {
                azimuth: 1.0,
                elevation: 1.0,
                distance: 0.0,
            }),
        ],
        [Some(0), Some(2), Some(6)]
    );
}

#[test]
fn profile_semantic_slots_follow_profile_spec_and_fixed_channels_only() {
    let profiles = [
        (
            JocProfile::Idx0,
            [Some(0), Some(1), Some(2), Some(3), Some(4), None, None],
        ),
        (
            JocProfile::Idx1,
            [
                Some(0),
                Some(1),
                Some(2),
                Some(3),
                Some(4),
                Some(5),
                Some(6),
            ],
        ),
        (
            JocProfile::Idx2,
            [Some(0), Some(1), Some(2), Some(3), Some(4), None, None],
        ),
        (
            JocProfile::Idx3,
            [Some(0), Some(1), Some(2), Some(3), Some(4), None, None],
        ),
        (
            JocProfile::Idx4,
            [Some(0), Some(1), Some(2), Some(3), Some(4), None, None],
        ),
    ];
    let labels = ["L", "R", "C", "Ls", "Rs", "Lrs", "Rrs"];
    for (profile, expected) in profiles {
        assert_eq!(
            labels.map(|label| semantic_slot_for_profile_label(profile, label)),
            expected
        );
    }
    assert_eq!(
        semantic_slot_for_profile_label(JocProfile::Idx2, "Tfl"),
        Some(5)
    );
    assert_eq!(
        semantic_slot_for_profile_label(JocProfile::Idx0, "Tfl"),
        None
    );
    assert_eq!(
        semantic_slot_for_profile_label(JocProfile::Idx1, "Tfl"),
        None
    );
    assert_eq!(
        semantic_slot_for_profile_label(JocProfile::Idx2, "Lrs"),
        None
    );

    let centre = AdmPosition {
        azimuth: 0.0,
        elevation: -1.0,
        distance: 0.0,
    };
    assert_eq!(
        semantic_slot_for_source_channel(
            JocProfile::Idx1,
            AdmObjectKind::DirectSpeakers,
            Some("RC_C"),
            centre,
        ),
        Some(2)
    );
    assert_eq!(
        semantic_slot_for_source_channel(
            JocProfile::Idx1,
            AdmObjectKind::Objects,
            Some("RC_C"),
            centre,
        ),
        Some(2)
    );
}

#[test]
fn canonical_object_positions_can_seed_profile_carriers_but_arbitrary_objects_cannot() {
    let centre = AdmPosition {
        azimuth: 0.0,
        elevation: -1.0,
        distance: 0.0,
    };
    let arbitrary = AdmPosition {
        azimuth: 0.37,
        elevation: 0.12,
        distance: 1.0,
    };
    assert_eq!(
        semantic_slot_for_source_channel(JocProfile::Idx1, AdmObjectKind::Objects, None, centre),
        Some(2)
    );
    assert_eq!(
        semantic_slot_for_source_channel(JocProfile::Idx1, AdmObjectKind::Objects, None, arbitrary,),
        None
    );
    assert_eq!(
        semantic_slot_for_source_channel(
            JocProfile::Idx1,
            AdmObjectKind::DirectSpeakers,
            Some("Tfl"),
            AdmPosition {
                azimuth: -1.0,
                elevation: 1.0,
                distance: 0.0,
            },
        ),
        None
    );
    assert_eq!(
        semantic_slot_for_source_channel(
            JocProfile::Idx2,
            AdmObjectKind::DirectSpeakers,
            Some("Lrs"),
            AdmPosition {
                azimuth: -1.0,
                elevation: 1.0,
                distance: 0.0,
            },
        ),
        None
    );
}
