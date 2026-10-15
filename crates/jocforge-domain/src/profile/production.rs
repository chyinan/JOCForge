// pattern: Functional Core
//! Six-dimensional production profile. It has no validation render target.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthoredInputProfile {
    SelectedAdmBwf38ChannelPcm24At48Khz,
    Unsupported,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreCarrierProfile {
    SelectedFivePointOneSixCarrier,
    Unsupported,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JocDownmixProfile {
    SelectedMode0Phase1Index3,
    Unsupported,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubstreamTopology {
    SelectedSingleIndependentZeroDependent,
    Unsupported,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodecRateProfile {
    Selected48Khz1024Kbps6Blocks1536Samples4096Bytes,
    Unsupported,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmdfCarriageProfile {
    DeployedCompat,
    EtsiEmdf,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EncodingProfile {
    authored_input: AuthoredInputProfile,
    core_carrier: CoreCarrierProfile,
    joc_downmix: JocDownmixProfile,
    substream_topology: SubstreamTopology,
    codec_rate: CodecRateProfile,
    emdf_carriage: EmdfCarriageProfile,
}
impl EncodingProfile {
    pub const fn selected() -> Self {
        Self::selected_with_carriage(EmdfCarriageProfile::DeployedCompat)
    }

    pub const fn selected_with_carriage(emdf_carriage: EmdfCarriageProfile) -> Self {
        Self {
            authored_input: AuthoredInputProfile::SelectedAdmBwf38ChannelPcm24At48Khz,
            core_carrier: CoreCarrierProfile::SelectedFivePointOneSixCarrier,
            joc_downmix: JocDownmixProfile::SelectedMode0Phase1Index3,
            substream_topology: SubstreamTopology::SelectedSingleIndependentZeroDependent,
            codec_rate: CodecRateProfile::Selected48Khz1024Kbps6Blocks1536Samples4096Bytes,
            emdf_carriage,
        }
    }

    pub const fn new(
        authored_input: AuthoredInputProfile,
        core_carrier: CoreCarrierProfile,
        joc_downmix: JocDownmixProfile,
        substream_topology: SubstreamTopology,
        codec_rate: CodecRateProfile,
        emdf_carriage: EmdfCarriageProfile,
    ) -> Self {
        Self {
            authored_input,
            core_carrier,
            joc_downmix,
            substream_topology,
            codec_rate,
            emdf_carriage,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProfileError {
    UnsupportedProfile,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedEncodingProfile {
    carriage: EmdfCarriageProfile,
}

impl ResolvedEncodingProfile {
    pub const fn carriage(self) -> EmdfCarriageProfile {
        self.carriage
    }
}

/// Accepts exactly the approved six-dimensional production row.
pub const fn resolve_selected_profile(
    profile: &EncodingProfile,
) -> Result<ResolvedEncodingProfile, ProfileError> {
    if !matches!(
        profile.authored_input,
        AuthoredInputProfile::SelectedAdmBwf38ChannelPcm24At48Khz
    ) || !matches!(
        profile.core_carrier,
        CoreCarrierProfile::SelectedFivePointOneSixCarrier
    ) || !matches!(
        profile.joc_downmix,
        JocDownmixProfile::SelectedMode0Phase1Index3
    ) || !matches!(
        profile.substream_topology,
        SubstreamTopology::SelectedSingleIndependentZeroDependent
    ) || !matches!(
        profile.codec_rate,
        CodecRateProfile::Selected48Khz1024Kbps6Blocks1536Samples4096Bytes
    ) {
        return Err(ProfileError::UnsupportedProfile);
    }
    match profile.emdf_carriage {
        EmdfCarriageProfile::DeployedCompat | EmdfCarriageProfile::EtsiEmdf => {
            Ok(ResolvedEncodingProfile {
                carriage: profile.emdf_carriage,
            })
        }
        EmdfCarriageProfile::Unsupported => Err(ProfileError::UnsupportedProfile),
    }
}
