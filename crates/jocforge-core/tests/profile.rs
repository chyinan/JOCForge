// pattern: Functional Core

use jocforge_core::{EmdfProfile, JocCarriage, JocProfile, profile_spec};

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
