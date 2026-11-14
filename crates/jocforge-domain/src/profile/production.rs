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

/// Elementary carrier syntax, separate from the shared 5.1 core speaker layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CarrierCodecProfile {
    IndependentEac3,
    Flat7xAc3I0DependentEac3D0,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JocDownmixProfile {
    /// Selected mode/phase algorithm control profile retained by the current implementation.
    SelectedMode0Phase1Index3,
    Unsupported,
}

/// The ID14 payload's three-bit `downmix_index`, independent from its reconstruction-layout index.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JocPayloadDownmixIndex {
    FiveChannel,
    SevenChannel,
}

impl JocPayloadDownmixIndex {
    pub const fn value(self) -> u8 {
        match self {
            Self::FiveChannel => 3,
            Self::SevenChannel => 1,
        }
    }
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
    Flat7xIndependentI0DependentD0,
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
    carrier_codec: CarrierCodecProfile,
    joc_downmix: JocDownmixProfile,
    joc_payload_downmix_index: JocPayloadDownmixIndex,
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
            carrier_codec: CarrierCodecProfile::IndependentEac3,
            joc_downmix: JocDownmixProfile::SelectedMode0Phase1Index3,
            joc_payload_downmix_index: JocPayloadDownmixIndex::FiveChannel,
            joc_reconstruction_layout: JocReconstructionLayout::ExistingSelectedFivePointOne,
            substream_topology: SubstreamTopology::SelectedSingleIndependentZeroDependent,
            codec_rate,
            emdf_carriage,
        }
    }

    /// Selects the registered carrier-codec, substream, reconstruction, and payload-selector row.
    pub const fn selected_with_joc_reconstruction_layout(
        joc_reconstruction_layout: JocReconstructionLayout,
    ) -> Self {
        Self::selected_with_rate_and_carriage_and_joc_reconstruction_layout(
            CodecRateProfile::Kbps1024,
            EmdfCarriageProfile::DeployedCompat,
            joc_reconstruction_layout,
        )
    }

    pub const fn selected_with_codec_rate_and_joc_reconstruction_layout(
        codec_rate: CodecRateProfile,
        joc_reconstruction_layout: JocReconstructionLayout,
    ) -> Self {
        Self::selected_with_rate_and_carriage_and_joc_reconstruction_layout(
            codec_rate,
            EmdfCarriageProfile::DeployedCompat,
            joc_reconstruction_layout,
        )
    }

    pub const fn selected_with_rate_and_carriage_and_joc_reconstruction_layout(
        codec_rate: CodecRateProfile,
        emdf_carriage: EmdfCarriageProfile,
        joc_reconstruction_layout: JocReconstructionLayout,
    ) -> Self {
        let selected = Self::selected_with_rate_and_carriage(codec_rate, emdf_carriage);
        match joc_reconstruction_layout {
            JocReconstructionLayout::ExistingSelectedFivePointOne => Self {
                joc_reconstruction_layout,
                ..selected
            },
            JocReconstructionLayout::Flat7x => Self {
                carrier_codec: CarrierCodecProfile::Flat7xAc3I0DependentEac3D0,
                joc_payload_downmix_index: JocPayloadDownmixIndex::SevenChannel,
                joc_reconstruction_layout,
                substream_topology: SubstreamTopology::Flat7xIndependentI0DependentD0,
                ..selected
            },
            JocReconstructionLayout::Unsupported => Self {
                joc_reconstruction_layout,
                ..selected
            },
        }
    }

    /// Returns this immutable profile with a different reconstruction-layout selector only.
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

    /// Returns this immutable profile with a separately selected payload `downmix_index`.
    #[must_use]
    pub const fn with_joc_payload_downmix_index(
        self,
        joc_payload_downmix_index: JocPayloadDownmixIndex,
    ) -> Self {
        Self {
            joc_payload_downmix_index,
            ..self
        }
    }

    /// Returns the ID14 payload's coefficient-channel selector independently from reconstruction layout.
    pub const fn joc_payload_downmix_index(self) -> JocPayloadDownmixIndex {
        self.joc_payload_downmix_index
    }

    pub const fn carrier_codec(self) -> CarrierCodecProfile {
        self.carrier_codec
    }

    pub const fn substream_topology(self) -> SubstreamTopology {
        self.substream_topology
    }

    #[must_use]
    pub const fn with_carrier_codec(self, carrier_codec: CarrierCodecProfile) -> Self {
        Self {
            carrier_codec,
            ..self
        }
    }

    #[must_use]
    pub const fn with_substream_topology(self, substream_topology: SubstreamTopology) -> Self {
        Self {
            substream_topology,
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
            carrier_codec: CarrierCodecProfile::IndependentEac3,
            joc_downmix,
            joc_payload_downmix_index: JocPayloadDownmixIndex::FiveChannel,
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
            && self.carrier_codec == other.carrier_codec
            && self.joc_downmix == other.joc_downmix
            && self.joc_payload_downmix_index == other.joc_payload_downmix_index
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
    carrier_codec: CarrierCodecProfile,
    substream_topology: SubstreamTopology,
    joc_downmix: JocDownmixProfile,
    joc_payload_downmix_index: JocPayloadDownmixIndex,
    joc_reconstruction_layout: JocReconstructionLayout,
}

impl ResolvedEncodingProfile {
    pub const fn carriage(self) -> EmdfCarriageProfile {
        self.carriage
    }

    pub const fn codec_rate(self) -> CodecRateProfile {
        self.codec_rate
    }

    pub const fn carrier_codec(self) -> CarrierCodecProfile {
        self.carrier_codec
    }

    pub const fn substream_topology(self) -> SubstreamTopology {
        self.substream_topology
    }

    pub const fn joc_downmix(self) -> JocDownmixProfile {
        self.joc_downmix
    }

    pub const fn joc_payload_downmix_index(self) -> JocPayloadDownmixIndex {
        self.joc_payload_downmix_index
    }

    pub const fn joc_reconstruction_layout(self) -> JocReconstructionLayout {
        self.joc_reconstruction_layout
    }

    pub const fn joc_dmx_config_idx(self) -> Option<u8> {
        self.joc_reconstruction_layout.joc_dmx_config_idx()
    }
}

/// Accepts the existing selected row and the additive 1024 kbps Flat7x profile row.
pub const fn resolve_selected_profile(
    profile: &EncodingProfile,
) -> Result<ResolvedEncodingProfile, ProfileError> {
    let selected_codec_route = match (
        profile.joc_reconstruction_layout,
        profile.carrier_codec,
        profile.substream_topology,
        profile.joc_payload_downmix_index,
    ) {
        (
            JocReconstructionLayout::ExistingSelectedFivePointOne,
            CarrierCodecProfile::IndependentEac3,
            SubstreamTopology::SelectedSingleIndependentZeroDependent,
            JocPayloadDownmixIndex::FiveChannel,
        ) => true,
        (
            JocReconstructionLayout::Flat7x,
            CarrierCodecProfile::Flat7xAc3I0DependentEac3D0,
            SubstreamTopology::Flat7xIndependentI0DependentD0,
            JocPayloadDownmixIndex::SevenChannel,
        ) => matches!(profile.codec_rate, CodecRateProfile::Kbps1024),
        _ => false,
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
    ) || !selected_codec_route
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
                carrier_codec: profile.carrier_codec,
                substream_topology: profile.substream_topology,
                joc_downmix: profile.joc_downmix,
                joc_payload_downmix_index: profile.joc_payload_downmix_index,
                joc_reconstruction_layout: profile.joc_reconstruction_layout,
            })
        }
        EmdfCarriageProfile::Unsupported => Err(ProfileError::UnsupportedProfile),
    }
}
