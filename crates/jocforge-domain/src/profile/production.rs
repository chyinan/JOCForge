// pattern: Functional Core
//! Selected production-profile values. Validation render targets stay in a separate type.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthoredInputProfile {
    SelectedAdmBwfPcm24At48Khz,
    /// 38-channel spelling retained for callers that name that specific subset.
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

/// JOC decoder reconstruction layout, independent from internal downmix controls.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JocReconstructionLayout {
    /// Preserve the currently selected 5.1 header behavior.
    ExistingSelectedFivePointOne,
    /// ETSI 7.X Lb/Rb layout, serialized as `joc_dmx_config_idx=1`.
    Flat7x,
    Unsupported,
}

impl JocReconstructionLayout {
    /// Maps an explicitly supplied ETSI layout index to the one additive row supported here.
    pub const fn from_joc_dmx_config_idx(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Flat7x),
            _ => None,
        }
    }

    /// Returns an explicit wire index for additive layouts; the existing 5.1 row keeps its
    /// established writer input unchanged.
    pub const fn joc_dmx_config_idx(self) -> Option<u8> {
        match self {
            Self::Flat7x => Some(1),
            Self::ExistingSelectedFivePointOne | Self::Unsupported => None,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubstreamTopology {
    SelectedSingleIndependentZeroDependent,
    Unsupported,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedDataRate {
    Kbps384,
    Kbps448,
    Kbps576,
    Kbps640,
    Kbps768,
    Kbps1024,
    Unsupported,
}

/// The selected data rate is the codec-rate dimension of an encoding profile.
pub type CodecRateProfile = SelectedDataRate;

impl SelectedDataRate {
    pub const fn from_kbps(value: u32) -> Option<Self> {
        match value {
            384 => Some(Self::Kbps384),
            448 => Some(Self::Kbps448),
            576 => Some(Self::Kbps576),
            640 => Some(Self::Kbps640),
            768 => Some(Self::Kbps768),
            1_024 => Some(Self::Kbps1024),
            _ => None,
        }
    }

    /// Maps one explicitly authored, single-valued XML rate to its selected carrier row.
    pub fn from_xml_token(value: &str) -> Option<Self> {
        match value {
            "384" => Some(Self::Kbps384),
            "448" => Some(Self::Kbps448),
            "576" => Some(Self::Kbps576),
            "640" => Some(Self::Kbps640),
            "768" => Some(Self::Kbps768),
            "1024" => Some(Self::Kbps1024),
            _ => None,
        }
    }

    pub const fn kbps(self) -> Option<u32> {
        match self {
            Self::Kbps384 => Some(384),
            Self::Kbps448 => Some(448),
            Self::Kbps576 => Some(576),
            Self::Kbps640 => Some(640),
            Self::Kbps768 => Some(768),
            Self::Kbps1024 => Some(1_024),
            Self::Unsupported => None,
        }
    }

    /// Wash-bound setup selector code; this is not the E-AC-3 `frmsiz` field.
    pub const fn selector_code(self) -> Option<u8> {
        match self {
            Self::Kbps384 => Some(0),
            Self::Kbps448 => Some(1),
            Self::Kbps576 => Some(2),
            Self::Kbps640 => Some(3),
            Self::Kbps768 => Some(4),
            Self::Kbps1024 => Some(5),
            Self::Unsupported => None,
        }
    }

    /// E-AC-3 `frmsiz` code from the six-rate, 48 kHz observation set.
    pub const fn frame_size_code(self) -> Option<u16> {
        match self {
            Self::Kbps384 => Some(767),
            Self::Kbps448 => Some(895),
            Self::Kbps576 => Some(1_151),
            Self::Kbps640 => Some(1_279),
            Self::Kbps768 => Some(1_535),
            Self::Kbps1024 => Some(2_047),
            Self::Unsupported => None,
        }
    }

    pub const fn frame_bytes(self) -> Option<usize> {
        match self {
            Self::Kbps384 => Some(1_536),
            Self::Kbps448 => Some(1_792),
            Self::Kbps576 => Some(2_304),
            Self::Kbps640 => Some(2_560),
            Self::Kbps768 => Some(3_072),
            Self::Kbps1024 => Some(4_096),
            Self::Unsupported => None,
        }
    }

    pub const fn identity_label(self) -> &'static str {
        match self {
            Self::Kbps384 => "48khz_384kbps_6blocks_1536samples_1536bytes",
            Self::Kbps448 => "48khz_448kbps_6blocks_1536samples_1792bytes",
            Self::Kbps576 => "48khz_576kbps_6blocks_1536samples_2304bytes",
            Self::Kbps640 => "48khz_640kbps_6blocks_1536samples_2560bytes",
            Self::Kbps768 => "48khz_768kbps_6blocks_1536samples_3072bytes",
            Self::Kbps1024 => "48khz_1024kbps_6blocks_1536samples_4096bytes",
            Self::Unsupported => "unsupported",
        }
    }
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
    joc_reconstruction_layout: JocReconstructionLayout,
    substream_topology: SubstreamTopology,
    codec_rate: CodecRateProfile,
    emdf_carriage: EmdfCarriageProfile,
}
impl EncodingProfile {
    pub const fn selected() -> Self {
        Self::selected_with_carriage(EmdfCarriageProfile::DeployedCompat)
    }

    pub const fn selected_with_codec_rate(codec_rate: CodecRateProfile) -> Self {
        Self::selected_with_rate_and_carriage(codec_rate, EmdfCarriageProfile::DeployedCompat)
    }

    pub const fn selected_with_carriage(emdf_carriage: EmdfCarriageProfile) -> Self {
        Self::selected_with_rate_and_carriage(CodecRateProfile::Kbps1024, emdf_carriage)
    }

    pub const fn selected_with_rate_and_carriage(
        codec_rate: CodecRateProfile,
        emdf_carriage: EmdfCarriageProfile,
    ) -> Self {
        Self {
            authored_input: AuthoredInputProfile::SelectedAdmBwfPcm24At48Khz,
            core_carrier: CoreCarrierProfile::SelectedFivePointOneSixCarrier,
            joc_downmix: JocDownmixProfile::SelectedMode0Phase1Index3,
            joc_reconstruction_layout: JocReconstructionLayout::ExistingSelectedFivePointOne,
            substream_topology: SubstreamTopology::SelectedSingleIndependentZeroDependent,
            codec_rate,
            emdf_carriage,
        }
    }

    /// Selects one reconstruction-layout row while preserving the current 1024 kbps profile
    /// defaults for all other dimensions.
    pub const fn selected_with_joc_reconstruction_layout(
        joc_reconstruction_layout: JocReconstructionLayout,
    ) -> Self {
        Self::selected().with_joc_reconstruction_layout(joc_reconstruction_layout)
    }

    /// Returns this immutable profile with a different reconstruction-layout selection.
    #[must_use]
    pub const fn with_joc_reconstruction_layout(
        self,
        joc_reconstruction_layout: JocReconstructionLayout,
    ) -> Self {
        Self {
            joc_reconstruction_layout,
            ..self
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
            joc_reconstruction_layout: JocReconstructionLayout::ExistingSelectedFivePointOne,
            substream_topology,
            codec_rate,
            emdf_carriage,
        }
    }

    /// Compares the input, carrier layout, JOC mode, topology and carriage while leaving the
    /// independently selected E-AC-3 carrier rate out of the comparison.
    pub fn same_dimensions_except_codec_rate(self, other: Self) -> bool {
        self.authored_input == other.authored_input
            && self.core_carrier == other.core_carrier
            && self.joc_downmix == other.joc_downmix
            && self.joc_reconstruction_layout == other.joc_reconstruction_layout
            && self.substream_topology == other.substream_topology
            && self.emdf_carriage == other.emdf_carriage
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProfileError {
    UnsupportedProfile,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedEncodingProfile {
    carriage: EmdfCarriageProfile,
    codec_rate: CodecRateProfile,
    joc_downmix: JocDownmixProfile,
    joc_reconstruction_layout: JocReconstructionLayout,
}

impl ResolvedEncodingProfile {
    pub const fn carriage(self) -> EmdfCarriageProfile {
        self.carriage
    }

    pub const fn codec_rate(self) -> CodecRateProfile {
        self.codec_rate
    }

    pub const fn joc_downmix(self) -> JocDownmixProfile {
        self.joc_downmix
    }

    pub const fn joc_reconstruction_layout(self) -> JocReconstructionLayout {
        self.joc_reconstruction_layout
    }

    pub const fn joc_dmx_config_idx(self) -> Option<u8> {
        self.joc_reconstruction_layout.joc_dmx_config_idx()
    }
}

/// Accepts the existing selected rows and the additive 1024 kbps Flat7x layout row.
pub const fn resolve_selected_profile(
    profile: &EncodingProfile,
) -> Result<ResolvedEncodingProfile, ProfileError> {
    let reconstruction_layout_supported = match profile.joc_reconstruction_layout {
        JocReconstructionLayout::ExistingSelectedFivePointOne => true,
        JocReconstructionLayout::Flat7x => {
            matches!(profile.codec_rate, CodecRateProfile::Kbps1024)
        }
        JocReconstructionLayout::Unsupported => false,
    };
    if !matches!(
        profile.authored_input,
        AuthoredInputProfile::SelectedAdmBwfPcm24At48Khz
            | AuthoredInputProfile::SelectedAdmBwf38ChannelPcm24At48Khz
    ) || !matches!(
        profile.core_carrier,
        CoreCarrierProfile::SelectedFivePointOneSixCarrier
    ) || !matches!(
        profile.joc_downmix,
        JocDownmixProfile::SelectedMode0Phase1Index3
    ) || !reconstruction_layout_supported
        || !matches!(
            profile.substream_topology,
            SubstreamTopology::SelectedSingleIndependentZeroDependent
        )
        || profile.codec_rate.kbps().is_none()
        || profile.codec_rate.selector_code().is_none()
        || profile.codec_rate.frame_size_code().is_none()
        || profile.codec_rate.frame_bytes().is_none()
    {
        return Err(ProfileError::UnsupportedProfile);
    }
    match profile.emdf_carriage {
        EmdfCarriageProfile::DeployedCompat | EmdfCarriageProfile::EtsiEmdf => {
            Ok(ResolvedEncodingProfile {
                carriage: profile.emdf_carriage,
                codec_rate: profile.codec_rate,
                joc_downmix: profile.joc_downmix,
                joc_reconstruction_layout: profile.joc_reconstruction_layout,
            })
        }
        EmdfCarriageProfile::Unsupported => Err(ProfileError::UnsupportedProfile),
    }
}
