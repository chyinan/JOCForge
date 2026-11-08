// pattern: Functional Core
//! Selected F01E job-XML normalization and production-profile admission.
//!
//! The parser consumes XML text and a validated WAVE descriptor only. It does
//! not open file references, parse ADM/CHNA graphs, decode DBMD, or create audio.

use crate::profile::{
    CodecRateProfile, EncodingProfile, ProfileError, ResolvedEncodingProfile,
    resolve_selected_profile,
};
use crate::timeline::AdmSampleCount;
use roxmltree::{Document, Node};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const SELECTED_SAMPLE_RATE_HZ: u32 = 48_000;
const SELECTED_RUNTIME_CHANNELS: u16 = 38;
const MAX_SELECTED_INPUT_CHANNELS: u16 = u8::MAX as u16 + 1;
const SELECTED_PCM_BITS: u16 = 24;
const SELECTED_LAYOUT_ENUM: u8 = 8;
const SELECTED_TARGET_CAPACITY: u8 = 15;
const SELECTED_SAMPLE_LENGTH_ENUM: u8 = 1;
const SELECTED_FRAME_SAMPLES: u16 = 1_536;
const SELECTED_SPATIAL_CONFIG_MODE: u8 = 0;
const SELECTED_DISPATCH_MODE: u8 = 2;
const SELECTED_ROUTE_VIRTUAL_PREFIX_COUNT: u8 = 11;
/// Exact lexical tokens for separate input-side and encoder-side timecode fields.
const ADMITTED_TIMECODE_FRAME_RATE_TOKENS: &[&str] = &[
    "not_indicated",
    "23.976",
    "24",
    "25",
    "29.97",
    "30",
    "48",
    "50",
    "59.94",
    "60",
];
const ADMITTED_TIME_BASE_TOKENS: &[&str] = &["file_position", "embedded_timecode"];

/// Validated WAVE descriptor supplied by the input provider.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedWaveDescriptor {
    sample_rate_hz: u32,
    channels: u16,
    pcm_bits: u16,
}

impl SelectedWaveDescriptor {
    pub const fn new(sample_rate_hz: u32, channels: u16, pcm_bits: u16) -> Self {
        Self {
            sample_rate_hz,
            channels,
            pcm_bits,
        }
    }

    pub const fn sample_rate_hz(self) -> u32 {
        self.sample_rate_hz
    }

    pub const fn channels(self) -> u16 {
        self.channels
    }

    pub const fn pcm_bits(self) -> u16 {
        self.pcm_bits
    }
}

/// Selected input branch from the supported job XML.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedInputKind {
    Adm,
    AtmosMezz,
}

impl SelectedInputKind {
    pub const fn selected_label(self) -> &'static str {
        match self {
            Self::Adm => "adm",
            Self::AtmosMezz => "atmos_mezz",
        }
    }

    /// Both authored XML branches reference an ADM BWF consumed by the same RIFF reader.
    /// The enum value remains distinct in normalized configuration identity.
    pub const fn is_adm_bwf(self) -> bool {
        matches!(self, Self::Adm | Self::AtmosMezz)
    }
}

/// One explicit XML-selected DRC profile name.
///
/// The enum preserves the authored value through admission. It does not itself select or
/// implement an audio effect or codec/BSI field; those consumers require their own binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedDrcProfile {
    FilmStandard,
    FilmLight,
    MusicStandard,
    MusicLight,
    Speech,
    None,
}

impl SelectedDrcProfile {
    pub fn from_xml_value(value: &str) -> Option<Self> {
        match value {
            "film_standard" => Some(Self::FilmStandard),
            "film_light" => Some(Self::FilmLight),
            "music_standard" => Some(Self::MusicStandard),
            "music_light" => Some(Self::MusicLight),
            "speech" => Some(Self::Speech),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub const fn xml_value(self) -> &'static str {
        match self {
            Self::FilmStandard => "film_standard",
            Self::FilmLight => "film_light",
            Self::MusicStandard => "music_standard",
            Self::MusicLight => "music_light",
            Self::Speech => "speech",
            Self::None => "none",
        }
    }
}

/// One exact XML choice for the authored 5.1 surround trim field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedSurroundTrim {
    Auto,
    ZeroDb,
    Minus3Db,
    Minus6Db,
    Minus9Db,
}

impl SelectedSurroundTrim {
    pub fn from_xml_value(value: &str) -> Option<Self> {
        match value {
            "auto" => Some(Self::Auto),
            "0" => Some(Self::ZeroDb),
            "-3" => Some(Self::Minus3Db),
            "-6" => Some(Self::Minus6Db),
            "-9" => Some(Self::Minus9Db),
            _ => None,
        }
    }

    pub const fn xml_value(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::ZeroDb => "0",
            Self::Minus3Db => "-3",
            Self::Minus6Db => "-6",
            Self::Minus9Db => "-9",
        }
    }
}

/// One exact XML choice for the authored 5.1 height trim field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedHeightTrim {
    Auto,
    Minus3Db,
    Minus6Db,
    Minus9Db,
    Minus12Db,
}

impl SelectedHeightTrim {
    pub fn from_xml_value(value: &str) -> Option<Self> {
        match value {
            "auto" => Some(Self::Auto),
            "-3" => Some(Self::Minus3Db),
            "-6" => Some(Self::Minus6Db),
            "-9" => Some(Self::Minus9Db),
            "-12" => Some(Self::Minus12Db),
            _ => None,
        }
    }

    pub const fn xml_value(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Minus3Db => "-3",
            Self::Minus6Db => "-6",
            Self::Minus9Db => "-9",
            Self::Minus12Db => "-12",
        }
    }
}

/// Sample count derived from one explicitly authored silence duration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedSilenceSampleCount(u64);

impl SelectedSilenceSampleCount {
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// File reference returned as a value. The parser never opens it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedFileReference {
    file_name: String,
    local_path: String,
}

impl SelectedFileReference {
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn local_path(&self) -> &str {
        &self.local_path
    }
}

/// File references extracted from one structurally supported external job XML.
///
/// Functional Core: this value contains authored references only; it performs no file I/O.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedJobFileReferences {
    input_kind: SelectedInputKind,
    input_reference: SelectedFileReference,
    output_reference: SelectedFileReference,
}

impl SelectedJobFileReferences {
    pub const fn input_kind(&self) -> SelectedInputKind {
        self.input_kind
    }

    pub fn input_reference(&self) -> &SelectedFileReference {
        &self.input_reference
    }

    pub fn output_reference(&self) -> &SelectedFileReference {
        &self.output_reference
    }
}

/// Selected measurement-only loudness configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedLoudnessConfiguration {
    metering_mode: String,
    dialogue_intelligence: bool,
    speech_threshold: u32,
}

impl SelectedLoudnessConfiguration {
    pub fn metering_mode(&self) -> &str {
        &self.metering_mode
    }

    pub const fn dialogue_intelligence(&self) -> bool {
        self.dialogue_intelligence
    }

    pub const fn speech_threshold(&self) -> u32 {
        self.speech_threshold
    }
}

/// Selected preferred downmix mode and Loro/LtRt mix level fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedDownmixConfiguration {
    preferred_mode: &'static str,
    loro_center_mix_level: i8,
    loro_surround_mix_level: i8,
    ltrt_center_mix_level: i8,
    ltrt_surround_mix_level: i8,
}

impl SelectedDownmixConfiguration {
    pub const fn preferred_mode(self) -> &'static str {
        self.preferred_mode
    }

    pub const fn mix_levels(self) -> [i8; 4] {
        [
            self.loro_center_mix_level,
            self.loro_surround_mix_level,
            self.ltrt_center_mix_level,
            self.ltrt_surround_mix_level,
        ]
    }
}

/// Normalized selected F01E job configuration before ADM graph/DBMD/event work.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedJobConfiguration {
    selected_profile: ResolvedEncodingProfile,
    input_kind: SelectedInputKind,
    input_reference: SelectedFileReference,
    output_reference: SelectedFileReference,
    wave: SelectedWaveDescriptor,
    source_samples: u64,
    bit_rate_kbps: u32,
    bit_rate_enum: u8,
    layout_enum: u8,
    target_capacity: u8,
    sample_length_enum: u8,
    frame_samples: u16,
    spatial_config_mode: u8,
    dispatch_mode: u8,
    route_virtual_prefix_count: u8,
    loudness: SelectedLoudnessConfiguration,
    line_drc_profile: SelectedDrcProfile,
    rf_drc_profile: SelectedDrcProfile,
    downmix: SelectedDownmixConfiguration,
    surround_trim_5_1: SelectedSurroundTrim,
    height_trim_5_1: SelectedHeightTrim,
    input_timecode_frame_rate: String,
    input_offset: String,
    input_ffoa: String,
    encode_timecode_frame_rate: String,
    start: String,
    end: String,
    time_base: String,
    prepend_silence_duration: String,
    append_silence_duration: String,
    prepend_silence_samples: SelectedSilenceSampleCount,
    append_silence_samples: SelectedSilenceSampleCount,
    append_silence_duration_explicit: bool,
}

impl SelectedJobConfiguration {
    /// Returns every normalized selected F01E value used by job/config identity.
    pub fn normalized_identity_value(&self) -> Value {
        let carriage = match self.selected_profile.carriage() {
            crate::profile::EmdfCarriageProfile::DeployedCompat => "deployed_compat",
            crate::profile::EmdfCarriageProfile::EtsiEmdf => "etsi_emdf",
            crate::profile::EmdfCarriageProfile::Unsupported => "unsupported",
        };
        let downmix = self.downmix;
        json!({
            "profile_dimensions": {
                "authored_input": "selected_adm_bwf_pcm24_48khz",
                "core_carrier": "selected_5_1_six_carriers",
                "joc_downmix": "mode0_phase1_index3",
                "substream_topology": "single_independent_zero_dependent",
                "codec_rate": self.selected_profile.codec_rate().identity_label(),
                "emdf_carriage": carriage
            },
            "input_kind": self.input_kind.selected_label(),
            "input_reference": {
                "file_name": self.input_reference.file_name(),
                "local_path": self.input_reference.local_path()
            },
            "output_reference": {
                "file_name": self.output_reference.file_name(),
                "local_path": self.output_reference.local_path()
            },
            "wave": {
                "sample_rate_hz": self.wave.sample_rate_hz(),
                "channels": self.wave.channels(),
                "pcm_bits": self.wave.pcm_bits()
            },
            "source_samples": self.source_samples,
            "bit_rate_kbps": self.bit_rate_kbps,
            "bit_rate_enum": self.bit_rate_enum,
            "layout_enum": self.layout_enum,
            "target_capacity": self.target_capacity,
            "sample_length_enum": self.sample_length_enum,
            "frame_samples": self.frame_samples,
            "spatial_config_mode": self.spatial_config_mode,
            "dispatch_mode": self.dispatch_mode,
            "route_virtual_prefix_count": self.route_virtual_prefix_count,
            "loudness": {
                "metering_mode": self.loudness.metering_mode(),
                "dialogue_intelligence": self.loudness.dialogue_intelligence(),
                "speech_threshold": self.loudness.speech_threshold()
            },
            "line_drc_profile": self.line_drc_profile.xml_value(),
            "rf_drc_profile": self.rf_drc_profile.xml_value(),
            "downmix": {
                "preferred_mode": downmix.preferred_mode(),
                "mix_levels": downmix.mix_levels()
            },
            "surround_trim_5_1": self.surround_trim_5_1.xml_value(),
            "height_trim_5_1": self.height_trim_5_1.xml_value(),
            "input_timecode_frame_rate": self.input_timecode_frame_rate,
            "input_offset": self.input_offset,
            "input_ffoa": self.input_ffoa,
            "encode_timecode_frame_rate": self.encode_timecode_frame_rate,
            "start": self.start,
            "end": self.end,
            "time_base": self.time_base,
            "prepend_silence_duration": self.prepend_silence_duration,
            "append_silence_duration": self.append_silence_duration,
            "prepend_silence_samples": self.prepend_silence_samples.value(),
            "append_silence_samples": self.append_silence_samples.value(),
            "append_silence_duration_explicit": self.append_silence_duration_explicit
        })
    }

    pub const fn selected_profile(&self) -> ResolvedEncodingProfile {
        self.selected_profile
    }

    pub const fn input_kind(&self) -> SelectedInputKind {
        self.input_kind
    }

    pub fn input_reference(&self) -> &SelectedFileReference {
        &self.input_reference
    }

    pub fn input_timecode_frame_rate(&self) -> &str {
        &self.input_timecode_frame_rate
    }

    pub fn output_reference(&self) -> &SelectedFileReference {
        &self.output_reference
    }

    pub const fn wave(&self) -> SelectedWaveDescriptor {
        self.wave
    }

    pub const fn source_samples(&self) -> u64 {
        self.source_samples
    }

    pub const fn bit_rate_kbps(&self) -> u32 {
        self.bit_rate_kbps
    }

    pub const fn bit_rate_enum(&self) -> u8 {
        self.bit_rate_enum
    }

    pub const fn layout_enum(&self) -> u8 {
        self.layout_enum
    }

    pub const fn target_capacity(&self) -> u8 {
        self.target_capacity
    }

    pub const fn sample_length_enum(&self) -> u8 {
        self.sample_length_enum
    }

    pub const fn frame_samples(&self) -> u16 {
        self.frame_samples
    }

    pub const fn spatial_config_mode(&self) -> u8 {
        self.spatial_config_mode
    }

    pub const fn dispatch_mode(&self) -> u8 {
        self.dispatch_mode
    }

    pub const fn route_virtual_prefix_count(&self) -> u8 {
        self.route_virtual_prefix_count
    }

    pub fn loudness(&self) -> &SelectedLoudnessConfiguration {
        &self.loudness
    }

    pub const fn line_drc_profile(&self) -> SelectedDrcProfile {
        self.line_drc_profile
    }

    pub const fn rf_drc_profile(&self) -> SelectedDrcProfile {
        self.rf_drc_profile
    }

    pub const fn downmix(&self) -> SelectedDownmixConfiguration {
        self.downmix
    }

    pub const fn surround_trim_5_1(&self) -> SelectedSurroundTrim {
        self.surround_trim_5_1
    }

    pub const fn height_trim_5_1(&self) -> SelectedHeightTrim {
        self.height_trim_5_1
    }

    pub fn input_offset(&self) -> &str {
        &self.input_offset
    }

    pub fn input_ffoa(&self) -> &str {
        &self.input_ffoa
    }

    pub fn start(&self) -> &str {
        &self.start
    }

    pub fn end(&self) -> &str {
        &self.end
    }

    pub fn time_base(&self) -> &str {
        &self.time_base
    }

    pub fn prepend_silence_duration(&self) -> &str {
        &self.prepend_silence_duration
    }

    pub fn append_silence_duration(&self) -> &str {
        &self.append_silence_duration
    }

    pub const fn prepend_silence_samples(&self) -> SelectedSilenceSampleCount {
        self.prepend_silence_samples
    }

    pub const fn append_silence_samples(&self) -> SelectedSilenceSampleCount {
        self.append_silence_samples
    }

    pub const fn append_silence_duration_explicit(&self) -> bool {
        self.append_silence_duration_explicit
    }

    /// Checks whether this fully parsed job shares the primary metadata/control source binding.
    /// It compares only source/profile fields consumed by the metadata producer, not output paths.
    pub fn verify_primary_control_binding(
        &self,
        binding: &SelectedPrimaryControlBinding,
    ) -> Result<(), PrimaryControlBindingError> {
        if self.selected_profile != binding.selected_profile
            || self.input_kind != binding.input_kind
            || self.input_reference.file_name != binding.source_file_name
            || self.wave != binding.wave
            || self.source_samples != binding.source_samples.value()
        {
            return Err(PrimaryControlBindingError::SelectedJobMismatch);
        }
        Ok(())
    }
}

/// Local selected job-configuration rejection. Codes are model API categories,
/// not recovered native error statuses.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JobConfigurationError {
    InvalidUtf8,
    ForbiddenXmlDeclaration,
    XmlSyntax,
    RootElement,
    DuplicateField,
    MissingField,
    InvalidInputReference,
    InvalidOutputReference,
    PcmProfile,
    SourceSampleCount,
    BitRate,
    TimeDomain,
    UnsupportedFieldValue,
    UnsupportedProfile,
}

/// Rejection from the strict external-XML reference boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JobXmlIngressError {
    Configuration(JobConfigurationError),
    UnsupportedElement(String),
    UnsupportedAttribute {
        path: String,
        name: String,
        value: String,
    },
    UnsupportedOutputContainer(String),
    UnsupportedContainerText(String),
    AmbiguousInputBranches,
}

impl std::fmt::Display for JobXmlIngressError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Configuration(error) => {
                write!(formatter, "job XML rejected ({})", error.code())
            }
            Self::UnsupportedElement(path) => {
                write!(formatter, "unsupported or unconsumed job XML element: {path}")
            }
            Self::UnsupportedAttribute { path, name, value } => {
                write!(formatter, "unsupported job XML attribute: {path}/@{name}={value}")
            }
            Self::UnsupportedOutputContainer(path) => write!(
                formatter,
                "unsupported output container at {path}; this CLI writer accepts only /job_config/output/ec3"
            ),
            Self::UnsupportedContainerText(path) => {
                write!(formatter, "unexpected text content in job XML container: {path}")
            }
            Self::AmbiguousInputBranches => formatter.write_str(
                "job XML must select exactly one input branch: /job_config/input/audio/adm or /job_config/input/audio/atmos_mezz",
            ),
        }
    }
}

impl std::error::Error for JobXmlIngressError {}

impl JobConfigurationError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidUtf8 | Self::XmlSyntax => "CONFIG_XML_PARSE",
            Self::ForbiddenXmlDeclaration => "CONFIG_XML_FORBIDDEN_DECLARATION",
            Self::RootElement => "CONFIG_ROOT",
            Self::DuplicateField => "CONFIG_DUPLICATE_FIELD",
            Self::MissingField => "CONFIG_MISSING_FIELD",
            Self::InvalidInputReference => "CONFIG_INPUT_REFERENCE",
            Self::InvalidOutputReference => "CONFIG_OUTPUT_REFERENCE",
            Self::PcmProfile => "CONFIG_PCM_PROFILE",
            Self::SourceSampleCount => "CONFIG_SOURCE_SAMPLE_COUNT",
            Self::BitRate => "CONFIG_BIT_RATE",
            Self::TimeDomain => "CONFIG_TIME_DOMAIN",
            Self::UnsupportedFieldValue => "CONFIG_FIELD_VALUE",
            Self::UnsupportedProfile => "UNSUPPORTED_PROFILE",
        }
    }
}

/// Primary-source job/config binding from the path-free R4 functional projection.
/// It is a metadata/control context, not the undistributed original job XML or a full output job.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedPrimaryControlBinding {
    selected_profile: ResolvedEncodingProfile,
    input_kind: SelectedInputKind,
    source_file_name: String,
    source_file_bytes: u64,
    data_bytes: u64,
    wave: SelectedWaveDescriptor,
    source_samples: AdmSampleCount,
    metadata_hashes: PrimaryMetadataChunkHashes,
    job_projection_sha256: [u8; 32],
    source_receipt_sha256: [u8; 32],
    original_job_sha256_declaration: [u8; 32],
    original_job_bytes_declaration: u64,
    original_job_bytes_available: bool,
    full_pcm_identity_rehashed: bool,
    runtime_resource_bound: bool,
}

impl SelectedPrimaryControlBinding {
    pub const fn selected_profile(&self) -> ResolvedEncodingProfile {
        self.selected_profile
    }

    pub const fn input_kind(&self) -> SelectedInputKind {
        self.input_kind
    }

    pub fn source_file_name(&self) -> &str {
        &self.source_file_name
    }

    pub const fn source_file_bytes(&self) -> u64 {
        self.source_file_bytes
    }

    pub const fn data_bytes(&self) -> u64 {
        self.data_bytes
    }

    pub const fn wave(&self) -> SelectedWaveDescriptor {
        self.wave
    }

    pub const fn source_samples(&self) -> AdmSampleCount {
        self.source_samples
    }

    pub const fn metadata_hashes(&self) -> PrimaryMetadataChunkHashes {
        self.metadata_hashes
    }

    pub const fn job_projection_sha256(&self) -> &[u8; 32] {
        &self.job_projection_sha256
    }

    pub const fn source_receipt_sha256(&self) -> &[u8; 32] {
        &self.source_receipt_sha256
    }

    /// This is the original job's declared digest; the original XML is not delivered.
    pub const fn original_job_sha256_declaration(&self) -> &[u8; 32] {
        &self.original_job_sha256_declaration
    }

    pub const fn original_job_bytes_declaration(&self) -> u64 {
        self.original_job_bytes_declaration
    }

    pub const fn original_job_bytes_available(&self) -> bool {
        self.original_job_bytes_available
    }

    pub const fn full_pcm_identity_rehashed(&self) -> bool {
        self.full_pcm_identity_rehashed
    }

    pub const fn runtime_resource_bound(&self) -> bool {
        self.runtime_resource_bound
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrimaryMetadataChunkHashes {
    axml: [u8; 32],
    chna: [u8; 32],
    dbmd: [u8; 32],
}

/// Resource, job, data-span and metadata identity for a freshly admitted job.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedPrimaryRuntimeIdentity {
    pub source_file_bytes: u64,
    pub data_bytes: u64,
    pub metadata_hashes: PrimaryMetadataChunkHashes,
    pub resource_sha256: [u8; 32],
    pub config_sha256: [u8; 32],
    pub job_xml_sha256: [u8; 32],
    pub job_xml_bytes: u64,
}

impl PrimaryMetadataChunkHashes {
    pub const fn new(axml: [u8; 32], chna: [u8; 32], dbmd: [u8; 32]) -> Self {
        Self { axml, chna, dbmd }
    }

    pub const fn axml(self) -> [u8; 32] {
        self.axml
    }

    pub const fn chna(self) -> [u8; 32] {
        self.chna
    }

    pub const fn dbmd(self) -> [u8; 32] {
        self.dbmd
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimaryControlBindingError {
    ArtifactIdentity,
    JsonSyntax,
    ProjectionReceipt,
    SourceReceipt,
    SourceExtent,
    SourceMetadataHash,
    SelectedProfile,
    SelectedFieldValue,
    SelectedJobMismatch,
}

impl PrimaryControlBindingError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ArtifactIdentity => "PRIMARY_BINDING_ARTIFACT_IDENTITY",
            Self::JsonSyntax => "PRIMARY_BINDING_JSON",
            Self::ProjectionReceipt => "PRIMARY_BINDING_PROJECTION_RECEIPT",
            Self::SourceReceipt => "PRIMARY_BINDING_SOURCE_RECEIPT",
            Self::SourceExtent => "PRIMARY_BINDING_SOURCE_EXTENT",
            Self::SourceMetadataHash => "PRIMARY_BINDING_METADATA_HASH",
            Self::SelectedProfile => "PRIMARY_BINDING_PROFILE",
            Self::SelectedFieldValue => "PRIMARY_BINDING_FIELD",
            Self::SelectedJobMismatch => "PRIMARY_BINDING_JOB_MISMATCH",
        }
    }
}

/// Validates the exact path-free R4 job/source projection. The caller must verify the
/// detached package envelope and manifest before providing these bytes. This function
/// does not claim that the undistributed original job XML or PCM bytes were rehashed.
pub fn bind_selected_primary_control_projection(
    job_projection_bytes: &[u8],
    projection_receipt_bytes: &[u8],
    frozen_job_binding_bytes: &[u8],
    source_receipt_bytes: &[u8],
) -> Result<SelectedPrimaryControlBinding, PrimaryControlBindingError> {
    const JOB_PROJECTION_SHA256: &str =
        "b710150f1b8357ca417d3fe705c88334b9c9bbd418656b458e49f8ffa3faba82";
    const PROJECTION_RECEIPT_SHA256: &str =
        "3c342e34dd5519f752a3f58cafcd0710e6dfb6137631baabf369a2221a08a2b8";
    const FROZEN_JOB_BINDING_SHA256: &str =
        "ad28d54ef72bf0c1af1f3315ccca74f27cd02de61f9006b25565acb49352361f";
    const SOURCE_RECEIPT_SHA256: &str =
        "c60c45033ee40592b9fc82c9c68edf94c4ee1676f27d9df34634291b2cb0f7ad";

    let job_projection_sha256 = sha256(job_projection_bytes);
    let projection_receipt_sha256 = sha256(projection_receipt_bytes);
    let frozen_binding_sha256 = sha256(frozen_job_binding_bytes);
    let source_receipt_sha256 = sha256(source_receipt_bytes);
    if hex_lower(&job_projection_sha256) != JOB_PROJECTION_SHA256
        || hex_lower(&projection_receipt_sha256) != PROJECTION_RECEIPT_SHA256
        || hex_lower(&frozen_binding_sha256) != FROZEN_JOB_BINDING_SHA256
        || hex_lower(&source_receipt_sha256) != SOURCE_RECEIPT_SHA256
    {
        return Err(PrimaryControlBindingError::ArtifactIdentity);
    }

    let job: Value = serde_json::from_slice(job_projection_bytes)
        .map_err(|_| PrimaryControlBindingError::JsonSyntax)?;
    let projection_receipt: Value = serde_json::from_slice(projection_receipt_bytes)
        .map_err(|_| PrimaryControlBindingError::JsonSyntax)?;
    let frozen_binding: Value = serde_json::from_slice(frozen_job_binding_bytes)
        .map_err(|_| PrimaryControlBindingError::JsonSyntax)?;
    let source: Value = serde_json::from_slice(source_receipt_bytes)
        .map_err(|_| PrimaryControlBindingError::JsonSyntax)?;

    let declared_job_sha = projection_receipt["original_job_sha256"]
        .as_str()
        .ok_or(PrimaryControlBindingError::ProjectionReceipt)?;
    if projection_receipt["projection_sha256"].as_str() != Some(JOB_PROJECTION_SHA256)
        || projection_receipt["primary_source_receipt_sha256"].as_str()
            != Some(SOURCE_RECEIPT_SHA256)
        || frozen_binding["primary_source_receipt_sha256"].as_str() != Some(SOURCE_RECEIPT_SHA256)
        || frozen_binding["job_sha256"].as_str() != Some(declared_job_sha)
        || projection_receipt["original_job_bytes"].as_u64() != frozen_binding["job_bytes"].as_u64()
        || frozen_binding["original_job_distributed"].as_bool() != Some(false)
        || frozen_binding["full_pcm_file_hash_recomputed"].as_bool() != Some(false)
    {
        return Err(PrimaryControlBindingError::ProjectionReceipt);
    }
    let codec_rate = validate_primary_job_fields(&job)?;

    let source_file_name = source["source_file_name"]
        .as_str()
        .filter(|value| !value.is_empty())
        .ok_or(PrimaryControlBindingError::SourceReceipt)?
        .to_owned();
    if job["input"]["file_name"].as_str() != Some(source_file_name.as_str())
        || frozen_binding["primary_source_file_name"].as_str() != Some(source_file_name.as_str())
        || frozen_binding["time_controls"] != job["encode"]["time_controls"]
    {
        return Err(PrimaryControlBindingError::SourceReceipt);
    }

    let sample_rate = value_u32(&source["sample_rate"])?;
    let channels = u16::try_from(value_u32(&source["physical_channels"])?)
        .map_err(|_| PrimaryControlBindingError::SourceExtent)?;
    let pcm_bits = u16::try_from(value_u32(&source["pcm_bits"])?)
        .map_err(|_| PrimaryControlBindingError::SourceExtent)?;
    let block_align = value_u32(&source["block_align"])?;
    let data_bytes = source["data_bytes"]
        .as_u64()
        .ok_or(PrimaryControlBindingError::SourceExtent)?;
    let source_file_bytes = source["source_file_bytes"]
        .as_u64()
        .ok_or(PrimaryControlBindingError::SourceExtent)?;
    let source_samples_value = source["source_samples"]
        .as_u64()
        .ok_or(PrimaryControlBindingError::SourceExtent)?;
    if sample_rate != SELECTED_SAMPLE_RATE_HZ
        || channels != SELECTED_RUNTIME_CHANNELS
        || pcm_bits != SELECTED_PCM_BITS
        || block_align != u32::from(channels) * u32::from(pcm_bits) / 8
        || data_bytes % u64::from(block_align) != 0
        || data_bytes / u64::from(block_align) != source_samples_value
        || data_bytes > source_file_bytes
        || source["metadata_only"].as_bool() != Some(true)
        || source["pcm_bytes_read"].as_u64() != Some(0)
        || source["pcm_content_hash_computed"].as_bool() != Some(false)
    {
        return Err(PrimaryControlBindingError::SourceExtent);
    }
    let source_samples = AdmSampleCount::new(source_samples_value)
        .map_err(|_| PrimaryControlBindingError::SourceExtent)?;

    let chunks = source["chunks"]
        .as_array()
        .ok_or(PrimaryControlBindingError::SourceMetadataHash)?;
    let chunk_hash = |id: &str| -> Result<[u8; 32], PrimaryControlBindingError> {
        let entry = chunks
            .iter()
            .find(|entry| entry["id"].as_str() == Some(id))
            .ok_or(PrimaryControlBindingError::SourceMetadataHash)?;
        parse_sha256(
            entry["sha256"]
                .as_str()
                .ok_or(PrimaryControlBindingError::SourceMetadataHash)?,
        )
    };
    let metadata_hashes = PrimaryMetadataChunkHashes {
        axml: chunk_hash("axml")?,
        chna: chunk_hash("chna")?,
        dbmd: chunk_hash("dbmd")?,
    };
    let selected_profile =
        resolve_selected_profile(&EncodingProfile::selected_with_codec_rate(codec_rate))
            .map_err(|_| PrimaryControlBindingError::SelectedProfile)?;

    Ok(SelectedPrimaryControlBinding {
        selected_profile,
        input_kind: SelectedInputKind::AtmosMezz,
        source_file_name,
        source_file_bytes,
        data_bytes,
        wave: SelectedWaveDescriptor::new(sample_rate, channels, pcm_bits),
        source_samples,
        metadata_hashes,
        job_projection_sha256,
        source_receipt_sha256,
        original_job_sha256_declaration: parse_sha256(declared_job_sha)?,
        original_job_bytes_declaration: projection_receipt["original_job_bytes"]
            .as_u64()
            .ok_or(PrimaryControlBindingError::ProjectionReceipt)?,
        original_job_bytes_available: false,
        full_pcm_identity_rehashed: false,
        runtime_resource_bound: false,
    })
}

/// Binds an admitted selected ADM job to its current complete RIFF resource for
/// independently constructed primary metadata/control values.
pub fn bind_admitted_selected_input(
    configuration: &SelectedJobConfiguration,
    identity: SelectedPrimaryRuntimeIdentity,
) -> Result<SelectedPrimaryControlBinding, PrimaryControlBindingError> {
    let selected_profile =
        resolve_selected_profile(&EncodingProfile::selected_with_rate_and_carriage(
            configuration.selected_profile().codec_rate(),
            configuration.selected_profile().carriage(),
        ))
        .map_err(|_| PrimaryControlBindingError::SelectedProfile)?;
    let wave = configuration.wave();
    let source_samples = AdmSampleCount::new(configuration.source_samples())
        .map_err(|_| PrimaryControlBindingError::SourceExtent)?;
    let expected_data_bytes = source_samples
        .value()
        .checked_mul(u64::from(wave.channels()) * u64::from(wave.pcm_bits()) / 8)
        .ok_or(PrimaryControlBindingError::SourceExtent)?;
    if configuration.selected_profile() != selected_profile
        || !configuration.input_kind().is_adm_bwf()
    {
        return Err(PrimaryControlBindingError::SelectedJobMismatch);
    }
    if wave.sample_rate_hz() != SELECTED_SAMPLE_RATE_HZ
        || wave.channels() == 0
        || wave.channels() > MAX_SELECTED_INPUT_CHANNELS
        || wave.pcm_bits() != SELECTED_PCM_BITS
        || identity.source_file_bytes == 0
        || identity.data_bytes != expected_data_bytes
        || identity.data_bytes > identity.source_file_bytes
        || identity.job_xml_bytes == 0
    {
        return Err(PrimaryControlBindingError::SourceExtent);
    }
    Ok(SelectedPrimaryControlBinding {
        selected_profile,
        input_kind: configuration.input_kind(),
        source_file_name: configuration.input_reference().file_name().to_owned(),
        source_file_bytes: identity.source_file_bytes,
        data_bytes: identity.data_bytes,
        wave,
        source_samples,
        metadata_hashes: identity.metadata_hashes,
        job_projection_sha256: identity.config_sha256,
        source_receipt_sha256: identity.resource_sha256,
        original_job_sha256_declaration: identity.job_xml_sha256,
        original_job_bytes_declaration: identity.job_xml_bytes,
        original_job_bytes_available: true,
        full_pcm_identity_rehashed: true,
        runtime_resource_bound: true,
    })
}

fn validate_primary_job_fields(
    job: &Value,
) -> Result<CodecRateProfile, PrimaryControlBindingError> {
    let expected_strings = [
        ("/schema", "selected-job-functional-field-projection.v1"),
        ("/input/kind", "atmos_mezz"),
        ("/input/version", "1"),
        ("/input/timecode_frame_rate", "not_indicated"),
        ("/input/offset", "auto"),
        ("/input/ffoa", "auto"),
        ("/encode/version", "1"),
        ("/encode/loudness/mode", "measure_only"),
        ("/encode/loudness/metering_mode", "1770-4"),
        ("/encode/time_controls/start", ""),
        ("/encode/time_controls/end", ""),
        ("/encode/time_controls/time_base", "file_position"),
        ("/encode/time_controls/prepend_silence_duration", "0.0"),
        ("/encode/time_controls/append_silence_duration", "0.0"),
        ("/encode/downmix/loro_center_mix_level", "-3"),
        ("/encode/downmix/loro_surround_mix_level", "-3"),
        ("/encode/downmix/ltrt_center_mix_level", "-3"),
        ("/encode/downmix/ltrt_surround_mix_level", "-3"),
        ("/encode/downmix/preferred_downmix_mode", "loro"),
        ("/encode/custom_trims/surround_trim_5_1", "auto"),
        ("/encode/custom_trims/height_trim_5_1", "auto"),
        ("/output/kind", "ec3"),
        ("/output/version", "1"),
    ];
    for (pointer, expected) in expected_strings {
        if job.pointer(pointer).and_then(Value::as_str) != Some(expected) {
            return Err(PrimaryControlBindingError::SelectedFieldValue);
        }
    }
    let encode_timecode_frame_rate = job
        .pointer("/encode/timecode_frame_rate")
        .and_then(Value::as_str)
        .ok_or(PrimaryControlBindingError::SelectedFieldValue)?;
    if !is_admitted_timecode_frame_rate_token(encode_timecode_frame_rate) {
        return Err(PrimaryControlBindingError::SelectedFieldValue);
    }
    let _dialogue_intelligence = job
        .pointer("/encode/loudness/dialogue_intelligence")
        .and_then(|value| {
            value.as_bool().or_else(|| match value.as_str() {
                Some("false") => Some(false),
                Some("true") => Some(true),
                _ => None,
            })
        })
        .ok_or(PrimaryControlBindingError::SelectedFieldValue)?;
    let speech_threshold = value_u32(&job["encode"]["loudness"]["speech_threshold"])?;
    let line_drc_profile = job
        .pointer("/encode/drc/line_mode_drc_profile")
        .and_then(Value::as_str)
        .ok_or(PrimaryControlBindingError::SelectedFieldValue)?;
    let rf_drc_profile = job
        .pointer("/encode/drc/rf_mode_drc_profile")
        .and_then(Value::as_str)
        .ok_or(PrimaryControlBindingError::SelectedFieldValue)?;
    let codec_rate = CodecRateProfile::from_kbps(value_u32(&job["encode"]["data_rate_kbps"])?)
        .ok_or(PrimaryControlBindingError::SelectedFieldValue)?;
    if SelectedDrcProfile::from_xml_value(line_drc_profile).is_none()
        || SelectedDrcProfile::from_xml_value(rf_drc_profile).is_none()
        || speech_threshold > 100
    {
        return Err(PrimaryControlBindingError::SelectedFieldValue);
    }
    Ok(codec_rate)
}

fn value_u32(value: &Value) -> Result<u32, PrimaryControlBindingError> {
    u32::try_from(
        value
            .as_u64()
            .ok_or(PrimaryControlBindingError::SelectedFieldValue)?,
    )
    .map_err(|_| PrimaryControlBindingError::SelectedFieldValue)
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn parse_sha256(value: &str) -> Result<[u8; 32], PrimaryControlBindingError> {
    if value.len() != 64 {
        return Err(PrimaryControlBindingError::ProjectionReceipt);
    }
    let mut result = [0_u8; 32];
    for (index, byte) in result.iter_mut().enumerate() {
        let start = index * 2;
        *byte = u8::from_str_radix(&value[start..start + 2], 16)
            .map_err(|_| PrimaryControlBindingError::ProjectionReceipt)?;
    }
    Ok(result)
}

fn hex_lower(bytes: &[u8; 32]) -> String {
    use std::fmt::Write as _;
    let mut text = String::with_capacity(64);
    for byte in bytes {
        write!(&mut text, "{byte:02x}").expect("writing to a String cannot fail");
    }
    text
}

/// Parses one selected XML job and admits its WAVE descriptor to the existing
/// six-dimensional production-profile resolver. File references remain values;
/// this function performs no file I/O.
pub fn parse_selected_job_configuration(
    xml_utf8: &[u8],
    wave: SelectedWaveDescriptor,
    source_samples: u64,
) -> Result<SelectedJobConfiguration, JobConfigurationError> {
    let xml = std::str::from_utf8(xml_utf8).map_err(|_| JobConfigurationError::InvalidUtf8)?;
    if contains_forbidden_declaration(xml.as_bytes()) {
        return Err(JobConfigurationError::ForbiddenXmlDeclaration);
    }
    let document = Document::parse(xml).map_err(|_| JobConfigurationError::XmlSyntax)?;
    let root = document.root_element();
    if root.tag_name().name() != "job_config" || root.tag_name().namespace().is_some() {
        return Err(JobConfigurationError::RootElement);
    }
    if source_samples == 0 {
        return Err(JobConfigurationError::SourceSampleCount);
    }
    let input_root =
        path_node(root, &["input", "audio"])?.ok_or(JobConfigurationError::MissingField)?;
    let (input_kind, input_node) = if let Some(adm) = single_child(input_root, "adm")? {
        (SelectedInputKind::Adm, adm)
    } else if let Some(mezz) = single_child(input_root, "atmos_mezz")? {
        (SelectedInputKind::AtmosMezz, mezz)
    } else {
        return Err(JobConfigurationError::MissingField);
    };
    let channel_count_supported = (1..=MAX_SELECTED_INPUT_CHANNELS).contains(&wave.channels);
    if wave.sample_rate_hz != SELECTED_SAMPLE_RATE_HZ
        || !channel_count_supported
        || wave.pcm_bits != SELECTED_PCM_BITS
    {
        return Err(JobConfigurationError::PcmProfile);
    }
    let input_reference =
        parse_file_reference(input_node, JobConfigurationError::InvalidInputReference)?;
    let input_timecode_frame_rate = required_text(input_node, &["timecode_frame_rate"])?;
    if !is_admitted_timecode_frame_rate_token(&input_timecode_frame_rate) {
        return Err(JobConfigurationError::TimeDomain);
    }
    let input_offset = optional_string_default_auto(input_node, &["offset"])?;
    let input_ffoa = optional_string_default_auto(input_node, &["ffoa"])?;

    let encoder = path_node(root, &["filter", "audio", "encode_to_atmos_ddp"])?
        .ok_or(JobConfigurationError::MissingField)?;
    let bit_rate = required_text(encoder, &["data_rate"])?;
    let codec_rate =
        CodecRateProfile::from_xml_token(&bit_rate).ok_or(JobConfigurationError::BitRate)?;
    let bit_rate_kbps = codec_rate.kbps().ok_or(JobConfigurationError::BitRate)?;
    let bit_rate_enum = codec_rate
        .selector_code()
        .ok_or(JobConfigurationError::BitRate)?;
    let encode_timecode_frame_rate = required_text(encoder, &["timecode_frame_rate"])?;
    if !is_admitted_timecode_frame_rate_token(&encode_timecode_frame_rate) {
        return Err(JobConfigurationError::TimeDomain);
    }

    let loudness_node = path_node(encoder, &["loudness", "measure_only"])?
        .ok_or(JobConfigurationError::MissingField)?;
    let dialogue_intelligence_text = required_text(loudness_node, &["dialogue_intelligence"])?;
    let dialogue_intelligence = match dialogue_intelligence_text.as_str() {
        "false" => false,
        "true" => true,
        _ => return Err(JobConfigurationError::UnsupportedFieldValue),
    };
    let speech_threshold =
        parse_decimal_u32(&required_text(loudness_node, &["speech_threshold"])?)?;
    if speech_threshold > 100 {
        return Err(JobConfigurationError::UnsupportedFieldValue);
    }
    let loudness = SelectedLoudnessConfiguration {
        metering_mode: require_exact(loudness_node, &["metering_mode"], "1770-4")?,
        dialogue_intelligence,
        speech_threshold,
    };

    let data_rate = path_node(encoder, &["drc"])?.ok_or(JobConfigurationError::MissingField)?;
    let line_drc_profile_value = required_text(data_rate, &["line_mode_drc_profile"])?;
    let line_drc_profile = SelectedDrcProfile::from_xml_value(&line_drc_profile_value)
        .ok_or(JobConfigurationError::UnsupportedFieldValue)?;
    let rf_drc_profile_value = required_text(data_rate, &["rf_mode_drc_profile"])?;
    let rf_drc_profile = SelectedDrcProfile::from_xml_value(&rf_drc_profile_value)
        .ok_or(JobConfigurationError::UnsupportedFieldValue)?;
    let downmix_node =
        path_node(encoder, &["downmix"])?.ok_or(JobConfigurationError::MissingField)?;
    let preferred_mode_value = required_text(downmix_node, &["preferred_downmix_mode"])?;
    let preferred_mode = match preferred_mode_value.as_str() {
        "loro" => "loro",
        "ltrt" => "ltrt",
        "ltrt-pl2" => "ltrt-pl2",
        "not_indicated" => "not_indicated",
        _ => return Err(JobConfigurationError::UnsupportedFieldValue),
    };
    let downmix = SelectedDownmixConfiguration {
        preferred_mode,
        loro_center_mix_level: parse_selected_mix_level(&required_text(
            downmix_node,
            &["loro_center_mix_level"],
        )?)?,
        loro_surround_mix_level: parse_selected_mix_level(&required_text(
            downmix_node,
            &["loro_surround_mix_level"],
        )?)?,
        ltrt_center_mix_level: parse_selected_mix_level(&required_text(
            downmix_node,
            &["ltrt_center_mix_level"],
        )?)?,
        ltrt_surround_mix_level: parse_selected_mix_level(&required_text(
            downmix_node,
            &["ltrt_surround_mix_level"],
        )?)?,
    };

    let trims =
        path_node(encoder, &["custom_trims"])?.ok_or(JobConfigurationError::MissingField)?;
    let surround_trim_value = required_text(trims, &["surround_trim_5_1"])?;
    let surround_trim_5_1 = SelectedSurroundTrim::from_xml_value(&surround_trim_value)
        .ok_or(JobConfigurationError::UnsupportedFieldValue)?;
    let height_trim_value = required_text(trims, &["height_trim_5_1"])?;
    let height_trim_5_1 = SelectedHeightTrim::from_xml_value(&height_trim_value)
        .ok_or(JobConfigurationError::UnsupportedFieldValue)?;

    let start = optional_authored_time_string(encoder, "start", "first_frame_of_action")?;
    let end = optional_authored_time_string(encoder, "end", "end_of_file")?;
    let time_base = required_text(encoder, &["time_base"])?;
    if !is_admitted_time_base_token(&time_base) {
        return Err(JobConfigurationError::TimeDomain);
    }
    let (prepend_silence_duration, prepend_silence_samples) =
        required_silence_duration(encoder, "prepend_silence_duration", wave.sample_rate_hz())?;
    let (append_silence_duration, append_silence_samples, append_silence_duration_explicit) =
        optional_silence_duration(encoder, "append_silence_duration", wave.sample_rate_hz())?;

    let output_node =
        path_node(root, &["output", "ec3"])?.ok_or(JobConfigurationError::MissingField)?;
    let output_reference =
        parse_file_reference(output_node, JobConfigurationError::InvalidOutputReference)?;
    if output_reference.file_name.is_empty() || output_reference.local_path.is_empty() {
        return Err(JobConfigurationError::InvalidOutputReference);
    }

    let selected_profile = resolve_selected_profile(&EncodingProfile::selected_with_codec_rate(
        codec_rate,
    ))
    .map_err(|ProfileError::UnsupportedProfile| JobConfigurationError::UnsupportedProfile)?;

    Ok(SelectedJobConfiguration {
        selected_profile,
        input_kind,
        input_reference,
        output_reference,
        wave,
        source_samples,
        bit_rate_kbps,
        bit_rate_enum,
        layout_enum: SELECTED_LAYOUT_ENUM,
        target_capacity: SELECTED_TARGET_CAPACITY,
        sample_length_enum: SELECTED_SAMPLE_LENGTH_ENUM,
        frame_samples: SELECTED_FRAME_SAMPLES,
        spatial_config_mode: SELECTED_SPATIAL_CONFIG_MODE,
        dispatch_mode: SELECTED_DISPATCH_MODE,
        route_virtual_prefix_count: SELECTED_ROUTE_VIRTUAL_PREFIX_COUNT,
        loudness,
        line_drc_profile,
        rf_drc_profile,
        downmix,
        surround_trim_5_1,
        height_trim_5_1,
        input_timecode_frame_rate,
        input_offset,
        input_ffoa,
        encode_timecode_frame_rate,
        start,
        end,
        time_base,
        prepend_silence_duration,
        append_silence_duration,
        prepend_silence_samples,
        append_silence_samples,
        append_silence_duration_explicit,
    })
}

/// Extracts file references from the XML surface admitted by the current CLI ingress.
///
/// This strict reference pass rejects unknown nodes and attributes before the CLI opens the
/// referenced media. It does not validate WAVE metadata or create a prepared job input; callers
/// must still pass the original bytes to `parse_selected_job_configuration` after reading the WAVE.
pub fn parse_selected_job_file_references(
    xml_utf8: &[u8],
) -> Result<SelectedJobFileReferences, JobXmlIngressError> {
    let xml = std::str::from_utf8(xml_utf8)
        .map_err(|_| JobXmlIngressError::Configuration(JobConfigurationError::InvalidUtf8))?;
    if contains_forbidden_declaration(xml.as_bytes()) {
        return Err(JobXmlIngressError::Configuration(
            JobConfigurationError::ForbiddenXmlDeclaration,
        ));
    }
    let document = Document::parse(xml)
        .map_err(|_| JobXmlIngressError::Configuration(JobConfigurationError::XmlSyntax))?;
    let root = document.root_element();
    if root.tag_name().name() != "job_config" || root.tag_name().namespace().is_some() {
        return Err(JobXmlIngressError::Configuration(
            JobConfigurationError::RootElement,
        ));
    }
    validate_external_job_xml_node(root, "/job_config")?;

    let input_root = path_node(root, &["input", "audio"])
        .map_err(JobXmlIngressError::Configuration)?
        .ok_or(JobXmlIngressError::Configuration(
            JobConfigurationError::MissingField,
        ))?;
    let adm = single_child(input_root, "adm").map_err(JobXmlIngressError::Configuration)?;
    let atmos_mezz =
        single_child(input_root, "atmos_mezz").map_err(JobXmlIngressError::Configuration)?;
    let (input_kind, input_node) = match (adm, atmos_mezz) {
        (Some(adm), None) => (SelectedInputKind::Adm, adm),
        (None, Some(atmos_mezz)) => (SelectedInputKind::AtmosMezz, atmos_mezz),
        (Some(_), Some(_)) | (None, None) => {
            return Err(JobXmlIngressError::AmbiguousInputBranches);
        }
    };
    let input_reference =
        parse_file_reference(input_node, JobConfigurationError::InvalidInputReference)
            .map_err(JobXmlIngressError::Configuration)?;

    let output_node = path_node(root, &["output", "ec3"])
        .map_err(JobXmlIngressError::Configuration)?
        .ok_or(JobXmlIngressError::Configuration(
            JobConfigurationError::MissingField,
        ))?;
    let output_reference =
        parse_file_reference(output_node, JobConfigurationError::InvalidOutputReference)
            .map_err(JobXmlIngressError::Configuration)?;
    if output_reference.file_name.is_empty() || output_reference.local_path.is_empty() {
        return Err(JobXmlIngressError::Configuration(
            JobConfigurationError::InvalidOutputReference,
        ));
    }

    Ok(SelectedJobFileReferences {
        input_kind,
        input_reference,
        output_reference,
    })
}

fn validate_external_job_xml_node(
    node: Node<'_, '_>,
    path: &str,
) -> Result<(), JobXmlIngressError> {
    if matches!(path, "/job_config/output/mp4" | "/job_config/output/dash") {
        return Err(JobXmlIngressError::UnsupportedOutputContainer(
            path.to_owned(),
        ));
    }
    if node.tag_name().namespace().is_some() || !external_job_xml_element_is_supported(path) {
        return Err(JobXmlIngressError::UnsupportedElement(path.to_owned()));
    }
    for attribute in node.attributes() {
        let allowed_version = attribute.namespace().is_none()
            && attribute.name() == "version"
            && attribute.value() == "1"
            && matches!(
                path,
                "/job_config/input/audio/adm"
                    | "/job_config/input/audio/atmos_mezz"
                    | "/job_config/filter/audio/encode_to_atmos_ddp"
                    | "/job_config/output/ec3"
            );
        if !allowed_version {
            return Err(JobXmlIngressError::UnsupportedAttribute {
                path: path.to_owned(),
                name: attribute.name().to_owned(),
                value: attribute.value().to_owned(),
            });
        }
    }

    let is_container = external_job_xml_container_is_supported(path);
    for child in node.children() {
        if child.is_element() {
            let child_path = format!("{path}/{}", child.tag_name().name());
            validate_external_job_xml_node(child, &child_path)?;
        } else if child.is_text()
            && is_container
            && child.text().is_some_and(|text| !text.trim().is_empty())
        {
            return Err(JobXmlIngressError::UnsupportedContainerText(
                path.to_owned(),
            ));
        }
    }
    Ok(())
}

fn external_job_xml_element_is_supported(path: &str) -> bool {
    matches!(
        path,
        "/job_config"
            | "/job_config/input"
            | "/job_config/input/audio"
            | "/job_config/input/audio/adm"
            | "/job_config/input/audio/adm/file_name"
            | "/job_config/input/audio/adm/storage"
            | "/job_config/input/audio/adm/storage/local"
            | "/job_config/input/audio/adm/storage/local/path"
            | "/job_config/input/audio/adm/timecode_frame_rate"
            | "/job_config/input/audio/adm/offset"
            | "/job_config/input/audio/adm/ffoa"
            | "/job_config/input/audio/atmos_mezz"
            | "/job_config/input/audio/atmos_mezz/file_name"
            | "/job_config/input/audio/atmos_mezz/storage"
            | "/job_config/input/audio/atmos_mezz/storage/local"
            | "/job_config/input/audio/atmos_mezz/storage/local/path"
            | "/job_config/input/audio/atmos_mezz/timecode_frame_rate"
            | "/job_config/input/audio/atmos_mezz/offset"
            | "/job_config/input/audio/atmos_mezz/ffoa"
            | "/job_config/filter"
            | "/job_config/filter/audio"
            | "/job_config/filter/audio/encode_to_atmos_ddp"
            | "/job_config/filter/audio/encode_to_atmos_ddp/loudness"
            | "/job_config/filter/audio/encode_to_atmos_ddp/loudness/measure_only"
            | "/job_config/filter/audio/encode_to_atmos_ddp/loudness/measure_only/metering_mode"
            | "/job_config/filter/audio/encode_to_atmos_ddp/loudness/measure_only/dialogue_intelligence"
            | "/job_config/filter/audio/encode_to_atmos_ddp/loudness/measure_only/speech_threshold"
            | "/job_config/filter/audio/encode_to_atmos_ddp/data_rate"
            | "/job_config/filter/audio/encode_to_atmos_ddp/timecode_frame_rate"
            | "/job_config/filter/audio/encode_to_atmos_ddp/start"
            | "/job_config/filter/audio/encode_to_atmos_ddp/end"
            | "/job_config/filter/audio/encode_to_atmos_ddp/time_base"
            | "/job_config/filter/audio/encode_to_atmos_ddp/prepend_silence_duration"
            | "/job_config/filter/audio/encode_to_atmos_ddp/append_silence_duration"
            | "/job_config/filter/audio/encode_to_atmos_ddp/drc"
            | "/job_config/filter/audio/encode_to_atmos_ddp/drc/line_mode_drc_profile"
            | "/job_config/filter/audio/encode_to_atmos_ddp/drc/rf_mode_drc_profile"
            | "/job_config/filter/audio/encode_to_atmos_ddp/downmix"
            | "/job_config/filter/audio/encode_to_atmos_ddp/downmix/loro_center_mix_level"
            | "/job_config/filter/audio/encode_to_atmos_ddp/downmix/loro_surround_mix_level"
            | "/job_config/filter/audio/encode_to_atmos_ddp/downmix/ltrt_center_mix_level"
            | "/job_config/filter/audio/encode_to_atmos_ddp/downmix/ltrt_surround_mix_level"
            | "/job_config/filter/audio/encode_to_atmos_ddp/downmix/preferred_downmix_mode"
            | "/job_config/filter/audio/encode_to_atmos_ddp/custom_trims"
            | "/job_config/filter/audio/encode_to_atmos_ddp/custom_trims/surround_trim_5_1"
            | "/job_config/filter/audio/encode_to_atmos_ddp/custom_trims/height_trim_5_1"
            | "/job_config/output"
            | "/job_config/output/ec3"
            | "/job_config/output/ec3/file_name"
            | "/job_config/output/ec3/storage"
            | "/job_config/output/ec3/storage/local"
            | "/job_config/output/ec3/storage/local/path"
    )
}

fn external_job_xml_container_is_supported(path: &str) -> bool {
    matches!(
        path,
        "/job_config"
            | "/job_config/input"
            | "/job_config/input/audio"
            | "/job_config/input/audio/adm"
            | "/job_config/input/audio/adm/storage"
            | "/job_config/input/audio/adm/storage/local"
            | "/job_config/input/audio/atmos_mezz"
            | "/job_config/input/audio/atmos_mezz/storage"
            | "/job_config/input/audio/atmos_mezz/storage/local"
            | "/job_config/filter"
            | "/job_config/filter/audio"
            | "/job_config/filter/audio/encode_to_atmos_ddp"
            | "/job_config/filter/audio/encode_to_atmos_ddp/loudness"
            | "/job_config/filter/audio/encode_to_atmos_ddp/loudness/measure_only"
            | "/job_config/filter/audio/encode_to_atmos_ddp/drc"
            | "/job_config/filter/audio/encode_to_atmos_ddp/downmix"
            | "/job_config/filter/audio/encode_to_atmos_ddp/custom_trims"
            | "/job_config/output"
            | "/job_config/output/ec3"
            | "/job_config/output/ec3/storage"
            | "/job_config/output/ec3/storage/local"
    )
}

fn contains_forbidden_declaration(xml_bytes: &[u8]) -> bool {
    [b"<!DOCTYPE".as_slice(), b"<!ENTITY".as_slice()]
        .iter()
        .any(|needle| {
            xml_bytes
                .windows(needle.len())
                .any(|window| window.eq_ignore_ascii_case(needle))
        })
}

fn single_child<'a, 'input>(
    parent: Node<'a, 'input>,
    local_name: &str,
) -> Result<Option<Node<'a, 'input>>, JobConfigurationError> {
    let mut matches = parent.children().filter(|child| {
        child.is_element()
            && child.tag_name().namespace().is_none()
            && child.tag_name().name() == local_name
    });
    let first = matches.next();
    if matches.next().is_some() {
        return Err(JobConfigurationError::DuplicateField);
    }
    Ok(first)
}

fn path_node<'a, 'input>(
    root: Node<'a, 'input>,
    path: &[&str],
) -> Result<Option<Node<'a, 'input>>, JobConfigurationError> {
    let mut current = Some(root);
    for component in path {
        current = match current {
            Some(parent) => single_child(parent, component)?,
            None => return Ok(None),
        };
    }
    Ok(current)
}

fn path_text<'a>(
    root: Node<'a, '_>,
    path: &[&str],
) -> Result<Option<&'a str>, JobConfigurationError> {
    Ok(path_node(root, path)?.and_then(|node| node.text()))
}

fn required_text(root: Node<'_, '_>, path: &[&str]) -> Result<String, JobConfigurationError> {
    path_text(root, path)?
        .map(ToOwned::to_owned)
        .ok_or(JobConfigurationError::MissingField)
}

fn is_admitted_timecode_frame_rate_token(value: &str) -> bool {
    ADMITTED_TIMECODE_FRAME_RATE_TOKENS.contains(&value)
}

fn is_admitted_time_base_token(value: &str) -> bool {
    ADMITTED_TIME_BASE_TOKENS.contains(&value)
}

fn require_exact(
    root: Node<'_, '_>,
    path: &[&str],
    expected: &str,
) -> Result<String, JobConfigurationError> {
    let value = required_text(root, path)?;
    if value != expected {
        return Err(JobConfigurationError::UnsupportedFieldValue);
    }
    Ok(value)
}

fn parse_file_reference(
    root: Node<'_, '_>,
    error: JobConfigurationError,
) -> Result<SelectedFileReference, JobConfigurationError> {
    let file_name = path_text(root, &["file_name"])?.ok_or(error)?;
    let local_path = path_text(root, &["storage", "local", "path"])?.ok_or(error)?;
    Ok(SelectedFileReference {
        file_name: file_name.to_owned(),
        local_path: local_path.to_owned(),
    })
}

/// Preserves an optional XML string, using the selected `auto` default when absent or empty.
fn optional_string_default_auto(
    root: Node<'_, '_>,
    path: &[&str],
) -> Result<String, JobConfigurationError> {
    match path_text(root, path)? {
        None | Some("") => Ok("auto".to_owned()),
        Some(value) if value.trim() == "auto" => Ok("auto".to_owned()),
        Some(value) => Ok(value.to_owned()),
    }
}

fn optional_authored_time_string(
    root: Node<'_, '_>,
    name: &str,
    allowed_value: &str,
) -> Result<String, JobConfigurationError> {
    match path_text(root, &[name])? {
        None => Ok(String::new()),
        Some(value) if value.trim().is_empty() => Ok(String::new()),
        Some(value) if value.trim() == allowed_value => Ok(allowed_value.to_owned()),
        Some(value) => Ok(value.to_owned()),
    }
}

fn required_silence_duration(
    root: Node<'_, '_>,
    name: &str,
    sample_rate_hz: u32,
) -> Result<(String, SelectedSilenceSampleCount), JobConfigurationError> {
    let value = required_text(root, &[name])?;
    let samples = silence_duration_samples(&value, sample_rate_hz)?;
    Ok((value, samples))
}

fn optional_silence_duration(
    root: Node<'_, '_>,
    name: &str,
    sample_rate_hz: u32,
) -> Result<(String, SelectedSilenceSampleCount, bool), JobConfigurationError> {
    match path_text(root, &[name])? {
        None => Ok(("0.0".to_owned(), SelectedSilenceSampleCount(0), false)),
        Some(value) => Ok((
            value.to_owned(),
            silence_duration_samples(value, sample_rate_hz)?,
            true,
        )),
    }
}

fn silence_duration_samples(
    value: &str,
    sample_rate_hz: u32,
) -> Result<SelectedSilenceSampleCount, JobConfigurationError> {
    let (whole, fraction) = match value.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (value, None),
    };
    if whole.is_empty()
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || fraction.is_some_and(|digits| {
            digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit())
        })
    {
        return Err(JobConfigurationError::TimeDomain);
    }

    let seconds = value
        .parse::<f64>()
        .map_err(|_| JobConfigurationError::TimeDomain)?;
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(JobConfigurationError::TimeDomain);
    }
    let scaled_samples = seconds * f64::from(sample_rate_hz);
    if !scaled_samples.is_finite() || scaled_samples >= i64::MAX as f64 {
        return Err(JobConfigurationError::TimeDomain);
    }
    let signed_samples = scaled_samples.trunc() as i64;
    if signed_samples < 0 {
        return Err(JobConfigurationError::TimeDomain);
    }
    let samples = u64::try_from(signed_samples).map_err(|_| JobConfigurationError::TimeDomain)?;
    Ok(SelectedSilenceSampleCount(samples))
}

fn parse_decimal_u32(value: &str) -> Result<u32, JobConfigurationError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(JobConfigurationError::UnsupportedFieldValue);
    }
    value
        .parse()
        .map_err(|_| JobConfigurationError::UnsupportedFieldValue)
}

fn parse_selected_mix_level(value: &str) -> Result<i8, JobConfigurationError> {
    if value == "-3" {
        Ok(-3)
    } else {
        Err(JobConfigurationError::UnsupportedFieldValue)
    }
}
