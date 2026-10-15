// pattern: Functional Core
use jocforge_domain::profile::{
    AuthoredInputProfile, CodecRateProfile, CoreCarrierProfile, EmdfCarriageProfile,
    EncodingProfile, JocDownmixProfile, ProfileError, RenderTarget, SubstreamTopology,
    ValidationProfile, resolve_selected_profile,
};

fn selected(carriage: EmdfCarriageProfile) -> EncodingProfile {
    EncodingProfile::new(
        AuthoredInputProfile::SelectedAdmBwf38ChannelPcm24At48Khz,
        CoreCarrierProfile::SelectedFivePointOneSixCarrier,
        JocDownmixProfile::SelectedMode0Phase1Index3,
        SubstreamTopology::SelectedSingleIndependentZeroDependent,
        CodecRateProfile::Selected48Khz1024Kbps6Blocks1536Samples4096Bytes,
        carriage,
    )
}

#[test]
fn both_registered_carriage_modes_accept_the_same_selected_row() {
    for mode in [
        EmdfCarriageProfile::DeployedCompat,
        EmdfCarriageProfile::EtsiEmdf,
    ] {
        assert!(resolve_selected_profile(&selected(mode)).is_ok());
    }
}

#[test]
fn each_production_dimension_rejects_independently() {
    let cases = [
        EncodingProfile::new(
            AuthoredInputProfile::Unsupported,
            CoreCarrierProfile::SelectedFivePointOneSixCarrier,
            JocDownmixProfile::SelectedMode0Phase1Index3,
            SubstreamTopology::SelectedSingleIndependentZeroDependent,
            CodecRateProfile::Selected48Khz1024Kbps6Blocks1536Samples4096Bytes,
            EmdfCarriageProfile::DeployedCompat,
        ),
        EncodingProfile::new(
            AuthoredInputProfile::SelectedAdmBwf38ChannelPcm24At48Khz,
            CoreCarrierProfile::Unsupported,
            JocDownmixProfile::SelectedMode0Phase1Index3,
            SubstreamTopology::SelectedSingleIndependentZeroDependent,
            CodecRateProfile::Selected48Khz1024Kbps6Blocks1536Samples4096Bytes,
            EmdfCarriageProfile::DeployedCompat,
        ),
        EncodingProfile::new(
            AuthoredInputProfile::SelectedAdmBwf38ChannelPcm24At48Khz,
            CoreCarrierProfile::SelectedFivePointOneSixCarrier,
            JocDownmixProfile::Unsupported,
            SubstreamTopology::SelectedSingleIndependentZeroDependent,
            CodecRateProfile::Selected48Khz1024Kbps6Blocks1536Samples4096Bytes,
            EmdfCarriageProfile::DeployedCompat,
        ),
        EncodingProfile::new(
            AuthoredInputProfile::SelectedAdmBwf38ChannelPcm24At48Khz,
            CoreCarrierProfile::SelectedFivePointOneSixCarrier,
            JocDownmixProfile::SelectedMode0Phase1Index3,
            SubstreamTopology::Unsupported,
            CodecRateProfile::Selected48Khz1024Kbps6Blocks1536Samples4096Bytes,
            EmdfCarriageProfile::DeployedCompat,
        ),
        EncodingProfile::new(
            AuthoredInputProfile::SelectedAdmBwf38ChannelPcm24At48Khz,
            CoreCarrierProfile::SelectedFivePointOneSixCarrier,
            JocDownmixProfile::SelectedMode0Phase1Index3,
            SubstreamTopology::SelectedSingleIndependentZeroDependent,
            CodecRateProfile::Unsupported,
            EmdfCarriageProfile::DeployedCompat,
        ),
        EncodingProfile::new(
            AuthoredInputProfile::SelectedAdmBwf38ChannelPcm24At48Khz,
            CoreCarrierProfile::SelectedFivePointOneSixCarrier,
            JocDownmixProfile::SelectedMode0Phase1Index3,
            SubstreamTopology::SelectedSingleIndependentZeroDependent,
            CodecRateProfile::Selected48Khz1024Kbps6Blocks1536Samples4096Bytes,
            EmdfCarriageProfile::Unsupported,
        ),
    ];
    for profile in cases {
        assert_eq!(
            resolve_selected_profile(&profile),
            Err(ProfileError::UnsupportedProfile)
        );
    }
}

#[test]
fn render_target_is_validation_only_and_does_not_change_resolution() {
    let profile = selected(EmdfCarriageProfile::DeployedCompat);
    let before = resolve_selected_profile(&profile);
    let open_joc = ValidationProfile::new(RenderTarget::OpenJoc714);
    let different = ValidationProfile::new(RenderTarget::Other);
    assert_ne!(open_joc, different);
    assert_eq!(before, resolve_selected_profile(&profile));
}

#[test]
fn unsupported_profile_error_is_stable_and_has_no_fallback() {
    let invalid = selected(EmdfCarriageProfile::Unsupported);
    assert_eq!(
        resolve_selected_profile(&invalid),
        Err(ProfileError::UnsupportedProfile)
    );
    assert_eq!(
        resolve_selected_profile(&invalid),
        Err(ProfileError::UnsupportedProfile)
    );
}

#[test]
fn selected_profile_defaults_to_deployed_compat_and_preserves_explicit_etsi() {
    let default = EncodingProfile::selected();
    let resolved_default = resolve_selected_profile(&default).unwrap();
    assert_eq!(
        resolved_default.carriage(),
        EmdfCarriageProfile::DeployedCompat
    );

    let etsi = EncodingProfile::selected_with_carriage(EmdfCarriageProfile::EtsiEmdf);
    let resolved_etsi = resolve_selected_profile(&etsi).unwrap();
    assert_eq!(resolved_etsi.carriage(), EmdfCarriageProfile::EtsiEmdf);
}
